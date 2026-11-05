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
