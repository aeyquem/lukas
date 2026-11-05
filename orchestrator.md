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

## Active implementation: development foundation

Status: **implementation is underway; backend lifecycle fixes and integrated
verification remain** (2026-11-05). The pushed base is `origin/rewrite-base` at
`b765807`. Work is isolated on `feature/dev-foundation`.

- OpenSpec change: `bootstrap-rust-postgres-dev-foundation`
- Planning artifacts: proposal, `backend-runtime` and `container-deployment`
  specs, design, and 11 implementation/evidence tasks are under
  `openspec/changes/bootstrap-rust-postgres-dev-foundation/`.
- Plan review: independent backend/deployment reviewer passed; an additional
  task-level requirement for bounded startup/migration-lock, readiness,
  migrate-only, and shutdown-lifetime coverage was strictly validated and
  approved by the reviewer. The reviewer has now passed the integrated backend
  and frontend/container code review. The independent tester is rerunning the
  latest integrated checks.
- OpenSpec tracked progress remains 0/11 tasks until each complete task's
  required evidence is available. Backend, frontend, and Compose/proxy writers
  have completed their assigned implementation. The reviewer passed the
  backend and frontend/container implementation. Tester verification
  previously passed frontend/build, static Compose, Rust
  formatting/checking/compilation, and ten library tests; the tester is
  repeating the suite after the latest backend coverage was added.
- Current source state: backend modules/migrations, the Vue/Vite shell and
  frontend image scripts, root Compose/proxy files, and setup documentation are
  present. Do not mark task checkboxes complete until their recorded checks
  pass.
- Checks to date: OpenSpec strict validation passes. Compose configuration
  resolves with only
  `127.0.0.1:4173` published; runtime-config safety script and production
  frontend build pass; `cargo fmt --check`, `cargo check --all-targets --offline`,
  `cargo test --all-targets --no-run --offline`, and the library suite pass (10
  tests, including shutdown tests, with ignored tests explicitly included).
  PostgreSQL integration tests compile but have not run against a live server.
- Environment note: Cargo, Node, npm, and Docker Compose are installed. Docker
  Engine access is denied for
  `/var/run/docker.sock`, including elevated execution. A prolonged escalated
  Cargo dependency request was aborted by the user. Do not retry Docker access
  or network dependency resolution without a new request.
- Next: collect the tester's final report, then mark only tasks whose complete
  evidence passes. Real PostgreSQL,
  Docker image, Compose health, HMR/browser, and integrated runtime evidence
  remain unavailable until Docker Engine/PostgreSQL access changes. Keep
  financial API/browser workflows and remaining first-slice checklist evidence
  for their later vertical-slice branches.

Resume commands: `git switch feature/dev-foundation`; inspect
`openspec status --change bootstrap-rust-postgres-dev-foundation --json`; then
continue with the OpenSpec apply workflow.
