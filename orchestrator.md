# Lukas implementation handoff

Status: **first-slice decisions reviewed and ready for development**.
The owner asked orchestration to resolve missing decisions without pausing for
questions and delegated technical decisions to a technical committee. This
planning task does not begin application implementation.

## Source of truth

Read `AGENTS.md`, then `desc/00-rewrite-context.md`. For the accepted first
web-slice scope, use `desc/05-mvp-acceptance.md` and
`desc/08-first-slice-plan.md`. The committee's concrete decisions and the
product-reference findings are consolidated in
`desc/13-first-slice-decisions.md`. `desc/12-pre-implementation-checklist.md`
tracks the audit gate, implementation evidence, release gates, and later
slices. `desc/09-technical-stack-proposal.md` remains broader context;
`desc/10-backend-architecture.md` and `desc/11-frontend-architecture.md`
retain implementation detail, with the decision record controlling any
resolved first-slice point that they previously marked open.

The old PoC documentation is historical and must not be used as current
implementation requirements. Preserve provenance, reviewable corrections,
decimal-safe money, actual dates, period context, and explainable projections.
Do not pull unresolved goals, pools, forecasts, imports, reconciliation, CLI,
or hosted identity behavior into the first slice.

## Reviewed first slice

The slice is accounts with dated signed opening balances, manual income,
expense and transfer activity, current-month budget assignments, and explicit
month close. It includes the accepted same-machine owner-only boundary,
PostgreSQL, a Rust backend, a Vue web client, independent backend/frontend
Dockerfiles, and local Docker Engine Compose. The owner may manage personal or
household finances; separate member identities are later scope. The CLI is a
planned interface, but its contract is required only before its later
implementation slice. MCP is deprecated.

The first slice preserves the accepted product details in
`desc/08-first-slice-plan.md`, including foreign opening liability funding
basis, incomplete foreign expense behavior, same-day/pre-opening choices,
provisional signed category carry, and no value history. The detailed period,
edit, API, money, runtime, frontend, and deployment contracts are in
`desc/13-first-slice-decisions.md`.

## Technical committee decision

Backend/persistence, finance-ledger, and frontend/deployment specialists
reviewed their areas. Their decisions are consolidated in
`desc/13-first-slice-decisions.md`. The chosen baseline is a readable one-crate
Rust modular monolith with Axum/Tokio, SQLx and PostgreSQL; Vue 3, strict
TypeScript and Vite; server-authoritative financial calculations; and the
accepted independent image and local Compose boundaries. Optional framework,
grid, chart, and UI-suite comparisons are not code gates.

## Writer/reviewer cycle and next action

The orchestration writer consolidated specialist proposals. The independent
reviewer passed the decision record after two corrections: a precise
once-only cross-currency transfer valuation rule and dated funding treatment
for account openings added after setup. Readiness is recorded in
`desc/12-pre-implementation-checklist.md`; runtime behavior remains to be
validated during development. This orchestration task did not start application
code. Self-hosted release operations and later feature contracts remain
tracked separately in the checklist and source documents.

## Rewrite base branch handoff

Status: **saving the rewrite baseline before implementation**. Preserve the
existing remote `main`; it contains a different history from the rewrite
working branch. Save the current clean-implementation rewrite state on a new
`rewrite-base` branch and use it as the parent of later feature branches.
Current changes include the intentional PoC removals, accepted rewrite context
and first-slice decisions, OpenSpec setup, and this handoff. The implementation
has not started.

The first implementation feature branch should use OpenSpec for the backend,
PostgreSQL, workspace bootstrap, health checks, local Compose path, and separate
backend/frontend Dockerfiles. Use a specialist writer and independent reviewer,
then have a tester validate the implementation and runtime evidence. Keep the
task status and next action current here so work can resume from the pushed base.

## Completed implementation: development foundation

Status: **implementation, integrated verification, independent reviewer/tester
audits, spec sync, and archive complete** (2026-11-05). The pushed base is
`origin/rewrite-base` at `b765807`. Work is isolated on `feature/dev-foundation`.

- OpenSpec change: archived at
  `openspec/changes/archive/2026-11-05-bootstrap-rust-postgres-dev-foundation/`
- Synced main specs: `openspec/specs/backend-runtime/spec.md` and
  `openspec/specs/container-deployment/spec.md`.
- Latest feature checkpoint: `feature/dev-foundation` is pushed to GitHub;
  implementation commit `4887fee` is followed by OpenSpec/handoff progress
  commits. The final integration test, spec sync, archive, and handoff edits are
  currently uncommitted.
