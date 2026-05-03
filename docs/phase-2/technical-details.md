# Technical details to discuss

This document captures the main technical decisions that should be discussed
before turning the use-case specification into implementation work. The goal is
to make the engineering tradeoffs explicit: what needs a decision, why it
matters, and what topics should be covered in the discussion.

## 1. Product shape and application architecture

What needs to be discussed:

- Whether Presupuesto starts as a web app, mobile app, desktop app, CLI-assisted
  tool or a combination.
- Whether the architecture should be monolith-first, modular monolith,
  serverless-first or service-oriented.
- Whether the core financial domain should live in a framework-independent
  package that can be reused by API, UI, jobs and future MCP tooling.
- How much of the product should work offline or local-first.

Why it matters:

The product has financial state, projections, sync jobs, reports and future API
access. A thin CRUD app will not be enough, but over-splitting too early would
slow down the PoC. The first architecture should keep the domain model isolated
while still being simple to build and deploy.

## 2. Programming language and runtime

What needs to be discussed:

- Primary language for backend/domain logic.
- Primary language and framework for frontend.
- Whether to use one language across the stack, such as TypeScript, or split
  backend and frontend languages.
- Runtime requirements for background jobs, integrations, report generation and
  future MCP/API usage.
- Package manager, build system and repository structure.

Why it matters:

Currency handling, period calculations, audit history and sync behavior will
benefit from strong typing and reusable domain code. The language choice affects
developer speed, validation strategy, testing, deployment targets and the ability
to share types between UI, API and jobs.

## 3. Frontend framework and user experience platform

What needs to be discussed:

- Framework choice: for example Next.js, Remix, SvelteKit, Vue/Nuxt or another
  option.
- Client-side state management for transaction review, duplicate resolution,
  scenario planning and reports.
- Data visualization approach for reports, cashflow, runway and net worth.
- Form handling and validation for complex financial entities.
- Accessibility, responsiveness and localization needs.

Why it matters:

The product will have dense operational screens, not only static pages. Users
need to review transactions, resolve ambiguity, compare scenarios and inspect
reports. The frontend stack should support fast iteration without making complex
state flows fragile.

## 4. Backend framework and API design

What needs to be discussed:

- Backend framework: for example Next.js API routes/server actions, NestJS,
  Fastify, Hono, Django, Rails, Laravel or another option.
- API style: REST, GraphQL, tRPC, RPC-style commands or a hybrid.
- Command/query separation for write operations that need auditability.
- Versioning strategy for future public API and MCP access.
- Error model and validation boundaries.

Why it matters:

The backend must support normal app flows, background syncs, audit trails,
reports and eventually external tooling. Write operations need clear command
semantics because financial mutations must be traceable, reversible or at least
explainable.

## 5. Database and persistence model

What needs to be discussed:

- Database choice: PostgreSQL, SQLite/local-first, document database or hybrid.
- Whether to use Supabase, another BaaS, managed Postgres or self-managed
  infrastructure.
- ORM/query layer: Prisma, Drizzle, SQLAlchemy, Ecto, ActiveRecord, raw SQL or
  another option.
- Data model for accounts, transactions, source records, conversions, goals,
  allocations, periods, reports and audit history.
- How to store raw provider payloads and snapshots without coupling the domain
  model to provider schemas.
- Migration strategy and seed/test data strategy.

Why it matters:

The hardest product behavior is identity, history and reconciliation. The
database must preserve source records, user decisions, tombstones, period
assignments and multiple conversion values without turning every report into
unmaintainable logic.

## 6. Identity, auth and workspace model

What needs to be discussed:

- Authentication provider: Supabase Auth, Auth.js, Clerk, custom auth or another
  service.
- Workspace model: single-user first, household/workspace from the start, or a
  migration path from one to the other.
- Authorization rules for accounts, reports, integrations, API tokens and future
  MCP access.
- Session management and account recovery.

Why it matters:

Financial data is sensitive. Even if v1 is single-user, the model should not
block household/workspace sharing, scoped external access or per-integration
permissions later.

## 7. Security, secrets and compliance posture

What needs to be discussed:

- Where integration credentials, tokens and secrets are stored.
- Encryption at rest and application-level encryption needs.
- Whether bank/provider tokens are ever exposed to external tools.
- Audit logging for user edits, imports, external writes and admin actions.
- Data retention and deletion policy.
- Backup and recovery expectations.
- Legal/compliance posture for financial data, Open Finance integrations and
  Chile tax-support features.

Why it matters:

The app will process bank data, credentials, personal financial history and
possibly tax-related information. Security cannot be bolted on after the data
model and integration layer are already designed.

## 8. Integration architecture

What needs to be discussed:

- Adapter interface for CSV imports, manual imports, Splitwise, Fintual,
  aggregators, Open Finance Chile and future providers.
- Sync model: pull jobs, webhooks, manual refresh, scheduled refresh or hybrid.
- Source identity, external ID history, tombstones and provider mutation audits.
- Retry, backoff and error states.
- How imported records move through normalization, deduplication, review and
  final ledger state.

Why it matters:

Integrations are central to the product, but provider data is inconsistent.
Adapters should isolate provider-specific behavior while preserving enough raw
history to debug and audit sync results.

## 9. Background jobs and scheduling

