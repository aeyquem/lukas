# Backend architecture: first slice

Status: **Selected backend architecture**, November 2026, under the owner's
delegation of backend technical choices. The owner prioritizes code that a
Rust beginner can read and change. Accepted decisions remain binding; the
selected stack and boundaries passed independent review and need
implementation validation. First-slice contracts that this document previously
marked as proposals or unresolved are decided in
`desc/13-first-slice-decisions.md`; that record controls any discrepancy.
Product scope is defined by [the first-slice plan](08-first-slice-plan.md), and
[rewrite context](00-rewrite-context.md) and accepted product decisions take
precedence. Later product and security policy remains open as noted below.

## Decisions and recommendation

Already accepted: one Rust modular monolith and one crate initially, with
domain, application, persistence and HTTP responsibilities; PostgreSQL, SQLx,
explicit SQL and versioned migrations; and `rust_decimal::Decimal` with a
96-bit mantissa and scale 0–28, stored in PostgreSQL `NUMERIC`. Reject values
and operations outside that envelope without silently rounding or truncating.
Self-hosted backend startup applies migrations with SQLx locking enabled;
hosted deployment may run the same migrator as a separate release step. The
first instance is owner-only, with a workspace boundary; SQLite is deferred.
Separate backend and frontend images and development Compose are accepted;
production does not require Compose.

**Selected stack:** use Axum and Tokio for HTTP and asynchronous I/O, Serde
and `serde_json` for transport, SQLx for PostgreSQL, `rust_decimal` for
calculations, `chrono` for date and UTC timestamp types, and `tracing` plus
`tracing-subscriber` for structured logs. Use `thiserror` for named error enums
and ordinary `Result` propagation. Start with the smallest features needed;
commit `Cargo.lock` for reproducible application builds. Pin and review exact
versions during implementation, rather than treating today's latest releases
as a permanent requirement.

