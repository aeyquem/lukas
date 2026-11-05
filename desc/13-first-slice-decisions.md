# First-slice decision record

Status: **Reviewed and ready for implementation**, 2026-11-05. Product decisions
follow the accepted rewrite context and first-slice scope. Technical decisions
were made by the backend, frontend/deployment, and finance members of the
technical committee. The independent review outcome is recorded below. This
record resolves the first web slice only; it does not settle later product
features or self-hosted release operations.

## Decision authority and evidence

- Accepted behavior in `desc/00-rewrite-context.md`, `desc/05-mvp-acceptance.md`,
  and `desc/08-first-slice-plan.md` takes precedence over product references.
- Technical choices in this record are the committee's selected implementation
  baseline under the owner's delegation and readability constraint.
- Actual Budget, Lunch Money, and YNAB were consulted for dated openings,
  budget eligibility, paired transfers, rollover explanations, and correction
  patterns. Their differing month-rollover and reconciliation behavior does
  not replace Lukas's accepted rules. Reconciliation remains later scope.
  References: [Actual accounts](https://actualbudget.org/docs/accounts/),
  [Actual budget and rollover](https://actualbudget.org/docs/budgeting/),
  [Actual transfers](https://actualbudget.org/docs/transactions/transfers/),
  [Actual reconciliation](https://actualbudget.org/docs/accounts/reconciliation/),
  [Lunch Money budgeting](https://support.lunchmoney.app/guides/budgeting/step-4-budgeting),
  [Lunch Money balance adjustments](https://support.lunchmoney.app/guides/budgeting/step-4-budgeting),
  [YNAB starting balances](https://support.ynab.com/en_us/the-starting-balance-an-overview-H1uozOfJs),
  [YNAB month rollover](https://support.ynab.com/en_us/when-the-month-rolls-over-a-guide-rkyyd6qC9),
  [YNAB balance adjustments](https://support.ynab.com/en_us/balance-adjustments-a-guide-rko4OwILs),
  and [YNAB reconciliation](https://support.ynab.com/en_us/reconciling-accounts-a-guide-BJFE3fHys).
- Proposals and open points outside the first-slice gate remain in their
  source documents; this record does not accept them by implication.

## Product and ledger behavior

### Activity, accounts, and starter categories

1. A transaction's actual date selects its budget month, using
   `America/Santiago`; retain the actual date and do not add an assigned-period
   override in this slice. Budget activity is counted in exactly one period.
2. Create account, tracked currency wallets, dated opening entries, and any
   foreign-liability CLP basis atomically. The signed opening entries are the
   initial source of funding; never infer funding again from a later account
   balance read. Account eligibility, not a credit/overdraft limit, determines
   whether a balance participates.
3. Seed the editable expense categories already specified in
   `desc/08-first-slice-plan.md` (Vivienda, Alimentación, Transporte, Servicios
   y comunicaciones, Salud, Educación, Hogar, Vestuario y cuidado personal,
   Restaurantes y ocio, Suscripciones, Seguros e impuestos, Mascotas, Regalos
   y donaciones, Ahorro general, Otros). Store them as workspace data; allow
   rename, reorder, add, and archive when unused. Keep uncategorized activity
   visible and do not seed goals or pools.

### Deterministic funding and period calculation

Calculate budget facts from the zero baseline and explicit events, never from
current account balances:

- Initial ready-to-assign receives each eligible signed opening balance once.
  Value eligible foreign assets using the selected daily observation and the
  accepted CLP rounding rule. Deduct eligible foreign opening debt using its
  separately supplied signed CLP basis once. A missing required asset rate or
  liability basis makes funding unavailable; return no partial total.
- An account/opening added after tracking starts is a dated funding event: add
  its signed eligible CLP value once in the opening date's open period. If its
  opening date is already closed, preserve that snapshot and post the net
  signed opening-funding delta as a linked budget correction in the current
  open correction month. Editing an opening amount follows the same delta
  rule; do not replay the full opening amount. A foreign opening liability
  still requires its paired signed CLP basis.
- Count recognized income in an eligible account once in its actual-date
  period. An unassigned positive ready-to-assign balance carries forward; it
  is not new income in a later month. Income in a tracking-only account does
  not fund the budget.
- Value an eligible source outflow at that wallet amount's CLP carrying value
  immediately before transfer; the bank's conversion rate does not rewrite
  that prior held-money value. Value an eligible destination inflow at the
  actual received value from the conversion evidence, or at the selected
  transaction-date daily rate if no actual CLP value is supplied. Preserve
  exact amounts, currencies, source, date, rate, and the resulting once-only
  difference. The signed funding effect is eligible destination inflows minus
  eligible source outflows. Thus two eligible sides produce the net realized
  conversion difference (CLP 95,000 received against CLP 100,000 of prior
  eligible carrying value removes CLP 5,000 once). An eligible source moved to
  a tracking-only destination removes only the source's prior eligible value;
  the destination is outside budget funding. The reverse adds only the
  destination's received value. Two tracking-only sides have no funding
  effect. Never add another FX adjustment for a difference already included
  in this calculation. Separately exposed fees retain their own category
  treatment. If a required eligible side has no usable valuation, preserve the
  transfer but mark dependent funding incomplete; do not return a partial
  total. A transfer never creates income, expense, or goal funding merely
  because money changed accounts.
- Expenses and refunds change category activity once; they do not reduce
  ready-to-assign a second time. A card purchase is spending in its actual
  budget month; a card payment is a transfer and never repeats that spending
  or the opening-debt deduction. An unvalued foreign expense preserves its
  original-currency account effect and leaves dependent CLP category values
  incomplete with known subtotals and pending original amounts. Assignments
  remain savable; close waits for required values. A confirmed CLP amount
  completes that same expense once.
- Each eligible foreign-asset valuation change contributes only its delta
  once, explained by rate, source, and effective date. It is neither income
  nor a replay of the receipt or conversion that created the balance.
- For period `m`, ready-to-assign is the previous checkpoint's unassigned
  amount plus the period's distinct funding effects, plus signed category
  remainders released at that period's close, minus assignments in `m`.
  Category remaining is its carried remainder plus assignments minus activity.
  Preserve negative values and report over-assignment; never silently clamp.
  An incomplete dependency makes the affected derived value incomplete rather
  than zero.

The first period starts from zero plus eligible openings. For each unclosed
month, carry the prior month's signed category remainder in the same category
provisionally. Existing ready-to-assign carries forward. The current calendar
month accepts assignments while earlier months remain open; each month is
calculated from the newest explicit close plus intervening activity. When an
earlier month closes, recalculate later open months from that checkpoint,
replaying each later event once and leaving saved assignments unchanged.

### Close and correction rules

- A month can close only after its calendar month has ended, only when it is
  the earliest unclosed month, and only when required values are complete.
  This resolves early-close and skipped-period behavior. Closing is explicit;
  no month closes automatically.
- In one transaction, close snapshots the computed period and records the
  user's per-category choice to carry the signed remainder in-category or
  release it to ready-to-assign. Existing unassigned ready-to-assign carries
  automatically. The snapshot stores the choices and resulting checkpoint;
  close never reseeds from account balances. One close per period is allowed;
  an exact retry returns its original receipt, while a new command for an
  already-closed period returns a stable conflict.
- Preserve actual transaction dates during ordinary edits. A date correction
  is explicit and updates the actual date; recalculate both old and new period
  effects atomically and re-resolve any affected opening treatment. Closed
  snapshots never change. Apply the net budget correction in the current
  `America/Santiago` month if it is open, otherwise the next unclosed month;
  link it to the corrected record and update the net linked effect on later
  edits. Do not duplicate wallet effects or apply the same difference through
  checkpoint replay.
- Mutate assignments only while their month is open. An assignment correction
  after close is a linked budget effect in the current open correction month;
  preserve the original snapshot and assignment intent.
- Account name and descriptive fields are editable in place. Account type,
  base currency, and tracked wallet currency may change only before that wallet
  has non-opening ledger activity; otherwise return a clear conflict and
  preserve the financial facts. Opening amount corrections change the desired
  effective opening amount under revision and lock. Opening date/currency can
  change only while that wallet has no transactions; once activity exists,
  use a new wallet/account or a separately reviewed migration feature.
  Eligibility changes remain excluded from this slice.
- Transaction payee, note, and category are editable in place, including
  clearing a category. Amount, account, currency, type, transfer destination,
  and actual date can be explicitly corrected. Replace only affected wallet
  and budget effects atomically; revalidate transfer sides and ask again for
  each wallet whose opening treatment becomes unresolved. Corrections do not
  retain prior field values or support rollback in this MVP.

For example, a closed-month CLP 1,000 expense corrected to CLP 1,200 creates a
CLP 200 budget correction in the current open correction month while leaving
the original snapshot intact. Correcting it later to CLP 1,300 updates that
linked correction to CLP 300 total. The account's historical ledger fact is
corrected once; the budget correction does not create another account debit.

### Opening treatment and foreign-liability basis

For every affected wallet, a transaction before its opening date requires an
explicit choice to apply its balance effect or offset the opening amount to
preserve the current balance. Same-day activity with unknown inclusion requires
the same initially unset choice. Persist the decision per wallet and never
infer chronology. Opening offsets are explicit, dated, provenance-bearing
effects.

For an eligible foreign opening liability, store original signed currency
amount and signed CLP funding basis as separate exact values. Record the basis
source as user-supplied or user-confirmed, its effective opening date, and a
revision. Daily debt FX never changes it. A correction or preserve-current
offset that changes foreign opening principal must atomically supply or confirm
the corresponding signed CLP basis; otherwise the correction is rejected and
funding remains unavailable. Never infer a new basis from current rates.

## Technical committee decisions

### Backend, data, and API

- Use one readable Rust crate with Axum/Tokio, Serde, SQLx runtime queries and
  typed row mappings, `rust_decimal`, `chrono`, `tracing`, and `thiserror`.
  Keep domain calculations as plain functions/types, use cases as the owner of
  database transactions, and SQL/HTTP mapping at the edges. Do not create a
  repository trait and mock for every table.
- PostgreSQL is the only first-slice store. Use explicit immutable SQLx
  migrations; apply pending migrations under the SQLx migration lock before
  readiness in self-hosted startup. Keep a migration-only hosted command as an
  operational mode. SQLite is later scope.
- On startup, after migrations, idempotently create or load one owner workspace
  and its starter categories. Keep its ID in shared API state, pass it
  explicitly through each use case and persistence function, expose no
  workspace selector, and scope every query and foreign key to that workspace.
- Model account, wallet, opening entry, logical transaction, wallet effect,
  category, monthly assignment, close snapshot, correction, rate observation,
  and idempotency receipt distinctly. Create account, wallets, opening entries,
  and basis atomically. One logical transfer owns both wallet effects.
  Use workspace-scoped composite foreign keys, uniqueness on
  `(workspace_id, period, category_id)` assignments, one close snapshot per
  `(workspace_id, period)`, and one idempotency receipt per
  `(workspace_id, key)`. Index workspace/date for transaction lists,
  workspace/period for budget reads, and currency-pair/effective-date for rate
  observations.
- Every mutation requires a client UUID idempotency key. In the same database
  transaction as effects and revisions, store workspace, key, method, route,
  request hash, and minimal result metadata only (no previous financial
  values). Exact retries return the original receipt before time-sensitive
  validation; mismatched content returns `409 idempotency_key_reused`.
  Receipts last for the workspace lifetime. Unknown outcomes are retried only
  by an explicit client retry with exact request bytes and key. Retry confirmed
  serialization/deadlock aborts at most twice after the initial attempt.
- Serialize financial writes by locking the workspace row first, then
  affected rows in deterministic ID order. Use expected revisions for edits,
  unique constraints for absent rows, and re-read all balance-dependent facts
  under the transaction. Stale writes return `409 stale_revision` and retain
  the local draft. Keep the serialized approach until measured contention
  justifies a change.
- Define request/response contracts with each vertical use case; do not freeze
  the entire API or generate a client before wire behavior is proven. Start
  with `GET /api/bootstrap`, account create/read, transaction/transfer
  create/read, budget read/assignment, and `POST /api/budgets/{YYYY-MM}/close`.
  Mutations return IDs/revisions; clients refetch authoritative balances and
  budgets. Responses use stable machine error codes, field errors, and request
  IDs: malformed `400`, invalid `422`, missing scoped resource `404`, stale or
  idempotency conflict `409`, temporary storage failure `503`, sanitized
  integrity failure `500`.
- The close preview is a budget read containing each category's signed
  remainder, ready-to-assign, required-value completeness, and proposed
  per-category disposition. Close submits expected period/checkpoint revision,
  one disposition per category, and an idempotency key. Reject stale previews
  with `409 stale_revision` and incomplete required values with `422` and
  stable field/reason codes. Successful commands return only IDs/revisions;
  the client refetches the snapshot and next open budget.
- Persist exact pending request bytes, method, route, key, and server-issued
  database-generation binding in browser origin storage before sending. If
  storage fails, do not send. Unknown commands survive alert dismissal and
  reload and can only be retried exactly. Cancel cannot erase an unknown
  command. A database restore rotates the generation before writes resume.

### Money, runtime, and deployment

- JSON money/rates use exact ASCII decimal strings and currency codes. Reject
  exponent notation, grouping, whitespace, NaN, and infinity. Preserve exact
  input within the `rust_decimal` 96-bit mantissa and scale 0–28 envelope;
  validate inputs, reads, arithmetic, aggregates, and PostgreSQL values.
  Persist PostgreSQL `NUMERIC` without a rounding typmod and enforce the
  accepted envelope with finite/scale/magnitude checks. Fail on overflow or
  unrepresentable calculation; never silently round. Use `DATE` business
  dates, UTC audit instants, and `America/Santiago` month boundaries. The
  accepted foreign-asset CLP valuation rounds once to whole pesos, nearest
  midpoint away from zero.
- Preserve the accepted sign profile: openings and foreign-liability CLP bases
  are signed, transaction and transfer inputs are positive magnitudes, and
  assignments are nonnegative. Derived balances, category remaining, and
  ready-to-assign may be negative and remain visible.
- Use typed runtime config and a bounded PostgreSQL pool. Validate config,
  connect, migrate, and resolve workspace before readiness. Database failure
  fails readiness; FX outage uses a stale stored observation if available and
  otherwise reports valuation unavailable without partial funding. Shutdown
  stops new work and drains bounded in-flight requests. Production secret
  delivery, backups, restore rehearsal, TLS/proxy, and rolling schema policy
  are release gates.
- Runtime values: require secret `DATABASE_URL` and `CANONICAL_ORIGIN`; default
  `API_BIND_ADDR=0.0.0.0:8080`, `DATABASE_MAX_CONNECTIONS=5`,
  `DATABASE_CONNECT_TIMEOUT=5s`, `DATABASE_ACQUIRE_TIMEOUT=5s`, and
  `SHUTDOWN_TIMEOUT=10s`. Reject invalid values at startup. Provide
  `GET /health/live` for process liveness and `GET /health/ready` only after
  migration and workspace bootstrap; readiness goes false when PostgreSQL is
  unavailable. Embed migrations from `backend/migrations/`; self-hosted startup
  applies them, and `--migrate-only` runs the same migrator without serving
  HTTP.
- Local Compose services are `postgres`, `api`, `vite`, and `dev-proxy`; publish
  only `127.0.0.1:4173`. Keep API, Vite, and PostgreSQL ports private. Validate
  exact Host `localhost:4173`, exact browser Origin `http://localhost:4173`,
  `/api` prefix preservation, HMR WebSockets, and SPA fallback exclusions.
  Production backend and static frontend have separate Dockerfiles. The
  frontend serves SPA routes, proxies `/api` to `API_UPSTREAM`, and loads a
  public non-secret `/runtime-config.js` generated from `API_BASE_URL` before
  app bootstrap. Production does not need Compose.

### Frontend interaction baseline

- Use Vue 3 Composition API, strict TypeScript, Vite, Vue Router, semantic HTML,
  basic CSS, native controls, and the small typed `fetch` client. Keep
  Pinia Colada for server snapshots and local drafts separate from cached
  state. No UI suite or spreadsheet grid is required for this slice.
- Enter submits a validated multi-field form or saves a budget cell; Escape
  discards the form draft or restores the cell baseline. Use accessible field
  errors and announcements, managed dialog focus, and reduced-motion support.
  Save failures show details with Retry and Cancel; cancellation does not
  clear an unknown outcome. Do not optimistically present derived money as
  confirmed.
- Transaction lists use server pagination with default page size 50 and
  maximum 100; search/date-range filters are later. First-slice list review
  uses at least 250 records. Support Firefox and Chrome (smoke-check Zen);
  manually review keyboard-only use and NVDA with Firefox.
- The API must be reachable; there is no offline mode. During implementation,
  demonstrate the real browser → proxy → API → PostgreSQL path, including
  routing, HMR, Host/Origin checks, read-back, persistence, failure, and retry
  behavior. Test doubles can support isolated UI work but cannot be the only
  integration path.

## Independent review

Review status: **PASS**, 2026-11-05. The independent reviewer checked the
record against the accepted context, accounting consistency, first-slice
scope, and technical feasibility. Two initial gaps were corrected and
re-reviewed: the once-only cross-currency transfer valuation rule and the
funding/correction treatment for later account openings. The reviewer found no
remaining first-slice blocker. Runtime behavior still needs implementation
validation as listed in `desc/12-pre-implementation-checklist.md`.

## Deliberately not decided here

CLI command/output/authentication contract; import review and recovery;
reconciliation; report basis; advanced rollover; goals and pools; exceptional
goal refunds; projection and forecast overlap details; FX scenario ranges and
alert thresholds; future identity and multi-member access; SaaS boundary;
SQLite; production PostgreSQL provisioning, backups, TLS, secrets, and rollout
compatibility remain in the later-slice or release gates in the tracker.
