# Product Catalog Service

When this service is run the output should be similar to the following

```json
{"message":"successfully parsed product catalog json","severity":"info","timestamp":"2022-06-02T23:54:10.191283363Z"}
{"message":"starting grpc server at :3550","severity":"info","timestamp":"2022-06-02T23:54:10.191849078Z"}
```

## Local Build

To build the service binary, run:

```sh
go build -o /go/bin/productcatalogservice/
```

## Docker Build

From the root directory, run:

```sh
docker compose build productcatalogservice
```

## Regenerate protos

> [!NOTE]
> [`protoc`](https://grpc.io/docs/protoc-installation/) is required.

To regenerate gRPC code run:

```sh
go generate
```

## Bump dependencies

To bump all dependencies run:

```sh
go get -u -t ./...
go mod tidy
```

## Profiling

The service serves Go pprof endpoints on pod port `6060`, including
`/debug/pprof/heap` and `/debug/pprof/profile`. This listener has no
authentication. Keep it internal and use a pod port-forward for local access;
do not route it through public ingress or publish it as a host port.

Unary and streaming gRPC handlers attach the active trace ID as the
`otel.trace_id` goroutine label. Labels are restored after each RPC and are
available in CPU and goroutine profiles, but not in Go heap profiles.
