// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use actix_web::{post, web, HttpResponse, Responder};
use open_feature::provider::FeatureProvider;
use open_feature::EvaluationContext;
use tracing::{info, warn};

mod quote;
use quote::create_quote_from_count;

mod tracking;
use tracking::create_tracking_id;

mod shipping_types;
pub use shipping_types::*;

const NANOS_MULTIPLE: u32 = 10000000u32;

#[post("/get-quote")]
pub async fn get_quote(
    req: web::Json<GetQuoteRequest>,
    flag_provider: web::Data<dyn FeatureProvider>,
) -> impl Responder {
    simulate_slowness(&flag_provider).await;

    let itemct: u32 = req.items.iter().map(|item| item.quantity as u32).sum();

    let quote = match create_quote_from_count(itemct).await {
        Ok(q) => q,
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!("Failed to get quote: {}", e));
        }
    };

    let reply = GetQuoteResponse {
        cost_usd: Some(Money {
            currency_code: "USD".into(),
            units: quote.dollars,
            nanos: quote.cents * NANOS_MULTIPLE,
        }),
    };

    info!(
        name: "shipping.quote.sent",
        dollars = quote.dollars,
        cents = quote.cents,
        "Sending Quote"
    );

    HttpResponse::Ok().json(reply)
}

#[post("/ship-order")]
pub async fn ship_order(
    req: web::Json<ShipOrderRequest>,
    flag_provider: web::Data<dyn FeatureProvider>,
) -> impl Responder {
    simulate_slowness(&flag_provider).await;

    let is_outside_us = req
        .address
        .as_ref()
        .map(|addr| {
            !matches!(
                addr.country.to_uppercase().trim(),
                "US" | "USA" | "UNITED STATES" | "UNITED STATES OF AMERICA"
            )
        })
        .unwrap_or(false);

    let slowdown_secs = if is_outside_us {
        flag_provider
            .resolve_int_value("intlShippingSlowdown", &EvaluationContext::default())
            .await
            .map(|res| {
                info!(
                    name: "shipping.feature_flag.evaluated",
                    feature_flag_key = "intlShippingSlowdown",
                    feature_flag_provider_name = "flagd",
                    feature_flag_variant = res.variant.as_deref().unwrap_or("unknown"),
                );
                res.value
            })
            .unwrap_or_else(|e| {
                warn!(name: "shipping.feature_flag.evaluation_failed", error = ?e);
                0
            })
    } else {
        0
    };

    if slowdown_secs > 0 {
        info!(
            name: "shipping.international.slowdown",
            delay_secs = slowdown_secs,
            "Delaying international shipment due to intlShippingSlowdown feature flag"
        );
        actix_web::rt::time::sleep(std::time::Duration::from_secs(slowdown_secs as u64)).await;
    }

    let tid = create_tracking_id();
    info!(
        name: "shipping.tracking_id.created",
        tracking_id = tid.as_str(),
        "Tracking ID Created"
    );
    HttpResponse::Ok().json(ShipOrderResponse { tracking_id: tid })
}

/// Waits a random time between the lower and upper bound flags, with the
/// probability set by the shippingServiceSimulateSlowness flag.
async fn simulate_slowness(flag_provider: &web::Data<dyn FeatureProvider>) {
    let ctx = EvaluationContext::default();
    let probability = flag_provider
        .resolve_float_value("shippingServiceSimulateSlowness", &ctx)
        .await
        .map(|res| res.value)
        .unwrap_or(0.0);
    if random_unit() >= probability {
        return;
    }
    let lower_bound = flag_provider
        .resolve_int_value("shippingServiceSimulateSlownessLowerBound", &ctx)
        .await
        .map(|res| res.value)
        .unwrap_or(0);
    let upper_bound = flag_provider
        .resolve_int_value("shippingServiceSimulateSlownessUpperBound", &ctx)
        .await
        .map(|res| res.value)
        .unwrap_or(1000);
    let delay_millis =
        lower_bound + (random_unit() * (upper_bound - lower_bound).max(0) as f64) as i64;
    info!(
        name: "shipping.simulated_slowness",
        delay_millis = delay_millis,
        "Simulating shipping service slowness"
    );
    actix_web::rt::time::sleep(std::time::Duration::from_millis(delay_millis as u64)).await;
}

