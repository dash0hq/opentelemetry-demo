# OpenTelemetry Demo Commands and Guidelines

## Build Commands
- Build all services: `make build`
- Start the demo: `make start` 
- Start minimal demo: `make start-minimal`
- Stop the demo: `make stop`
- Run all tests: `make run-tests`
- Restart a service: `make restart service=<service-name>`
- Rebuild and restart a service: `make redeploy service=<service-name>`
- Generate protobuf: `make generate-protobuf`

## Code Style Guidelines
- Follow language-specific conventions for each microservice
- Use OpenTelemetry instrumentation libraries for all services
- Add comprehensive spans and attributes following OpenTelemetry semantic conventions
- Prefix commit messages with service name when making service-specific changes
- Error handling should include proper span recording with error attributes
- Import patterns should match existing service code organization
- Follow 12-factor app principles for configuration (environment variables)
- Format code according to language-specific formatters
- Document public APIs with clear descriptions and examples

## Documentation
- Update CHANGELOG.md when making notable changes
- Keep README.md up to date with accurate build/run instructions