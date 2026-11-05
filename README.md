# Lukas

Lukas is a personal-finance application focused initially on Chile. The
rewrite uses a Rust API, a Vue web application, and PostgreSQL. The owner-only
instance may manage personal or household finances. A CLI for people and agents
is planned; MCP is deprecated for now.

The backend and frontend have independent Dockerfiles for separate deployment.
Production does not require Compose. Local development uses Docker Compose and
the local Docker Engine. SQLite remains a possible later desktop option.

## Local development

Requirements: Docker Engine and the Docker Compose plugin.

```sh
cp .env.example .env
docker compose up --build
```

Open [http://localhost:4173](http://localhost:4173). The same-machine health
check is [http://localhost:4173/health/ready](http://localhost:4173/health/ready).
The development proxy forwards requests to Vite, which forwards `/api` and
`/health` to the private API container. Only the proxy is published, bound to
loopback; the API, Vite, and PostgreSQL ports are private to Compose.

Stop the running stack with `Ctrl+C`, or run `docker compose down` from another
terminal. The named PostgreSQL volume preserves local data across container
restarts and `docker compose down`. `docker compose down --volumes` deletes that
local database volume.

This initial deployment has no authentication and is for same-machine use
only. Do not expose the frontend or backend to a LAN or public proxy. Wider
access requires a separately reviewed access-control design. The example
PostgreSQL credentials are for local development only; do not reuse them for a
self-hosted or public deployment. Keep custom local passwords URL-safe because
Compose uses them to construct the API's database URL.

When `frontend/app/package.json` or its lockfile changes, rebuild the Vite
service from the lockfile:

```sh
docker compose build vite
```

## Project context

The current product and domain context, rewrite direction, and unresolved
design topics are in [`desc/00-rewrite-context.md`](desc/00-rewrite-context.md).
First-slice behavior and architecture decisions are under `desc/`, especially
[`desc/13-first-slice-decisions.md`](desc/13-first-slice-decisions.md). Direct
owner discussions and accepted decisions take precedence over proposals that
have not been accepted. The obsolete PoC files under `docs/` do not define
current product or implementation requirements.