What needs to be discussed:

- Job runner: hosted queues, serverless cron, database-backed queue, worker
  process or managed service.
- Jobs for sync, report precomputation, recurring estimates, period close,
  exchange-rate refresh, stale-balance checks and notifications.
- Idempotency strategy for sync and import jobs.
- Retry policy and dead-letter handling.

Why it matters:

Many important operations happen outside a user request. Syncs and projections
must be repeatable and safe; duplicate imports or partial job failures can
corrupt financial history if idempotency is not designed early.

## 10. Currency, decimal and date handling

What needs to be discussed:

- Decimal representation for money and exchange rates.
- Currency library and ISO currency handling.
- Timezone policy for users, institutions and reports.
- Date model for transaction date, authorization date, statement date, due date,
  payment date, budget period and tax period.
- Holiday/business-day source for Chile and optional future regions.

Why it matters:

Money and dates are core correctness risks. Floating point math, implicit
timezone conversion or a single transaction date would break reports,
projections and reconciliation.

## 11. Reporting and calculation engine

What needs to be discussed:

- Whether reports are calculated on demand, cached, materialized or a mix.
- Shared calculation primitives for spending, cashflow, net worth, goals,
  survival scenarios and health reports.
- How reports represent actual, pending, projected, adjusted and reconciled
  values.
- How to preserve report stability for locked periods.
- Export formats and reproducibility.

Why it matters:

Reports are where users judge whether the product is trustworthy. The reporting
engine must explain its inputs, statuses and assumptions, especially when data is
pending, estimated or manually adjusted.

## 12. Scenario and projection engine

What needs to be discussed:

- How projections are represented: derived views, persisted scenario entities or
  both.
- How scenario assumptions are stored and compared to the baseline.
- How survival scenarios model income loss, severance estimates, liquidity,
  debt payment strategy, reserves and account exhaustion.
- Whether calculations need deterministic snapshots for later review.

Why it matters:

Scenarios should not mutate real history, but they need to reuse real balances,
goals, allocations and recurring transactions. A clear scenario model avoids
mixing hypothetical decisions with the ledger.

## 13. API and MCP strategy

What needs to be discussed:

- Whether v1 exposes no external access, read-only API, private API, MCP server
  or both API and MCP.
- Authentication and scoped permissions for external tools.
- Audit and rollback strategy for external writes.
- Rate limits, revocation and secret isolation.
- Which resources and reports are safe to expose first.

Why it matters:

External access is useful for automation and future agents, but it increases the
security surface. The model should be designed now even if implementation comes
later.

## 14. Cloud services and deployment

What needs to be discussed:

- Hosting provider: Vercel, Fly.io, Render, AWS, GCP, Azure, Supabase platform or
  another option.
- Database hosting and backups.
- Object storage for imports, provider payload snapshots, exports and documents.
- Queue/worker hosting.
- Environment strategy for local, staging and production.
- Cost expectations for a personal finance product.

Why it matters:

Deployment choices affect architecture, background jobs, database access,
secrets, observability and cost. The PoC should be cheap and simple, but not
paint the project into a corner.

## 15. Testing and quality strategy

What needs to be discussed:

- Unit tests for domain calculations.
- Integration tests for imports, deduplication, sync and period rules.
- End-to-end tests for critical user flows.
- Snapshot/golden tests for reports and scenarios.
- Property-based tests for money/date edge cases where useful.
- Test data fixtures for Chile-specific cases.

Why it matters:

Most defects in this product would be subtle: wrong period, wrong conversion,
double-counted transfer, overwritten user edit. The test strategy should focus
on financial correctness, not just UI rendering.

## 16. Observability and operations

What needs to be discussed:

- Logging, metrics and tracing needs.
- Sync job monitoring and user-visible sync status.
- Error reporting and alerting.
- Audit log queryability.
- Admin/support tooling boundaries.

Why it matters:

When financial data looks wrong, the system needs to explain what happened.
Operational visibility is part of product trust, especially for imports and
background jobs.

## 17. Internationalization and Chile-specific rules

What needs to be discussed:

- Default locale, currency and formatting.
- Chile-specific holiday, tax-support, Open Finance and severance assumptions.
- Whether country-specific rules live in configuration, plugins or code modules.
- How to expand beyond Chile without rewriting core models.

Why it matters:

The product is Chile-centered, but many core concepts are general. Keeping
country-specific behavior modular makes the app easier to maintain and test.

## 18. Development workflow and repository conventions

What needs to be discussed:

- Monorepo vs single app repository.
- Code organization for domain, API, UI, integrations, jobs and shared types.
- Formatting, linting, type checking and test commands.
- CI strategy.
- Documentation and decision-record format.

Why it matters:

Clear conventions reduce friction once implementation begins. The project will
touch several domains, so repository structure should make ownership and
boundaries obvious.

## Suggested discussion order

1. Product shape and architecture.
2. Language, runtime and framework choices.
3. Database, persistence and domain model.
4. Auth, workspace and security posture.
5. Integration architecture and background jobs.
6. Money/date correctness rules.
7. Reporting and scenario engines.
8. API/MCP and external access.
9. Deployment, observability and testing.

This order starts with decisions that constrain the rest of the system, then
moves toward implementation details and operational concerns.
