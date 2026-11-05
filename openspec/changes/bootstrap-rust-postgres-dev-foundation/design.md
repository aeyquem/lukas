# Design

## Context

See `proposal.md` for motivation and `specs/backend-runtime/spec.md` plus
`specs/container-deployment/spec.md` for externally observable requirements.
The repository currently has no backend or frontend implementation. The
accepted architecture selects one readable Rust crate, Axum/Tokio, SQLx with
PostgreSQL, embedded versioned migrations, Vue 3/Vite, and separate production
images. The single owner workspace and starter categories are resolved in
`desc/13-first-slice-decisions.md`.

## Goals / Non-Goals

**Goals:**

- Make a fresh PostgreSQL database reach backend readiness through migrations
  and repeatable owner-workspace/category bootstrap.
- Make health state and startup failures observable to Compose and operators.
- Establish a loopback-only browser path for local development and independent
  production images for the frontend and API.
- Keep the Rust startup and SQL readable to a Rust beginner.

**Non-Goals:**

- Implement financial account, transaction, budget, or close workflows.
- Add user authentication, LAN/public access, a CLI, or hosted deployment
  orchestration.
- Define production PostgreSQL provisioning, backup/restore, TLS, secrets
  delivery, or rolling schema compatibility; those remain release gates.

## Decisions

### Backend crate and startup

Use one crate rooted at `backend/`. Keep `main.rs` limited to configuration,
database connection, migration/bootstrap ordering, router construction, and
shutdown. Put runtime configuration, PostgreSQL setup, workspace bootstrap,
and health routes in small named modules. Use the selected Axum/Tokio and SQLx
stack with explicit SQL and typed row mappings; do not add an ORM or a
repository trait layer.

Startup order is: parse and validate configuration; connect with bounded
timeouts and pool size; apply embedded `backend/migrations/` through SQLx's
migration runner; create or load the single owner workspace and seed any
missing starter categories in a transaction; construct and bind the HTTP
server. Bound the complete migration/bootstrap phase with
`DATABASE_STARTUP_TIMEOUT` (default 30 seconds) so SQLx's migration advisory
lock and workspace writes cannot hold startup indefinitely. If the deadline
expires, cancel the operation, close the pool, and fail without binding the
HTTP server. The workspace receives a stable singleton key and a generated
UUID on first creation. Category slugs are unique within the workspace, so a
restart does not duplicate or reset categories the owner has edited. The
shared API state carries the resolved workspace ID explicitly.

The `--migrate-only` argument uses the same validated database configuration
and migrator, then exits without running workspace bootstrap or binding HTTP.
This keeps hosted schema rollout separable from application startup without
creating a second migration implementation.

`/health/live` only reflects that the process can answer requests. `/health/ready`
requires successful startup and performs a bounded `SELECT 1` check so a later
PostgreSQL outage is visible. Health routes remain available when readiness is
false. There are no business routes in this change. Graceful shutdown stops new
requests, drains the server within the configured deadline, and closes the
pool. The shutdown deadline starts only after the shutdown signal; it must not
limit normal server lifetime. Test lifecycle behavior with a held request and
with a healthy idle server that remains available beyond the drain timeout
until explicitly signaled.

Keep shutdown behavior in a small server lifecycle function so it can be
verified without adding a production business route. A test-only router holds
one request behind a synchronization primitive: release it before the
deadline to verify graceful drain and stop acceptance of new work; leave it
held to verify shutdown exits by the configured deadline.

The API disables CORS. Middleware requires the exact configured `Origin` on
state-changing browser requests and rejects missing or mismatched origins
before route handling. Health GET routes do not require an Origin. The API
does not treat `X-Forwarded-Host` or `X-Forwarded-Origin` as trusted input; a
private container network does not replace this API-side check.

Read environment variables directly into a small typed config struct rather
than adding a configuration framework. Require `DATABASE_URL` and
`CANONICAL_ORIGIN`; use the accepted defaults for bind address, pool size,
connect/acquire timeouts, and shutdown timeout. Validate numeric ranges and the
origin shape at startup. Keep secret values out of logs and errors.

### Initial persistence model