/// Returns a random number in [0, 1).
// ponytail: takes the 53 low random bits of a v4 UUID to avoid a new rand dependency; use rand if more randomness is needed.
fn random_unit() -> f64 {
    let bits = (uuid::Uuid::new_v4().as_u128() as u64) & ((1u64 << 53) - 1);
    bits as f64 / (1u64 << 53) as f64
}

#[cfg(test)]
mod tests {
    use actix_web::{http::header::ContentType, test, App};
    use open_feature::provider::{MockFeatureProvider, ResolutionDetails};
    use std::sync::Arc;

    use super::*;

    fn mock_provider(value: i64) -> web::Data<dyn FeatureProvider> {
        let mut mock = MockFeatureProvider::new();
        mock.expect_resolve_int_value()
            .returning(move |_, _| Ok(ResolutionDetails::new(value)));
        mock.expect_resolve_float_value()
            .returning(|_, _| Ok(ResolutionDetails::new(0.0)));
        web::Data::from(Arc::new(mock) as Arc<dyn FeatureProvider>)
    }

    async fn call_ship_order(
        address: Option<Address>,
        provider: web::Data<dyn FeatureProvider>,
    ) -> ShipOrderResponse {
        let app = test::init_service(App::new().app_data(provider).service(ship_order)).await;
        let req = test::TestRequest::post()
            .uri("/ship-order")
            .insert_header(ContentType::json())
            .set_json(&ShipOrderRequest {
                address,
                items: vec![],
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        test::read_body_json(resp).await
    }

    fn make_address(country: &str) -> Address {
        Address {
            street_address: "123 Main St".into(),
            city: "Anytown".into(),
            state: "CA".into(),
            country: country.into(),
            zip_code: "00000".into(),
        }
    }

    #[actix_web::test]
    async fn test_ship_order_no_address() {
        let order = call_ship_order(None, mock_provider(0)).await;
        assert!(!order.tracking_id.is_empty());
    }

    #[actix_web::test]
    async fn test_ship_order_us_address() {
        let order = call_ship_order(Some(make_address("US")), mock_provider(0)).await;
        assert!(!order.tracking_id.is_empty());
    }

    #[actix_web::test]
    async fn test_ship_order_international_flag_off() {
        let order = call_ship_order(Some(make_address("FR")), mock_provider(0)).await;
        assert!(!order.tracking_id.is_empty());
    }

    #[actix_web::test]
    async fn test_ship_order_international_flag_on() {
        let start = std::time::Instant::now();
        let order = call_ship_order(Some(make_address("FR")), mock_provider(1)).await;
        assert!(start.elapsed() >= std::time::Duration::from_secs(1));
        assert!(!order.tracking_id.is_empty());
    }

    #[actix_web::test]
    async fn test_ship_order_simulated_slowness() {
        let mut mock = MockFeatureProvider::new();
        // Both bounds resolve to 300ms, so the delay is exactly 300ms.
        mock.expect_resolve_int_value()
            .returning(|_, _| Ok(ResolutionDetails::new(300)));
        mock.expect_resolve_float_value()
            .returning(|_, _| Ok(ResolutionDetails::new(1.0)));
        let provider = web::Data::from(Arc::new(mock) as Arc<dyn FeatureProvider>);

        let start = std::time::Instant::now();
        let order = call_ship_order(Some(make_address("US")), provider).await;
        assert!(start.elapsed() >= std::time::Duration::from_millis(300));
        assert!(!order.tracking_id.is_empty());
    }

    #[actix_web::test]
    async fn test_ship_order_us_flag_on() {
        let start = std::time::Instant::now();
        let order = call_ship_order(Some(make_address("US")), mock_provider(10)).await;
        assert!(start.elapsed() < std::time::Duration::from_secs(1));
        assert!(!order.tracking_id.is_empty());
    }
}
