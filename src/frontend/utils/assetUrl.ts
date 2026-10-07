// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

/**
 * Session replays reference images by URL, and the replay viewer can only load them from a public
 * https origin. The load generator browses the shop through an in-cluster hostname, so relative
 * paths would resolve to URLs nobody outside the cluster can reach. Release images therefore serve
 * assets from jsDelivr, which mirrors this public repository at the release tag.
 */
const RELEASE_ASSETS_BASE_URL = 'https://cdn.jsdelivr.net/gh/dash0hq/opentelemetry-demo@{version}/src/frontend/public';

/**
 * Server-side only. PUBLIC_ASSETS_BASE_URL wins when set (empty disables the default); otherwise
 * release builds, which have DEMO_RELEASE_VERSION baked in, use the release tag. Other builds
 * (local, CI) keep relative paths.
 */
export function resolveAssetsBaseUrl(): string {
  const { PUBLIC_ASSETS_BASE_URL, DEMO_RELEASE_VERSION = '' } = process.env;
  if (PUBLIC_ASSETS_BASE_URL !== undefined) {
    return PUBLIC_ASSETS_BASE_URL;
  }
  return /^v?\d+\.\d+\.\d+([-+].*)?$/.test(DEMO_RELEASE_VERSION)
    ? RELEASE_ASSETS_BASE_URL.replace('{version}', DEMO_RELEASE_VERSION)
    : '';
}

/**
 * Resolves a path under /public against the assets base URL. The browser reads the value the
 * server resolved from window.ENV, so server and client render the same URL.
 */
export function assetUrl(path: string): string {
  const base = typeof window !== 'undefined' ? window.ENV?.NEXT_PUBLIC_ASSETS_BASE_URL : resolveAssetsBaseUrl();
  return base ? base.replace(/\/+$/, '') + path : path;
}