The initial migration sequence creates only the foundation needed now: an
owner workspace and workspace-scoped expense categories. Keep workspace and
category schema changes in ordered immutable migrations so an integration test
can prove that a database at the prior migration version advances on startup.
Use UUID primary keys, a stable
singleton workspace key, a stable category slug, name, sort order, archived
state, and UTC creation/update instants. Add a uniqueness constraint for the
singleton and for each category slug within its workspace. Seed the accepted
starter expense categories as workspace rows in bootstrap, not as a hard-coded
read-only API list. Account, wallet, transaction, budget, and idempotency tables
belong to their vertical slices and are not pre-created speculatively.

Migrations are immutable, ordered SQL files embedded into the binary. SQLx's
migration lock serializes concurrent startup migration attempts. Fresh-database
and repeated-startup tests use real PostgreSQL; unit-only mocks are not
evidence for migration locking or persistence.

### Local Compose and proxy

Use root `compose.yaml` with `postgres`, `api`, `vite`, and `dev-proxy`. Postgres
uses a named development volume and a health check. The API waits for healthy
PostgreSQL and is checked through its readiness endpoint. Only `dev-proxy`
publishes `127.0.0.1:4173`; all other ports stay on the Compose network.

Use Nginx for `dev-proxy` and the static production frontend. The development
proxy sends all HTTP and HMR WebSocket traffic to Vite on `vite:5173` and
validates the canonical Host. Vite forwards `/api/` and `/health/` unchanged to
`api:8080`, preserving the original Origin. SPA fallback applies only to
frontend routes. The API itself checks exact Origin on state-changing requests;
the proxy's Host check is not used as a substitute.

For Vite, bind the source and configuration files individually so container
dependencies are not hidden by a source mount. Rebuild the Vite image when its
package manifest or lockfile changes. Use the selected frontend package and
lockfile at `frontend/app/`; do not rely on host `node_modules`.

The production backend image is a multi-stage Rust build with a small runtime
image. The production frontend image builds Vue/Vite assets in Node and serves
only static files with Nginx; it does not run a Node app server and does not
depend on Compose. Independent images are build/deployment units, not approval
to expose this unauthenticated first version. The supported topology keeps the
frontend loopback-published and the API private until access control receives a
separate review. At container start, the frontend image generates a
non-secret `/runtime-config.js` from `API_BASE_URL` and renders its Nginx
upstream from `API_UPSTREAM`. Validate or safely encode values before writing
JavaScript/configuration. SPA fallback applies only to client routes; `/api`
requests proxy to the configured backend. The same frontend image can therefore
serve multiple backend locations without rebuilding assets.

The local setup documentation must state that the unauthenticated stack is for
same-machine use only and must not be exposed to a LAN or public proxy. Wider
access waits for a separately reviewed access-control design.

### Dependency and image versions

Use Cargo and npm lockfiles as the exact build inputs. At implementation, pin
the selected Axum/SQLx and Node/PostgreSQL/Nginx image versions in manifests and
Dockerfiles, then record the resolved versions in the lockfiles or image tags.
Do not use floating `latest` tags. The current official image listings expose
PostgreSQL 18.6 and Node 24 LTS tags; verify the chosen patch tags immediately
before implementation and builds. (See the [PostgreSQL official image tags](https://hub.docker.com/_/postgres)
and [Node official image tags](https://hub.docker.com/_/node).)

## Risks / Trade-offs

- **Concurrent first startup can race during singleton creation** → use a
  database uniqueness constraint and an atomic insert-or-load transaction;
  verify concurrent bootstrap against PostgreSQL.
- **A readiness probe can briefly fail during a transient database restart** →
  keep liveness independent, bound the readiness query, and let Compose restart
  policy remain conservative rather than treating every readiness failure as a
  process crash.
- **Environment substitution can inject invalid JavaScript or Nginx config** →
  validate allowed URL forms and JSON-escape public runtime values before
  generating files; never expose secrets in frontend config.
- **Development bind mounts can hide installed dependencies** → bind only
  source/config paths and rebuild when the lockfile changes.
- **A foundation migration can become difficult to evolve** → keep the schema
  limited to workspace and categories, make every migration additive and
  immutable, and add financial tables with their own later migrations.

## Migration Plan

There is no deployed database to migrate. Build the initial image and migration
from an empty database, verify restart behavior against the same named volume,
and keep the migration-only command available for later hosted rollout. If the
foundation schema needs correction before release, add a new migration rather
than editing an applied migration.
