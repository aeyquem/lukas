# Proposal

## Why

The rewrite has product and technical decisions ready for implementation, but
it has no running application scaffold. Establishing the backend, database,
workspace bootstrap, and container paths first gives later financial workflows
a real PostgreSQL-backed runtime to build on.

## What Changes

- Add a readable single-crate Rust API using Axum/Tokio and SQLx with PostgreSQL.
- Add typed runtime configuration, versioned SQL migrations, and an idempotent
  owner-workspace plus starter-category bootstrap.
- Add liveness and readiness endpoints with readiness gated on database
  connection, migrations, and workspace bootstrap.
- Add local Docker Compose development services for PostgreSQL, API, Vite, and
  a loopback-published development proxy.
- Add independent backend and static frontend production images. The frontend
  image serves SPA routes, proxies `/api`, and loads public runtime API
  configuration.

## Capabilities

### New Capabilities

- `backend-runtime`: Backend startup, configuration, migrations, owner
  workspace bootstrap, health endpoints, and graceful shutdown behavior.
- `container-deployment`: Local development topology and independent
  production backend/frontend container behavior.

### Modified Capabilities

None.

## Impact

Adds the Rust backend crate and migration directory, a minimal Vue/Vite frontend
shell, backend and frontend Dockerfiles, local Compose and proxy configuration,
and focused runtime/deployment checks. This change establishes infrastructure
only; account and transaction workflows, financial calculations, authentication,
CLI behavior, and production operations remain out of scope.
