# Lukas

Lukas is a personal-finance project focused initially on Chile. The rewrite is
starting from a clean implementation state with a Rust backend and a web app.
The backend and frontend have separate Dockerfiles for independent deployment
to Docker-compatible environments; production does not require Compose. Local
development uses Docker Compose and the local Docker Engine. The initial
instance is owner-only, with a possible SaaS path later; its owner may manage
personal or household finances.

The product will also provide a CLI for people and agents to query and add
data. MCP is deprecated for now.

The first slice and initial self-hosted setup use PostgreSQL. Local development
runs the web frontend, Rust API and PostgreSQL in containers with Docker
Compose. SQLite is deferred as a possible later local desktop option.

There is no implementation scaffold yet. The current product and domain
context, rewrite direction, and unresolved design topics are in
[`desc/00-rewrite-context.md`](desc/00-rewrite-context.md). Current product
decisions, acceptance criteria, domain model, and planning notes are under
`desc/`, including the [technical stack proposal](desc/09-technical-stack-proposal.md).
Direct owner discussions and accepted decisions take precedence over proposals
that have not been accepted. The obsolete PoC files under `docs/` do not define
current product or implementation requirements.
