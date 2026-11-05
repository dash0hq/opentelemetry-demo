// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0
package main

import (
	"bytes"
	"context"
	"io"
	"net"
	"runtime/pprof"
	"strings"
	"testing"
	"time"

	"go.opentelemetry.io/contrib/instrumentation/google.golang.org/grpc/otelgrpc"
	"go.opentelemetry.io/otel/propagation"
	sdktrace "go.opentelemetry.io/otel/sdk/trace"
	"go.opentelemetry.io/otel/trace"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/credentials/insecure"
	healthpb "google.golang.org/grpc/health/grpc_health_v1"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/status"
	"google.golang.org/grpc/test/bufconn"
)

type profilingHealthServer struct {
	healthpb.UnimplementedHealthServer
	checkLabels func(context.Context)
}

func (s *profilingHealthServer) Check(ctx context.Context, req *healthpb.HealthCheckRequest) (*healthpb.HealthCheckResponse, error) {
	s.checkLabels(ctx)
	if req.Service == "error" {
		return nil, status.Error(codes.NotFound, "unknown service")
	}
	return &healthpb.HealthCheckResponse{Status: healthpb.HealthCheckResponse_SERVING}, nil
}

func (s *profilingHealthServer) Watch(req *healthpb.HealthCheckRequest, stream healthpb.Health_WatchServer) error {
	s.checkLabels(stream.Context())
	if req.Service == "error" {
		return status.Error(codes.NotFound, "unknown service")
	}
	return stream.Send(&healthpb.HealthCheckResponse{Status: healthpb.HealthCheckResponse_SERVING})
}

func TestGRPCProfilingLabels(t *testing.T) {
	for _, traceID := range []string{"00112233445566778899aabbccddeeff", "ffeeddccbbaa99887766554433221100", ""} {
		t.Run("trace="+traceID, func(t *testing.T) {
			var provider trace.TracerProvider = trace.NewNoopTracerProvider()
			if traceID != "" {
				tp := sdktrace.NewTracerProvider()
				defer tp.Shutdown(context.Background())
				provider = tp
			}
			checkLabels := func(ctx context.Context) {
				if got, _ := pprof.Label(ctx, "otel.trace_id"); got != traceID {
					t.Errorf("context trace label = %q, want %q", got, traceID)
				}
				var profile bytes.Buffer
				if err := pprof.Lookup("goroutine").WriteTo(&profile, 1); err != nil {
					t.Error(err)
				}
				if traceID != "" && !strings.Contains(profile.String(), `"otel.trace_id":"`+traceID+`"`) {
					t.Errorf("goroutine profile is missing trace label %s", traceID)
				}
				if traceID == "" && strings.Contains(profile.String(), `"otel.trace_id":`) {
					t.Error("untraced RPC inherited a trace label")
				}
			}
			lis := bufconn.Listen(1024 * 1024)
			srv := grpc.NewServer(
				grpc.StatsHandler(otelgrpc.NewServerHandler(
					otelgrpc.WithTracerProvider(provider),
					otelgrpc.WithPropagators(propagation.TraceContext{}),
				)),
				grpc.ChainUnaryInterceptor(profilingUnaryInterceptor),
				grpc.ChainStreamInterceptor(profilingStreamInterceptor),
			)
			healthpb.RegisterHealthServer(srv, &profilingHealthServer{checkLabels: checkLabels})
			defer srv.Stop()
			go srv.Serve(lis)
			ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
			defer cancel()
			conn, err := grpc.DialContext(ctx, "passthrough:///profiling-test",
				grpc.WithContextDialer(func(context.Context, string) (net.Conn, error) { return lis.Dial() }),
				grpc.WithTransportCredentials(insecure.NewCredentials()),
			)
			if err != nil {
				t.Fatal(err)
			}
			defer conn.Close()
			if traceID != "" {
				ctx = metadata.NewOutgoingContext(ctx, metadata.Pairs("traceparent", "00-"+traceID+"-0123456789abcdef-01"))
			}
			client := healthpb.NewHealthClient(conn)
			for _, service := range []string{"", "error"} {
				wantCode := codes.OK
				if service == "error" {
					wantCode = codes.NotFound
				}
				resp, err := client.Check(ctx, &healthpb.HealthCheckRequest{Service: service})
				if status.Code(err) != wantCode || (err == nil && resp.Status != healthpb.HealthCheckResponse_SERVING) {
					t.Fatalf("Check = %v, %v; want SERVING or %v", resp, err, wantCode)
				}
				stream, err := client.Watch(ctx, &healthpb.HealthCheckRequest{Service: service})
				if err != nil {
					t.Fatal(err)
				}
				resp, err = stream.Recv()
				if status.Code(err) != wantCode || (err == nil && resp.Status != healthpb.HealthCheckResponse_SERVING) {
					t.Fatalf("Watch = %v, %v; want SERVING or %v", resp, err, wantCode)
				}
				if err == nil {
					if _, err := stream.Recv(); err != io.EOF {
						t.Fatalf("Watch termination = %v, want EOF", err)
					}
				}
			}
		})
	}
}
