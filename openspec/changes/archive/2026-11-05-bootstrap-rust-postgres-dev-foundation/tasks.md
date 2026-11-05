# Tasks

## 1. Rust backend and PostgreSQL foundation

- [x] 1.1 Create the single-crate `backend/` application with the selected Axum/Tokio and SQLx/PostgreSQL dependencies, a committed Cargo lockfile, and typed runtime configuration; unit-test missing `DATABASE_URL`/`CANONICAL_ORIGIN`, canonical-origin shape, accepted defaults, timeout/pool bounds, and sanitized errors.
- [x] 1.2 Add immutable ordered workspace/category migrations and transactionally seed or load the owner workspace and accepted starter categories; verify fresh, repeated, concurrent, and prior-version-to-current migration behavior against real PostgreSQL.
- [x] 1.3 Add startup migration ordering, `--migrate-only`, liveness/readiness endpoints, bounded readiness checks, graceful shutdown, and API Origin middleware; test that `DATABASE_STARTUP_TIMEOUT` bounds migrations/bootstrap (including a held migration lock), readiness success/failure against database availability, exact versus missing/mismatched Origin and forwarded-origin bypass rejection, origin-free health GETs, migration-only no-bind behavior, and test-only held requests that verify the server outlives its shutdown timeout before a signal, graceful drain, stopped new work, and forced exit by the configured deadline.
- [x] 1.4 Add the independent multi-stage backend Dockerfile and runtime health check; verify the image builds and runs with a separately started PostgreSQL container, without Docker Compose or the frontend image.

## 2. Static Vue frontend image

- [x] 2.1 Create the minimal Vue 3, strict TypeScript, and Vite app shell under `frontend/app/` with a committed npm lockfile; verify a production Vite build succeeds.
- [x] 2.2 Add the independent static frontend Dockerfile, Nginx SPA/API routing, and safe runtime-config generation from `API_BASE_URL` and `API_UPSTREAM`; test invalid/injection-shaped URLs, confirm public config contains no secrets, and verify the image serves client routes and proxies `/api` without Compose or a Node server.

## 3. Local development topology

- [x] 3.1 Add Compose services `postgres`, `api`, `vite`, and `dev-proxy`, with PostgreSQL persistence, health dependencies, private service ports, and only `127.0.0.1:4173` published; verify Compose configuration and service health.
- [x] 3.2 Add the development proxy and Vite setup for exact Host validation, unchanged `/api` and `/health` forwarding with Origin preserved, HMR WebSockets, and SPA fallback exclusions; verify valid/invalid Hosts, direct API Origin enforcement (not only proxy rejection), path preservation, HMR, and browser route fallback.
- [x] 3.3 Document local setup, non-secret environment examples, and the same-machine-only boundary (no LAN/public exposure without separately reviewed access control); verify the documented startup and shutdown commands against the real Docker Engine path.

## 4. Integrated runtime evidence

- [x] 4.1 Exercise fresh startup, a second startup, an upgrade from the preceding migration version, concurrent workspace bootstrap, restart persistence, migration-only mode, database outage/readiness, and bounded shutdown through the real Compose/PostgreSQL stack; verify workspace/categories with PostgreSQL queries and record the results in the handoff.
- [x] 4.2 Request `GET http://localhost:4173/health/ready` through dev-proxy → Vite → API → PostgreSQL and verify it returns ready only when its database `SELECT 1` succeeds; confirm the canonical loopback port is the only host-published port and record that financial API/browser workflows are later vertical-slice work.

## Workflow follow-up

- Run the independent reviewer and tester cycle until the staged implementation passes review.
- Keep `orchestrator.md` updated with the feature branch, OpenSpec progress, implementation evidence, and next action.
- Archive the OpenSpec change only after verification passes.
