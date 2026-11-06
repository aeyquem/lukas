# Tasks

## 1. Financial persistence and exact value foundation

- [x] 1.1 Add the selected decimal/date dependencies and strict decimal-string/date parsing types; verify accepted-envelope, sign, precision, and invalid-wire-value cases with focused unit tests.
- [x] 1.2 Add ordered PostgreSQL migrations for wallets, openings, foreign-liability bases, ledger transactions/effects, idempotency receipts, assignments, rate observations, corrections, and close snapshots with workspace-scoped constraints; verify fresh migration and upgrade from the existing workspace/category schema against PostgreSQL.
- [x] 1.3 Add typed row mappings and persistence functions for account, wallet, category, transaction, budget, FX observation, receipt, and close records; verify every query through PostgreSQL integration coverage.
- [x] 1.4 Document the financial table relationships and migration/runtime behavior in the backend developer documentation; verify the documented fresh setup and upgrade commands against the local Compose stack.

## 2. Account and transaction-ledger APIs

- [x] 2.1 Add workspace bootstrap and account read/create/edit use cases and routes, including required name/type/base-currency/budget-eligibility fields, tracking-only investment default, atomic multi-currency openings, foreign-liability basis, revisions, and stable field errors; verify rollback and read-back against PostgreSQL.
- [x] 2.2 Add transaction, refund, and linked-transfer create/read/edit use cases with per-wallet opening treatment, exact amounts, actual dates, confirmed foreign expense values, and paginated listing; verify same-currency, cross-currency, refund linkage, pre-opening, same-day, and correction cases against PostgreSQL.
- [x] 2.3 Add transactional UUID idempotency receipts and workspace-first deterministic locking for financial writes; verify exact retries, changed-payload conflicts, stale revisions, rollback on partial write failure, and bounded serialization/deadlock retry behavior.
- [x] 2.4 Add accounts-and-ledger API contract documentation with routes, wire examples, stable error codes, revisions, pagination, and the accepted 400/422/404/409/503/sanitized-500 mapping; verify examples and status codes against the implemented handlers.
- [x] 2.5 Restrict bank, savings, and investment accounts to one currency wallet while allowing multiple wallets for cash and cards; validate API rejection, form behavior when switching account types, and cash account read-back through PostgreSQL and browser flows.

## 3. Monthly budget calculations and API

- [x] 3.1 Implement pure checked funding, transaction/refund attribution, category activity, and period calculation functions; verify signed openings, dated later openings and their correction deltas, eligible-account income and no funding from tracking-only income, eligible transfer effects, one-time foreign-asset revaluation deltas, negative values, incomplete dependencies, and accepted worked cases.
- [x] 3.2 Add persisted daily FX observations using Frankfurter v2 for eligible foreign assets, including effective-date/source provenance and stale stored-rate fallback; verify valuation rounding, stale-rate behavior, and unavailable-without-partial-total behavior.
- [x] 3.3 Add category management and budget read/assignment APIs with revisions and authoritative calculated values; verify starter-category idempotency, uncategorized activity, assignment conflicts, and PostgreSQL read-back.
- [x] 3.4 Add chronological explicit close preview/command, atomic snapshots, signed per-category dispositions, checkpoint replay, and closed-period correction effects; verify incomplete/out-of-order rejection, exact retry, stale preview, and later-period recalculation without changing assignments.
- [x] 3.5 Document the budget calculation inputs, incomplete states, FX provenance, assignment behavior, and close contract; verify documented examples against calculation and PostgreSQL integration cases.

## 4. Browser financial workflows

- [x] 4.1 Add Vue Router feature routes, the typed fetch client, Pinia Colada setup, stable API error mapping, and shared exact money/date formatting; verify type-check and production build.
- [x] 4.2 Add account and opening-balance forms with local drafts, field errors, currency wallets, and authoritative account views; verify create/edit/read behavior in component tests and browser flow.
- [x] 4.3 Add manual income, expense, refund, and transfer forms plus paginated transaction review and correction flows; verify refund linkage/category behavior, currency/date validation, opening-treatment prompts, idempotent retry states, and read-back in browser flow.
- [x] 4.4 Add current-month budget views, category assignment editing, incomplete-value explanations, and close preview/confirmation; verify signed/unknown states, stale draft retention, and successful refetch in browser flow.
- [x] 4.5 Persist pending write bytes, method, route, idempotency key, and database-generation binding before send; verify storage failure prevents writes, unknown outcomes survive alert dismissal and reload, exact retry returns the original result, and cancel does not erase unresolved commands.
- [x] 4.6 Document browser support and verify the required Firefox, Chrome, and Zen functional smoke checks. Formal accessibility review is deferred until the final UI design; it is not an MVP gate.

## 5. Integrated first-slice evidence

- [x] 5.1 Exercise account creation through browser → dev proxy → API → PostgreSQL, including multi-currency card and cash accounts, atomic failure, restart persistence, and read-back; record exact commands and results in the implementation handoff.
- [x] 5.2 Exercise income, expense, transfer, edits, funding, assignment, incomplete foreign expense, rate outage/stale fallback, close, and correction through the real Compose/PostgreSQL path; record results and inspect database facts for no duplicate effects.
- [x] 5.3 Exercise 250+ transaction records, pagination boundaries, exact retry after simulated lost response, database outage/readiness, browser routes, HMR, and Host/Origin checks; record evidence and reviewer/tester outcomes in the handoff.

## Workflow follow-up

- Run the independent reviewer and tester cycle, resolve findings, and repeat until both pass.
- Run OpenSpec strict validation and archive only after implementation evidence and review pass.
- Update `orchestrator.md` with the change branch, progress, evidence, and next feature.