- Planning artifacts: proposal, `backend-runtime` and `container-deployment`
  specs, design, and 11 completed implementation/evidence tasks are preserved
  in the archived change directory above.
- Plan review: independent backend/deployment reviewer passed; an additional
  task-level requirement for bounded startup/migration-lock, readiness,
  migrate-only, and shutdown-lifetime coverage was strictly validated and
  approved by the reviewer. The reviewer has now passed the integrated backend
  and frontend/container code review. The independent tester passed the latest
  offline and static checks.
- OpenSpec tracked progress: 11/11 tasks complete and archived. Backend, frontend, and Compose/proxy
  writers completed their assigned implementation. The reviewer passed backend
  and frontend/container implementation. The tester passed OpenSpec strict
  validation, frontend build/runtime-config checks, static Compose validation,
  Rust formatting/checking/test compilation, and all 13 library tests.
- Current source state: backend modules/migrations, the Vue/Vite shell and
  frontend image scripts, root Compose/proxy files, and setup documentation are
  present. Do not mark task checkboxes complete until their recorded checks
  pass.
- Local verification (2026-11-05): `cargo fmt --check`,
  `cargo check --all-targets --offline`,
  `cargo test --all-targets --no-run --offline`, and
  `cargo test --offline --lib -- --include-ignored` pass (13/13). `npm run
  build`, frontend runtime-config validation, and `openspec validate --all
  --strict` pass. PostgreSQL integration tests compile and run against live
  PostgreSQL below.
- Live runtime evidence (2026-11-05): the Compose stack reached healthy
  PostgreSQL/API status and exposed only `127.0.0.1:4173`. The PostgreSQL
  integration executable ran inside the API container: 7/7 passed, covering
  clean ordered migrations, repeat/concurrent workspace bootstrap, preservation
  of category edits, upgrade from the prior migration, held migration-lock
  deadline, readiness, and actual Origin middleware. The loader printed a
  non-fatal weak `GLIBC_2.39` version warning before the tests; the test process
  exited successfully. `--migrate-only` exited successfully while the normal
  API already occupied its configured port 8080, confirming migration-only did
  not bind a listener. Direct API POST checks returned 405 with canonical
  Origin and 403 with mismatched, missing, and forwarded-only Origin.

  Both production images built. The backend image was launched with `docker
  run` on a dedicated test network against a separate PostgreSQL container also
  launched with `docker run`; its `/health/ready` returned ready. The production
  frontend image was launched with `docker run` on the Compose private network;
  runtime config contained only `apiBaseUrl=/api`, proxied readiness returned
  ready, client routes returned 200 HTML, and `/api` plus unknown `/health`
  paths returned 404 rather than SPA content. Frontend runtime-config checks
  rejected injection-shaped values and confirmed secrets were not emitted.

  Through the development proxy, readiness returned ready; an invalid Host
  connection was closed; POST with canonical Origin reached the API (405),
  while mismatched and forwarded-only origins received 403; `/api` and
  `/health` remained outside SPA fallback; a client route returned 200 HTML;
  and HMR returned 101 Switching Protocols. PostgreSQL workspace/category
  counts stayed `1:15` across a container restart. Held-request server tests
  passed graceful drain, refusal of new requests, continued server lifetime
  beyond the shutdown timeout before signal, and forced exit at the deadline.
  While PostgreSQL was stopped,
  readiness returned HTTP 503 with `not_ready`; after restart it returned
  `ready`. `docker compose down` followed by `docker compose up --build -d`
  completed, the stack became healthy again, and persisted data remained. The
  only published port remained loopback `127.0.0.1:4173`. The feature stops at
  infrastructure; financial API/browser workflows remain later vertical-slice
  work. The independent reviewer and tester passed all 11/11 tasks and the
  verification record.
- Earlier environment note: Cargo, Node, npm, and Docker Compose are installed.
  The interactive sandbox lacked Docker Engine socket access, including its
  prior escalation attempt. The owner later explicitly authorized sudo Docker
  interaction and supplied live Compose output. A prolonged escalated Cargo
  dependency request was aborted by the user; do not retry network dependency
  resolution without a new request.
- Next: begin a separate vertical-slice change for financial API/browser
  workflows and keep the remaining first-slice checklist evidence in that scope.

Resume with `git switch feature/dev-foundation` and `openspec list --json` to
select or create the next implementation change.
