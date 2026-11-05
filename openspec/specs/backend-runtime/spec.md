# Backend Runtime Specification

## Purpose

Defines how the owner-only backend starts, prepares its PostgreSQL database,
exposes health state, and shuts down safely for self-hosted operation.

## Requirements

### Requirement: Backend configuration is validated before startup
The backend SHALL require valid `DATABASE_URL` and `CANONICAL_ORIGIN` values and SHALL reject invalid runtime configuration before serving requests. Unless overridden, it SHALL bind to `0.0.0.0:8080`, use at most five database connections, use five-second connection and acquisition timeouts, allow thirty seconds for database startup/migrations, and allow ten seconds for shutdown. It SHALL NOT include secret configuration values in logs or error responses.

#### Scenario: Required configuration is missing or invalid
- **WHEN** the backend starts without a valid required setting
- **THEN** it exits with a clear configuration error and does not report readiness

#### Scenario: Runtime values use defaults
- **WHEN** valid required settings are provided and optional runtime settings are omitted
- **THEN** the backend uses the documented bind address, pool size, connection/acquisition/startup timeouts, and shutdown limit

#### Scenario: Invalid origin or timeout is configured
- **WHEN** `CANONICAL_ORIGIN` has an invalid shape or a configured timeout/pool value is outside its accepted range
- **THEN** startup fails with a clear non-secret configuration error

### Requirement: Migrations and owner workspace bootstrap precede readiness
The backend SHALL connect to PostgreSQL, apply pending versioned migrations with SQLx migration locking, and idempotently create or load one owner workspace and its starter expense categories before becoming ready. The migration and bootstrap phase SHALL finish within `DATABASE_STARTUP_TIMEOUT` or startup SHALL fail without becoming ready.

#### Scenario: Fresh database startup
- **WHEN** the backend starts against an empty reachable database
- **THEN** migrations and owner workspace/category bootstrap complete before readiness succeeds

#### Scenario: Concurrent or repeated startup
- **WHEN** multiple backend starts or a restart performs the same bootstrap
- **THEN** exactly one owner workspace and one set of starter categories remain

#### Scenario: Migration or bootstrap fails
- **WHEN** a required migration or workspace bootstrap operation fails
- **THEN** the backend does not report readiness and does not serve application requests

#### Scenario: Migration lock or bootstrap exceeds its deadline
- **WHEN** a migration lock or workspace bootstrap remains blocked beyond `DATABASE_STARTUP_TIMEOUT`
- **THEN** startup fails within the configured bound, releases database resources, and does not report readiness

### Requirement: Backend exposes liveness and readiness separately
The backend SHALL expose `GET /health/live` for process liveness and `GET /health/ready` only after migrations and workspace bootstrap succeed. Readiness SHALL become unsuccessful when PostgreSQL is unavailable.

#### Scenario: Backend is running
- **WHEN** the HTTP server is responding
- **THEN** liveness succeeds independently of the current database readiness state

#### Scenario: Database becomes unavailable
- **WHEN** PostgreSQL is unavailable after startup
- **THEN** readiness is unsuccessful until the backend can use PostgreSQL again

### Requirement: Browser writes are restricted to the canonical origin
The API SHALL disable CORS and SHALL require an exact `Origin` match with `CANONICAL_ORIGIN` on state-changing browser requests. It SHALL NOT use forwarded host/origin headers to satisfy this check. Read-only health requests remain usable without an `Origin` header.

#### Scenario: State-changing request uses the canonical origin
- **WHEN** a state-changing request includes the configured exact Origin
- **THEN** the API origin check allows the request to continue to its route

#### Scenario: State-changing request has a missing or mismatched origin
- **WHEN** a state-changing request has no Origin or an Origin different from `CANONICAL_ORIGIN`
- **THEN** the API rejects it before route handling

#### Scenario: Forwarded origin is supplied without a browser origin
- **WHEN** a state-changing request supplies `X-Forwarded-Origin` but no valid `Origin`
- **THEN** the API rejects it

#### Scenario: Health request has no origin
- **WHEN** a health endpoint is requested without an Origin header
- **THEN** the endpoint responds according to its health state

### Requirement: Backend supports migration-only operation
The backend SHALL provide a `--migrate-only` mode that applies the same embedded migrations and exits without starting the HTTP server.

#### Scenario: Hosted migration step
- **WHEN** the executable is invoked with `--migrate-only`
- **THEN** migrations run and the process exits without binding an HTTP port

### Requirement: Backend shutdown is bounded
The backend SHALL remain available until a shutdown signal arrives. On that signal, it SHALL stop accepting new work and drain in-flight requests for no longer than the configured shutdown timeout.

#### Scenario: Shutdown with active requests
- **WHEN** shutdown begins while requests are in flight
- **THEN** new work is stopped and in-flight work is drained until completion or the timeout expires

#### Scenario: Server runs longer than its shutdown timeout
- **WHEN** no shutdown signal arrives during normal service beyond the configured shutdown timeout
- **THEN** the server continues responding and does not start draining