Axum offers straightforward routes, JSON extraction and shared application
state; Tokio supplies the async runtime SQLx and Axum need. This combination is
conventional but introduces async and extractor concepts for a Rust beginner.
Keep those at the edges: ordinary synchronous functions handle calculations,
with explicit async calls only for I/O. Do not add Tower middleware, trait
objects or generic frameworks by default. Axum's [extractor](https://docs.rs/axum/latest/axum/extract/index.html)
and [state](https://docs.rs/axum/latest/axum/extract/struct.State.html)
documentation show the underlying model. If later work performs substantial
blocking parsing, isolate and bound it; Tokio explains why blocking work needs
special handling in its [task documentation](https://docs.rs/tokio/latest/tokio/task/).

Use no ORM, dependency-injection container, general event bus, event sourcing,
job framework, API generation pipeline or extra workspace crates initially.
Explicit SQL and direct function calls keep the path from request to stored
fact visible. Revisit these choices when a real use case earns their cost.
The CLI remains a later slice; do not assume its transport or credentials.

## Organizing the code

One possible starting layout, expanded only when a file gets hard to read:

```text
backend/
  Cargo.toml
  Cargo.lock
  Dockerfile
  migrations/                 # ordered, immutable SQL migrations
  src/
    main.rs                   # configuration, startup and shutdown
    lib.rs                    # names the modules
    config.rs
    domain/
      mod.rs
      money.rs                # amount, currency and checked operations
      account.rs
      ledger.rs
    application/
      mod.rs
      accounts.rs             # named use cases
      ledger.rs
      reports.rs
    persistence/
      mod.rs
      postgres.rs             # pool and migration setup
      accounts.rs             # explicit SQL and row mapping
      ledger.rs
      reports.rs
    http/
      mod.rs                  # router and shared state
      accounts.rs             # request/response types and handlers
      ledger.rs
      reports.rs
      error.rs
  tests/                      # public behavior and PostgreSQL integration
```

This tree is illustrative, not a demand to create empty files or speculative
future features. A domain type names a financial concept and enforces its
rules; it does not import Axum, SQLx, SQL rows or HTTP status codes. Transport
and database rows are separate structs mapped explicitly at their boundaries.
Application functions coordinate named use cases and decide the atomic unit of
work; persistence functions execute parameterized SQL and translate rows.
Handlers parse the request, call a use case, and map its result to a response.
Use private modules and ordinary structs/functions. Introduce a small trait
only when a real alternative or test needs it, not one repository trait and
mock for every table. An application use case may call concrete PostgreSQL
persistence functions and pass a transaction; that dependency is explicit and
is not a claim of database independence. Financial decisions stay in domain
code, so changing persistence later does not redefine them.

## A write, from screen to database

For an audited correction in a later slice (first-slice edits overwrite the
current value without retaining value history):

1. The HTTP handler authenticates/obtains the authorized workspace through the
   eventual access boundary, parses a request and checks its shape. It passes
   the workspace, requested values and expected revision to a named use case.
2. The use case starts one database transaction and loads the workspace-scoped
   current record with the necessary lock or conditional revision check. It
   invokes domain rules to validate the correction. Preserve the actual date of
   the financial event and retain the original recorded date, source, previous
   interpretation and correction decision in history. Never replace the event
   date with the correction date. How a mistakenly recorded event date can be
   corrected remains a product contract to specify; the history must still
   show the prior recorded value and decision. This does not decide same-day
   opening ordering.
3. Persistence writes the new interpretation and its linked correction decision
   and provenance in that same transaction. The use case commits only after
   every required write succeeds. Otherwise all changes roll back.
4. The handler returns the saved representation and new revision. A stale
   revision is a conflict the caller must review; never silently overwrite it.

This is an example, not a finalized schema or endpoint. Transfers similarly
commit their linked effects together, not as independently visible half-writes.
Opening balances are explicit adjustment transactions with provenance, not
unexplained mutable account fields.
Reconciliation adjustments remain distinct from spending. Reports derive
inspectable totals from confirmed facts, preserving currency and period basis;
do not combine unlike currencies or invent a CLP estimate for the first slice.
For a transaction dated before a wallet's opening date, require the accepted
per-wallet apply-effect or preserve-current-balance choice. If the transaction
shares the opening date and inclusion is unknown, require that same initially
unset choice per affected wallet; do not infer chronology from dates, IDs or
record order. Correction of a mistaken transaction date remains separate.

## Transactions, concurrency and persistence

Begin and commit transactions at the use-case boundary, not independently
inside each repository function. All records for one decision, including audit
and provenance, must succeed together. Scope every query by the authorized
workspace ID. Use composite foreign keys and other database constraints to
prevent cross-workspace references and enforce required relationships and
uniqueness. The owner-only model does not remove isolation requirements.

Recommend an explicit revision per editable financial record and a conditional
update (`WHERE workspace_id = ... AND id = ... AND revision = ...`), checking
affected rows. For decisions involving multiple records or balance-dependent
invariants, acquire relevant row locks in a consistent order and recheck the
conditions within the transaction. An absent row needs a uniqueness constraint
or another suitable serialization mechanism; row locking cannot lock a row
that does not exist. Do not assume a transaction alone prevents races:
PostgreSQL's default Read Committed takes a fresh snapshot per statement
([documentation](https://www.postgresql.org/docs/current/transaction-iso.html)).
Define isolation/locking per use case and test competing corrections and
transfer writes. For multi-query reports requiring a coherent view, use a
single query or an appropriate consistent snapshot.

A timeout after a write has an unknown outcome. The owner accepted this
first-slice idempotency contract:

- Every mutating request carries a client-generated UUID idempotency key. The
  client creates it once per user intent and keeps it with the exact request
  bytes until the result is known. An explicit retry resends the same key and
  bytes; a new user intent gets a new key.
- In the same PostgreSQL transaction as the financial change, persist the
  workspace ID, key, method, route, request-body hash, and small original
  response result (status and resource ID/revision where relevant). A unique
  constraint on `(workspace_id, key)` serializes concurrent attempts.
- A repeated key with the same method, route and body hash returns the saved
  result without running the mutation again. A repeated key with a different
  request returns `409 idempotency_key_reused`. The UI then refetches the
  affected resource. No separate status endpoint is needed: retrying the exact
  command reports whether it committed.
- Retain these small command records for the lifetime of the workspace so a
  delayed retry cannot duplicate an old write. They do not retain prior
  financial field values or provide edit history. Removing a workspace removes
  its idempotency records.
- Do not automatically retry a timed-out write. The UI's explicit Retry action
  reuses the pending key and exact request. When the outcome is unknown,
  Cancel only dismisses the alert; it cannot cancel a committed write or clear
  the unresolved command. Keep that command available for an exact retry and do
  not silently issue the same user intent with a new key. For a definite
  rejection, Cancel may discard the draft.

Database deadlock/serialization failures use the bounded retry policy and
per-use-case API/error contract in `desc/13-first-slice-decisions.md`.

Use SQLx runtime `query`/`query_as` calls with explicit row structs and
parameter binding for the first slice; do not assemble SQL from user input.
This keeps the query and row mapping visible without a build-time database or
checked-in macro metadata. PostgreSQL integration checks must exercise every
query. If a representative query later shows that compile-time checking is
worth its database/metadata workflow, adopt SQLx checked macros and check their
metadata in CI. See the [SQLx query documentation](https://docs.rs/sqlx/latest/sqlx/macro.query.html).

## Money, dates and API boundaries

Use decimal strings such as `"1250.50"` with an explicit currency code in
JSON, and reject JSON numbers for money and rates. Validate the literal with an
exact parse and verify representability before constructing a domain amount;
do not rely on a parser that may round an over-precise input. The accepted
Decimal envelope (scale 0–28) is the only input/storage scale cap; preserve
exact values and do not add currency-specific scale limits. Checked arithmetic
is necessary but not by itself proof of exactness for every operation: detect any precision loss,
and define division and rounding before allowing operations that require them.
Never use float conversion, saturation or implicit database scale rounding for
financial facts. PostgreSQL `NUMERIC` can represent values outside the Decimal
envelope, as well as `NaN` and infinities, none of which Decimal can represent.
Reject non-finite and out-of-envelope inputs and use schema constraints to
protect stored financial values; validate both writes and reads, including
aggregates. If a stored value or computed aggregate cannot be represented
exactly, fail the operation or report with a clear error and sanitized
diagnostic; do not substitute zero, emit a partial total, round, or silently
discard the offending value. Treat an invalid stored value as an integrity
problem that needs investigation. Avoid a
`NUMERIC(p,s)` declaration that silently rounds a source value to its scale;
use explicit validation and constraints suitable for the chosen schema. The
[Decimal documentation](https://docs.rs/rust_decimal/latest/rust_decimal/)
describes its bounds and [checked operations](https://docs.rs/rust_decimal/latest/rust_decimal/struct.Decimal.html);
[PostgreSQL numeric types](https://www.postgresql.org/docs/current/datatype-numeric.html)
describe database precision and coercion. Preserve source values and their
currency; raw import precision outside the envelope remains an open later-slice
question.

Represent a business date as `YYYY-MM-DD` and PostgreSQL `DATE`, never as an
invented midnight instant. Store audit instants in UTC, and preserve known
source time and offset/timezone context separately from the business date.
For a current-month report, use the workspace timezone to determine today's
date and select its calendar month, then compare business `DATE` values directly
against the resulting date boundaries. Return the selected period/basis
explicitly. Do not shift a
recorded date based on the viewer's timezone; do not invent time or ordering
where none is known. Exact API formatting, timezone handling and same-day
ordering require contract examples and tests.

Use explicit request/response structs and a small error envelope, for example
`{"error":{"code":"stale_revision","message":"...","fields":[]},"request_id":"..."}`.
Recommend 400 for malformed input, 422 for invalid financial input, 404 for
an unavailable scoped resource, 409 for a stale revision, and 500 for an
unexpected failure. Authentication status and details depend on the eventual
security design. Give clients stable machine-readable codes and useful field
errors; log internal causes without returning SQL, credentials or stack traces.
The reviewed examples below propose routes and pagination; they are not yet
accepted or validated. Serialize and validate them end to end before selecting
OpenAPI generation.

### First-slice money/date wire profile

Use `desc/13-first-slice-decisions.md` as the selected contract where the
following draft labels or text still say “open” or “for owner review.”

The representation decisions are settled: JSON money and rates are decimal
strings paired with explicit currency codes, business dates use `YYYY-MM-DD`,
and the browser must preserve decimal text rather than using JavaScript
`Number` for financial calculations. Backend values use the accepted Decimal
envelope and checked operations; PostgreSQL storage and reads must not silently
coerce values. Business dates are not shifted by the viewer timezone, source
time/offset stays separate, audit instants use UTC, and current-month
boundaries use `America/Santiago`.

The following boundary details include accepted rules plus items that remain
proposals or owner choices:

- **Literal syntax — ACCEPTED:** accept plain ASCII decimal strings with
  digits, an optional leading minus and decimal point; reject exponent
  notation, grouping separators, whitespace, NaN and infinity. Parse and
  validate the literal before creating a Decimal. The Decimal envelope (scale
  0–28) is the only input/storage scale cap; never silently round to fit.
- **Field sign rules — ACCEPTED:** opening balances and foreign-liability CLP
  bases are signed. Transaction requests use positive magnitudes plus a
  required kind, and the backend derives signed ledger effects. Transfer sides
  use positive magnitudes. Category assignments are nonnegative; category
  remaining may be negative. The Decimal envelope (scale 0–28) is the only
  input/storage scale cap; preserve exact supplied values within it.
- **Overflow behavior:** use checked arithmetic and exact persistence/read
  validation against the Decimal envelope. Reject overflow or precision loss
  with a stable field or operation error; do not return a partial financial
  total. Implement finite, scale and maximum-magnitude PostgreSQL checks and
  stable aggregate errors per `desc/13-first-slice-decisions.md`.
- **Dates:** preserve date-only business values. Optional source time and
  offset representation remain open. For a manual transaction on its wallet's
  opening date with unknown inclusion, the accepted per-wallet apply-effect or
  preserve-current-balance choice determines treatment; do not infer ordering
  from the date. A transaction date correction remains subject to the edit
  contract.
- **Derived CLP foreign-asset valuation rounding — ACCEPTED:** multiply the
  exact foreign asset amount by the selected exact rate, then round this
  derived CLP valuation once to a whole peso, nearest with midpoint away from
  zero. This rule does not set rounding for foreign liabilities, actual
  conversions or allocations. Division policy can remain deferred for
  operations implemented without division.
  Before introducing any first-slice division, define its precision, rounding
  and failure behavior.
- **HTTP contract:** retain the proposed request IDs, stable error codes,
  field errors and status mapping above; finalize route/request/response shapes
  alongside each vertical slice and validate serialization end to end. These
  examples do not authorize generated-client/OpenAPI work.

## Startup, migration and operations

The proposed local/runtime contract is:

- Read typed environment configuration once. Require `DATABASE_URL`, API bind
  address, canonical public origin, pool size, connection/acquire timeouts and
  shutdown timeout. Keep the database URL out of source, image layers and
  logs. Local Compose reads dev credentials from an ignored local env file;
  production secret delivery remains a release/operator decision.
- Start with a five-connection SQLx pool, a five-second pool-acquire timeout
  and a separate five-second connection timeout. These are proposed bounded
  defaults to validate and tune during implementation; pool acquisition and
  connection establishment are distinct waits. Bound startup configuration,
  database and migration-lock waits, failing startup rather than serving before
  setup completes.
- Startup order: validate configuration; connect to PostgreSQL; apply pending
  migrations under the SQLx migration lock; create/resolve the singleton
  workspace and starter categories atomically; then report readiness. A
  liveness endpoint reports process health. A readiness endpoint reports not
  ready until initialization succeeds, and returns not-ready if PostgreSQL
  disconnects. FX lookup has a bounded request timeout and does not block API
  readiness; an outage degrades valuation only. Refresh at startup and daily.
- On SIGTERM, mark readiness false, stop accepting new business requests,
  drain bounded in-flight work within the configured deadline, then close the
  pool.
- Keep immutable SQL migrations at
  `backend/migrations/<VERSION>_<DESCRIPTION>.sql`. Embed the same migrator in
  the backend image. Self-hosted startup applies pending migrations before
  readiness; the same binary supports a hosted `--migrate-only` release step.
  Never edit an applied migration; add a new version instead.

For local Compose, use services `postgres`, `api`, `vite` and `dev-proxy`.
`postgres` has a `pg_isready` health check and a named data volume, but no host
port. `api` connects to `postgres:5432`, listens on private container port 8080,
waits for PostgreSQL health and has no host port. `vite` listens on private
port 5173, mounts application source without obscuring container dependencies,
and proxies `/api` to `api:8080` while preserving the path and browser Origin.
Only `dev-proxy` publishes a host port: bind `127.0.0.1:4173` to the proxy,
which accepts exactly Host `localhost:4173` and forwards HTTP and HMR WebSocket
traffic to Vite. Browser writes use exact Origin `http://localhost:4173`; no
broad CORS is enabled. The database URL and local
PostgreSQL credentials are supplied through ignored development configuration.

For production, build separate backend and frontend images. The backend image
contains the Rust service and embedded migrator. The frontend image contains
static Vue assets and a static server/reverse proxy: serve `/api` to the
configured backend upstream, serve client routes with SPA fallback, and do
not apply SPA fallback to API requests. Supply `API_UPSTREAM` to the static
server at container startup. Generate a non-secret `/runtime-config.js` from
the image template at startup with the public `API_BASE_URL`, and load it as a
script before the Vue app entry point so the same static image can target
different backend locations. Use `/api` for the initial same-origin setup.
Neither image requires Compose. Exact image versions and production rollout
compatibility remain release decisions.

Validate startup failure/readiness, fresh and subsequent migrations, bootstrap
concurrency, restart persistence, database outage/recovery, bounded shutdown,
browser→proxy→API→PostgreSQL connectivity, exact Host/Origin checks, `/api`
routing, SPA fallback, HMR WebSockets, source mounts and dependency refresh
during implementation. This validation is not a reason to design production
backup/restore or rolling-schema compatibility as a local code prerequisite.

Use structured `tracing` logs for startup, request ID, route, outcome, duration
and sanitized error context. Do not log passwords, tokens, connection strings,
raw statements, transaction descriptions or complete request bodies by default.
Application logs help diagnose operations; durable, inspectable correction
history is financial data stored transactionally, not a substitute log line.
Production PostgreSQL provisioning/storage, backup and restore rehearsal,
secret delivery, log retention, TLS/proxy guidance and schema rollout/rollback
remain self-hosted release gates.

## Security boundary and later decisions

Owner-only describes who may use the instance, not how access is secured. The
initial unauthenticated deployment is limited to the same machine: publish the
frontend on loopback, keep the backend on the private container network, and
route browser API calls through same-origin `/api`. Do not expose it through a
LAN-facing or public proxy. Wider access requires a separately reviewed
access-control design. Since requests do not identify a user, the server
resolves the sole owner workspace internally at startup and stores its ID in
shared application state. Every use case and persistence function receives
that ID explicitly. The API contract has no workspace selector, and no
caller-supplied workspace ID is accepted. A future authentication adapter can
supply an authorized workspace context without moving authorization into
product rules. Do not
invent member roles or settle Supabase Auth by implication.

For the initial local browser path, configure one canonical loopback UI origin
and use same-origin `/api` through the frontend container. The frontend accepts
only the configured loopback `Host`; the API disables CORS and requires the
matching `Origin` on state-changing browser requests. Do not trust forwarded
host/origin headers from outside the local proxy. Cross-origin browser access
and authenticated CSRF behavior belong to a later access-control review.

Before any wider deployment, decide bootstrap and recovery, TLS, browser
session/token behavior, workspace authorization, CLI credential strategy,
secret rotation, raw data retention/encryption and backup protection.

Implement and validate the first-slice API, revision, source-precision,
rounding, and persisted opening-choice contracts as specified in
`desc/13-first-slice-decisions.md`. Also define operational upgrade/restore
policy before self-hosted release. The first slice includes current-month
category assignments, transaction activity, and the accepted basic explicit
close transition. Imports, CLI commands, reports, reconciliation, advanced
multi-period rollover, goals and projections remain excluded.

## Validation before implementation expands

Exercise a vertical slice with fresh migrations and real PostgreSQL: dated
opening balances as explicit adjustment transactions with provenance, including
signed budgetable opening funding; manual income, expense and transfer
activity; and current-month category assignments with actual transaction
activity. Verify the backdated-transaction choice preserves the current balance
or applies the transaction effect as selected. Verify a card purchase affects
budget activity and its later payment is a transfer, not repeat spending.
Check exact decimal boundary round trips, non-finite and out-of-envelope
rejection and arithmetic/aggregate failure, original dates/currencies,
adjustment provenance, multi-record rollback, workspace isolation, concurrent
stale updates and retry safety. Once a second migration exists, verify
upgrading from an earlier schema as well as a fresh install. Include a
request/response contract example for decimal strings, dates and conflict
errors. Reports, reconciliation, and historical edit versions are outside this
slice. These are proposed checks, not tests already run.

## First-slice contract reference

The concrete first-slice selections are in
`desc/13-first-slice-decisions.md`. All material below this heading retains
some earlier proposal wording and is background only; any “owner review,”
“open,” or “unresolved” label below is superseded for the first slice by the
reviewed decision record. Later-scope or self-hosted release items remain open.

The earlier initialized-month contract and examples received scoped review.
The owner subsequently included the basic explicit close transition; its
selected command, accounting rules, and remaining first-slice edge cases are
now specified in `desc/13-first-slice-decisions.md`.

### Data and use-case boundaries

Use one singleton workspace. An account has name, type, base currency, budget
eligibility and revision. A wallet has an account and currency; enforce unique
`(workspace, account, currency)`. Each wallet has exactly one dated signed
opening entry. A transaction has its real date, kind, optional
category/payee/note, provenance and revision. Linked wallet effects represent
income, expense or one logical transfer. A transfer has exactly two effects
with matching wallet currencies and opposite signs; same-currency magnitudes are
equal. Retain both actual amounts for a cross-currency transfer. Validate
multi-row cardinality transactionally; use constraints or deferred enforcement
where needed so no partial account, wallet, opening or transfer can commit.

Keep current per-transaction, per-wallet opening offsets and their decisions.
Categories have revisions; assignments are unique on `(workspace, month,
category)`. Persist dated FX observations, including changed observations on the
same date, with source and effective date, and minimal idempotency receipts. Add
workspace-scoped composite foreign keys, uniqueness constraints and indexes for
wallets, transaction dates, categories/months, FX pairs/dates and receipts.
These are logical entities, not approved DDL.

Bootstrap or resolve the sole workspace and starter categories atomically,
guarded by a singleton constraint. Shared API state stores its ID; pass it to
every use case and repository function. The client never selects a workspace.
A later authenticated adapter can provide an authorized workspace context.
Use cases own database transactions; parameterized SQLx repository functions
receive the same transaction. Account plus openings, transfer effects, opening
offsets, assignment and associated command receipt commit or roll back
together. Multi-query budget reads use one coherent read-only snapshot.

For each wallet, `effective opening = baseline opening + sum(current offsets)`.
Show baseline, offset sum and effective amount. Editing the opening amount sends
the desired effective amount and expected opening revision; under lock, set
`baseline = desired effective - current offsets` and apply the effective change
to the balance once. An edited transaction first removes its old effects and
old offsets, then writes replacement effects and offsets. Increment affected
revisions. Editing opening date or wallet currency follows the first-slice
conditions in `desc/13-first-slice-decisions.md`; never silently reclassify
existing transactions. This stores current state, not prior-value history.

The reviewed budget calculation must also distinguish eligible opening funding
from current ledger balance. Track eligible opening basis and recognized income,
boundary movements and valuation changes as distinct funding components;
apply each once in its proper period. A prior-period transaction's balance
effect or compensating offset changes the opening basis only as appropriate to
that period. Ordinary current-period spending or newly incurred card debt must
not deduct the same funding again. The general component contract for
transfers, debt, and FX is resolved in `desc/13-first-slice-decisions.md`.
The following isolated CLP bank examples use initial opening 100 dated
November 4, current-month assignment 100, and one expense of 20. `F` means the
month's funding basis; activity and remaining are current-month category values.

| Expense and choice | Current bank balance | Effective opening | F | Ready-to-assign | Activity | Remaining |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| November before opening, apply | 80 | 100 | 100 | 0 | 20 | 80 |
| November before opening, offset | 100 | 120 | 120 | 20 | 20 | 80 |
| October before opening, apply | 80 | 100 | 80 | -20 | 0 | 100 |
| October before opening, offset | 100 | 120 | 100 | 0 | 0 | 100 |

An offset for prior-month activity cannot inflate current-month funding. These
are background examples for the selected complete funding derivation in
`desc/13-first-slice-decisions.md`, including card, foreign, and boundary cases.

### Prior funding derivation proposal (superseded by the decision record)

This earlier once-only calculation draft is superseded by the selected
derivation in `desc/13-first-slice-decisions.md`.

- The first calculation begins at zero. Add each eligible signed opening
  balance once to initial ready-to-assign. For an eligible foreign opening
  liability, require a separately supplied signed CLP funding basis and apply
  it once. Preserve the signed original-currency debt; do not use daily debt FX
  for further funding deductions. If the basis is missing, funding and
  ready-to-assign are unavailable. A direct correction to the foreign opening
  amount requires the user to supply or explicitly confirm its signed CLP basis
  in the same update; do not derive it from daily debt FX. Basis storage,
  provenance, and apply/offset correction effects are specified in
  `desc/13-first-slice-decisions.md`.
- From the latest closed checkpoint, carry forward ready-to-assign and add new
  funding effects once: recognized eligible income; signed effects of
  transfers crossing the budget boundary (tracking-only to budgetable adds,
  the reverse removes; transfers within either side have no ordinary boundary
  effect); actual conversion differences on cross-currency transfers between
  budgetable accounts (the principal has no boundary effect, but the independent
  difference, such as CLP 100,000 previously valued becoming CLP 95,000 at the
  destination, affects funding by CLP -5,000 once); and independent FX rate
  changes on eligible foreign assets after separating actual receipts,
  spending, transfers and conversions. Exclude any difference already recorded
  through a fee or another effect. Do not add an account balance again just
  because it remains held. Apply the
  accepted exception for payments of already-recognized debt (AC-5.11/5.19);
  do not apply another ordinary boundary deduction to those payments.
- A new category assignment reduces ready-to-assign once. Releasing an
  assignment returns it once; moving an assignment between categories changes
  neither total funding nor ready-to-assign. Ordinary expenses/refunds affect
  activity and remaining in their correctly resolved budget period; do not
  also debit/credit ready-to-assign. Preserve separately reviewed effects for
  prior-period activity and opening apply/offset adjustments, which can change
  the effective opening funding basis. A card purchase is spending once; its
  debt increase does not reduce funding again, and its later bill payment is
  not a second expense.
- At close, existing ready-to-assign carries forward. For each category, the
  user's choice either transfers its signed remainder to ready-to-assign or
  retains that signed balance in the same category. Returning a negative
  remainder reduces ready-to-assign; carrying it keeps the category deficit.
  The selected destinations redistribute the existing budget balance; they
  do not create or erase funding.
- While a month is open, calculate forward from the most recent explicit close
  and intervening activity. Until that month closes, provisionally carry each
  signed category remainder into the same category in the next month's
  calculation. At explicit close, the user may retain each carry or release it
  to ready-to-assign. Closing snapshots the selected result and replaces the
  earlier rolling checkpoint for later periods. Replay later activity from
  this new checkpoint without applying that activity twice or changing
  assignments already made in later periods.

Example: start with CLP 100 ready-to-assign. In November, assign CLP 80 and
spend CLP 30, leaving CLP 20 ready-to-assign and CLP 50 in the category.
Before November is closed, December's provisional result carries the CLP 50
remainder in the same category and has CLP 30 ready-to-assign after its CLP 10
eligible income. At November close, the user can keep that carry, making the
November checkpoint CLP 20 ready-to-assign plus CLP 50 in the category, or
release it, making the checkpoint CLP 70 ready-to-assign. Recalculate December
from the chosen checkpoint and replay December's income once: with carry,
December is CLP 30 ready-to-assign plus CLP 50 in the category; with release,
it is CLP 80 ready-to-assign. Do not count November's category remainder or
December's income twice, and do not change assignments already made in
December. Both choices preserve the same total. If a category instead has CLP -10
remaining and CLP 20 ready-to-assign, returning it yields CLP 10 ready-to-
assign; carrying it yields CLP 20 ready-to-assign and a CLP -10 category
balance.

For the FX distinction, a USD 100 eligible asset valued at CLP 900/USD
contributes CLP 90,000 at the initial valuation. Spending USD 10 with a
confirmed CLP value of CLP 9,000 reduces the wallet to USD 90 and affects its
assigned category; at the unchanged rate,
that balance decrease must not create a second CLP 9,000 funding deduction.
If the rate then changes from 900 to 950 CLP/USD while USD 90 remains, the
independent CLP 4,500 valuation increase is the funding effect. This example
assumes a confirmed CLP spending value; for an unvalued expense, preserve the
original ledger entry and mark dependent CLP values incomplete as described
below.

Open questions before this derivation can be accepted include how later-created
accounts and their opening balances enter a month and how a correction to a
closed-period fact changes subsequent calculations without rewriting the
closed snapshot. The accepted foreign-liability rule requires a signed CLP
basis applied once. A direct correction to its foreign opening amount must
include the user's supplied or explicitly confirmed signed CLP basis in the
same update; never derive a revised basis from daily debt FX. Basis storage
and provenance, and how apply/offset corrections affect it, remain to be
specified. For
incomplete foreign expenses, preserve the original entry, expose incomplete
dependent CLP values, and block close while required values remain incomplete.
Later eligibility changes and unusual source precision must follow their
separately reviewed rules rather than be inferred here.

### Prior edit and closed-period correction proposal (superseded by the decision record)

The first slice accepts in-place edits to account, transaction and budget
values, with no previous-value history or rollback. Revisions prevent stale
writes but do not retain the replaced values. This is narrower than the later
AC-2.7 inspection of previous details; that history remains outside the first
slice.

| Record/field | Earlier proposal | Selected first-slice decision (`desc/13`) |
| --- | --- | --- |
| Account name and descriptive fields | Replace current values in place. | Editable in place. |
| Account type, base currency, tracked wallet currency | Do not silently reinterpret existing ledger effects. | Editable only before the affected wallet has non-opening ledger activity. |
| Account budget eligibility | Editing eligibility is excluded from this slice. | Remains excluded. |
| Opening amount | Set desired effective amount under revision and lock; pair foreign-liability principal edits with signed CLP basis. | Apply signed delta once; retain separate basis provenance; pair principal-changing offsets atomically. |
| Opening date or wallet currency | Do not silently reclassify existing activity. | Editable only while the wallet has no transactions. |
| Transaction payee, note and category | In-place correction in an open period. | Editable in place, including clearing a category; closed-period budget changes use linked correction. |
| Transaction amount, account, currency, type and transfer destination | Replace linked effects atomically. | Editable with transfer validation and re-resolution of per-wallet opening treatment. |
| Transaction date | Preserve during ordinary correction. | Explicit date correction is allowed; atomically recalculate old/new effects and preserve closed snapshots. |
| Budget assignment amount/category | In-place edit while the budget is open. | Mutable while open; after close use a linked correction in the current open month. |

For any supported edit, check the expected revision and replace only the
financial effects affected by the edit. Metadata edits preserve ledger effects
and opening-treatment decisions; category edits change budget attribution
without reposting account movements. Replace wallet effects and opening offsets
only when their underlying financial facts change. Commit affected changes and
increment revisions atomically.

Resolve both the old and replacement budget effects. Recompute affected open
periods from the latest close. Preserve every affected closed snapshot and
route its correction effect to the applicable current period where possible;
do not also apply that effect through recalculation or checkpoint replay. A
proposed closed-period correction is a linked effect, committed atomically
with the edit and idempotency receipt; it does not create another wallet
effect. If the correction target period is open, update its existing linked
correction by the incremental difference, so its stored effect represents the
net difference from the closed snapshot. For example, correcting a closed-
period expense from CLP 1,000 to CLP 1,200 applies CLP 200 in the correction
period. A later edit to CLP 1,300 applies another CLP 100, leaving CLP 300
total. If that correction period has itself closed, preserve its snapshot and
apply the further CLP 100 in the applicable current period. If a permitted
edit reclassifies a closed-period CLP 1,000 expense from category A to B,
restore CLP 1,000 to A and record CLP 1,000 of use in B in the correction
period; the wallet effect is unchanged. If a permitted date/period edit moves
an expense between an open and closed period, preserve the closed snapshot and
route the old and replacement effects without double application.

The target period and open/closed transitions are selected in
`desc/13-first-slice-decisions.md`. A stale-write conflict retains the user's
draft for explicit review; an unknown outcome uses the same idempotency key
and exact request on retry. The field contract is selected there; the broad
account/transaction/budget edit scope and no-history rule remain in force.

### HTTP examples and incomplete values

Mutations carry one client UUID idempotency key per intent; edits include an
expected revision. JSON uses currency codes and exact decimal strings; a
business date is `YYYY-MM-DD`. One proposed account request is:

```http
POST /api/accounts
Idempotency-Key: 5c079e8e-9e43-4ac1-abf4-52c777830001
Content-Type: application/json

{"name":"Cuenta corriente","type":"bank","base_currency":"CLP","budget_eligible":true,"opening_balances":[{"currency":"CLP","date":"2026-11-01","amount":"100000"}]}
```

Return `201` and minimal metadata such as
`{"id":"aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa","revision":1}`. Then
`POST /api/transactions` can record an expense:

```json
{"type":"expense","date":"2026-11-04","account_id":"aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa","amount":{"currency":"CLP","value":"12000"},"category_id":"bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb","payee":null,"note":null,"opening_treatment":null}
```

The accepted transport uses a positive magnitude for income, expense and
transfer requests; the application derives signed effects. A transfer request
contains both account/currency/amount sides and any required opening choice.
For example, `POST /api/transfers` can carry:

```json
{"date":"2026-11-04","source":{"account_id":"aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa","currency":"CLP","amount":"5000","opening_treatment":null},"destination":{"account_id":"dddddddd-dddd-4ddd-8ddd-dddddddddddd","currency":"CLP","amount":"5000","opening_treatment":null},"note":null}
```

The response is a minimal logical transfer ID and revision; the client GETs its
current effects. Route naming and payload are recommendations.
For every affected wallet before its opening, require the accepted explicit
apply-effect or preserve-current choice. For same-day activity with unknown
inclusion, require the same initially unset per-wallet choice. Never infer
chronology from an ID or record order. Transfer choices are independent per
wallet.

For the example above, a subsequent account GET can return:

```json
{"id":"aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa","name":"Cuenta corriente","type":"bank","base_currency":"CLP","budget_eligible":true,"revision":1,"wallets":[{"currency":"CLP","opening":{"date":"2026-11-01","baseline_amount":"100000","offset_total":"0","effective_amount":"100000","revision":1},"balance":{"amount":"88000","currency":"CLP","revision":2}}]}
```

For an initialized current month, a proposed assignment is
`PUT /api/budgets/2026-11/categories/{category_id}/assignment` with
`{"amount":{"currency":"CLP","value":"20000"},"expected_revision":0}`.
Revision zero means no assignment exists. Return only category ID, period and
revision, never the financial amount in the receipt; the client GETs current
state. With the single CLP account and expense above, a budget GET can return:

```json
{"period":"2026-11","currency":"CLP","categories":[{"id":"bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb","assigned":{"value":"20000","currency":"CLP"},"activity":{"status":"complete","value":"12000","currency":"CLP"},"available":{"status":"complete","value":"8000","currency":"CLP"},"assignment_revision":1}],"ready_to_assign":{"status":"complete","value":"80000","currency":"CLP"},"warnings":[],"rates":[]}
```

This is a worked example, not a general formula for card, cross-boundary or
foreign cases. Under the accepted rule, unvalued foreign expenses retain their
original effect. Category activity and availability are incomplete, with known
CLP subtotals labelled partial and original pending amounts listed. Dependent
aggregate availability is incomplete. A confirmed CLP value completes the
same expense once. Ready-to-assign is complete only if its own funding inputs
are complete; warn separately about pending expense values. Assignments remain
savable intent, and close is blocked while required values are incomplete.
A missing eligible foreign opening-liability CLP basis makes funding and
ready-to-assign unavailable, with reason `opening_liability_clp_basis_missing`;
asset rate absence has a separate reason. Return no partial funding total and do
not use daily debt FX as the opening deduction. The user supplies the signed
CLP basis separately from the original debt balance. A direct correction to
that foreign opening amount requires the user to supply or explicitly confirm
its signed CLP basis in the same update; basis storage/provenance and
apply/offset correction effects are specified in
`desc/13-first-slice-decisions.md`. When an asset rate is
used, return base, quote,
exact decimal, effective date and stale/available status. These response
shapes remain illustrative pending API review.

For example, a category containing CLP 12,000 of confirmed activity plus an
unvalued USD 10 expense can expose:

```json
{"activity":{"status":"incomplete","known_clp":"12000","pending":[{"currency":"USD","amount":"10","transaction_id":"cccccccc-cccc-4ccc-8ccc-cccccccccccc","reason":"final_clp_value_missing"}]},"available":{"status":"incomplete","value":null,"currency":"CLP"}}
```

The CLP figure is a partial subtotal, not total spending. If an eligible
foreign opening liability has no CLP basis, funding and ready-to-assign are
unavailable; an illustrative funding field is
`{"status":"incomplete","value":null,"currency":"CLP","reasons":[{"code":"opening_liability_clp_basis_missing"}]}`.
A rate observation used for foreign assets can be
`{"base":"USD","quote":"CLP","rate":"950","effective_date":"2026-11-03","status":"available"}`.

Malformed requests yield 400, invalid input 422 with field errors, absent scoped
resource 404, stale revision or reused key with different content 409, temporary
storage failure 503, and unexpected integrity failure a sanitized 500. Return a
request ID and stable machine code, for example:
`{"error":{"code":"stale_revision","message":"Review the latest value.","fields":[]},"request_id":"..."}`.
Do not expose SQL, credentials or financial request bodies.

### Draft integrated browser/API/PostgreSQL vertical slice

The proposed real workflow through local Compose is:

1. `GET /api/bootstrap` returns owner-workspace categories and initial view
   state; the request has no workspace selector.
2. `POST /api/accounts` creates one account, its wallets and dated signed
   opening entries atomically. A client idempotency key is required.
3. `POST /api/transactions` records income, expense or one logical transfer
   with actual date and explicit currency amounts. Both sides of a transfer
   commit atomically. Each mutation carries a new idempotency key for that user
   intent.
4. `PUT /api/budgets/{YYYY-MM}/categories/{category_id}/assignment` saves the
   category amount with an expected revision and idempotency key. Over-
   assignment is allowed; the subsequent authoritative budget GET returns the
   warning. The mutation receipt remains minimal.
5. The client refetches affected accounts and the period identified by the
   command/result from PostgreSQL and displays authoritative values. A
   transfer refetches both accounts. Invalidate dependent later budget periods
   when a checkpoint or prior-period change affects them. Keep editable drafts
   outside the query cache and do not optimistically present money as confirmed.
6. At month close, the user reviews each category's carry/return choice and
   submits one close command. A candidate route is
   `POST /api/budgets/{YYYY-MM}/close`, with expected budget/checkpoint revision,
   idempotency key, and a per-category disposition. The use case locks and
   validates the source period, snapshots it, applies choices, establishes the
   next checkpoint and records the receipt atomically. The response contains
   resource IDs/revisions; the client GETs the resulting budget state.

Illustrative CLP check: an initial CLP 1,000 eligible opening balance plus
CLP 500 eligible income gives CLP 1,500 funding. A CLP 200 expense records
CLP 200 of expense activity and reduces category remaining by CLP 200; it does
not debit ready-to-assign again. Assigning CLP 300 to that category leaves
CLP 1,200 ready-to-assign and CLP 100 category remaining, assuming no other
funding or activity. A same-currency transfer between two budgetable accounts
with no fees has no ordinary boundary effect; independent conversion
differences and exposed fees retain their separate once-only effects.

All routes are workspace-scoped in the server. Every mutation and its
financial effects, revision changes and idempotency receipt use one database
transaction. The selected route, close preview/revision contract, incomplete
value errors, and month-end close policy are in
`desc/13-first-slice-decisions.md`. Every successful write is followed by a query
invalidation/refetch; a stale revision returns 409 and retains the browser
draft, while an unknown outcome retries only with the exact persisted request
and key.

### Concurrency, retries and money

For a readable single-owner starting point, serialize financial writes by
locking the workspace row first, then affected records in deterministic ID
order. Re-read balance-dependent data inside the transaction and use scoped,
revision-conditional updates. Unique constraints protect absent rows. This is a
throughput tradeoff; revisit if measured demand requires. Use at most three
total transaction attempts (initial attempt plus two retries), with jitter,
only after confirmed PostgreSQL serialization (`40001`) or deadlock (`40P01`)
aborts. A lost commit response has unknown outcome:
explicitly retry the same command and key. Never hold database locks while
waiting for user input or fetching FX.

Within the transaction, look up and match an existing receipt before applying
time-sensitive validation: an exact November assignment retry in December
returns its original status, resource identity and revision. Only a new
assignment checks the requested current month in `America/Santiago` and the
expected revision. A mismatched key returns 409. Receipts retain method, route,
hash and minimal result metadata, never past financial field values, for the
workspace lifetime. This does not itself define the month-close transition.

Before dispatch, persist exact pending request bytes, method, route, relevant
headers and key in browser origin storage; if persistence fails, fail visibly
before sending. Keep unknown commands when an alert is dismissed and permit
only explicit exact retry. Bind each pending command to a server-issued,
persistent installation generation token; reject mismatch after a database
reset. A normal restart does not rotate it. A restore requires rotating the
generation before accepting writes, because later receipts may have disappeared.
Restore procedure and rehearsal are release gates. Browser storage contains
financial request details and may be cleared; this design promises neither
encryption nor recovery after browser storage loss.

Accepted syntax is plain ASCII decimal strings with digits, an optional leading
minus and decimal point, and no exponent, grouping, whitespace, NaN or infinity.
Parse exactly and validate all arithmetic, reads and aggregates
against the accepted Decimal envelope. Use unconstrained PostgreSQL `NUMERIC`
with explicit finite, scale and 96-bit scaled-mantissa checks: `NUMERIC(p,s)`
can silently round on input. Preserve higher supplied precision within the
envelope; ordinary CLP and USD presentation uses pesos and cents. The accepted
rule rounds a derived CLP foreign-asset valuation once to whole pesos, nearest
with midpoint away from zero, using exact intermediates and retaining the rate.
Define any needed division/precision before use, detect loss and never rely on
implicit library defaults. PostgreSQL and Decimal default rounding can differ.
Business dates remain DATEs, audit instants UTC, and the
current month is computed in `America/Santiago` with inclusive start and
exclusive next-month start.

### Runtime and the close gate

Proposed configuration is typed, with a secret `DATABASE_URL`, bounded pool
(starting suggestion: five connections and five-second acquire timeout), API
bind address and canonical origin. Fail on invalid configuration, database or
locked migration failure; resolve workspace before readiness. DB disconnection
fails readiness; FX outage degrades valuation without failing API readiness.
Check Frankfurter at startup and daily, persist dated observations and use the
latest stored rate with a stale label during outage. Graceful shutdown stops
new work and drains bounded in-flight work. Use immutable
`migrations/<VERSION>_<DESCRIPTION>.sql` files and the same migrator for a
hosted migration-only command. Production rollout compatibility remains open.

The owner accepted explicit close, preservation of the closed snapshot, no
automatic close, and no reseeding the same funds from account balances.
The first calculation starts from a zero baseline; eligible signed opening
balances contribute once to initial ready-to-assign. Existing unassigned
ready-to-assign carries forward. For each category, the
user chooses whether its signed remainder returns to ready-to-assign or carries
in that category (AC-5.5). Preserve the sign: returning a negative remainder
reduces ready-to-assign; carrying it leaves a category deficit.
Until the prior month is explicitly closed, provisional calculations carry
each signed category remainder in the same category into the next month. At
close, the user can retain or release each remainder; subsequent periods
recalculate from that selected checkpoint without double-counting activity or
changing assignments already made.
The close command/response, idempotent atomic snapshot and funding transfer,
and technical first-month initialization are specified in
`desc/13-first-slice-decisions.md`. The
calendar month accepts assignments even when the prior month remains open. Calculate
each month from the most recent explicit close and intervening activity; lack
of a close does not prevent calculation. Budget calculation results after the
last close are provisional and recalculate as activity changes; confirmed
ledger facts remain actual. When a month is closed, its computed values become
the checkpoint for subsequent calculations. For
example, closing November during December makes November's close the checkpoint
for December; recalculate December from that checkpoint and December's activity
without double-counting activity already included in the prior rolling
calculation or changing December assignments. Do not auto-close the prior
month. The selected month-end close eligibility and full funding derivation
are specified in `desc/13-first-slice-decisions.md`; required incomplete
values block close.
For example, funding 100, assignment 80 and spending 30 leave ready-to-assign
20 and category remaining 50. If the 50 remainder returns to ready-to-assign,
the next month's ready-to-assign is 70, not 140 from reseeding the current
account balance. If it carries in the same category, the next month starts
with 20 ready-to-assign and 50 retained in that category. Both options preserve
the closed snapshot of assignment 80, spending 30 and remaining 50; the
snapshot does not add edit-value history. The accepted rule blocks close until
every required input is computable, including category remainders and
funding/ready-to-assign. The incomplete-value reasons and response shape are in
`desc/13-first-slice-decisions.md`. Ledger entries and
assignment intent remain allowed. Advanced multi-period rollover and other transition
details remain open. The
initialized-month examples alone do not establish first-slice readiness.
