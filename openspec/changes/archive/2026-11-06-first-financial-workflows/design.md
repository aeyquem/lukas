# Design

## Context

The service currently owns one workspace and starter categories and exposes health routes. The Vue app is a shell. The reviewed first-slice model and its detailed financial, money, API, concurrency, runtime, and browser contracts are in `desc/08-first-slice-plan.md` and `desc/13-first-slice-decisions.md`; those documents define behavior for this change.

## Goals / Non-Goals

**Goals:** Add the smallest complete account, ledger, and monthly-budget implementation that follows the accepted first-slice rules and can be exercised through the existing local browser-to-PostgreSQL topology.

**Non-Goals:** Authentication, CLI, imports, reconciliation, goals, pools, projections, advanced rollover, multi-member access, offline operation, production backup/TLS provisioning, and historical value rollback.

## Decisions

1. **Extend the existing Rust crate in readable feature modules.** Add domain calculations, application use cases, PostgreSQL persistence, and Axum request/response handlers in the current crate, grouping by accounts, transaction ledger, and budget. Keep SQL explicit and transaction ownership at use-case boundaries. This follows the selected architecture and keeps the route-to-row path visible to a Rust beginner. A separate service or generic repository framework would add indirection without an alternative implementation need.

2. **Persist financial facts and budget checkpoints as distinct records.** Add ordered immutable migrations for wallets, openings and liability bases, transactions and wallet effects, idempotency receipts, assignments, rate observations, corrections, and close snapshots as needed by the requirements. Use workspace-scoped composite constraints. A single logical transfer owns both wallet effects. The workspace row is the first serialization lock for financial writes; lock affected records by stable ID order, check revisions, and commit every affected fact and receipt atomically. This prevents cross-workspace references and partial transfer/correction outcomes. Do not store unexplained mutable balances as authoritative facts.

3. **Calculate from dated events with bounded exact decimal operations.** Introduce the selected `rust_decimal` and `chrono` types and validate the exact ASCII decimal/date wire forms at API boundaries. Store `NUMERIC` without a rounding typmod and enforce the accepted Decimal envelope. Keep money operations checked and report unavailable/integrity failures rather than substituting zero or partial totals. Separate pure funding/period calculations from SQL and HTTP. Use `America/Santiago` only to map actual business dates to periods.

4. **Treat FX observations as explicit persisted inputs.** For eligible foreign assets, request the selected Frankfurter v2 daily mid-market observation, store its source and effective date, and reuse the latest stored rate as stale during outage. If there is no usable observation, report funding unavailable. Do not use the asset rate for foreign opening debt; its user supplied or confirmed signed CLP basis is a separate stored fact. The backend must retain and expose valuation provenance so the UI can explain the result.

5. **Define API contracts per use case and keep authoritative results on the server.** Start with bootstrap, account create/read/edit, paginated transaction read and activity create/edit, budget read/assignment, and close preview/command routes. Use stable error codes, field errors, request IDs, revisions, and command receipts. Writes take a UUID idempotency key; save the request hash and minimal result in the same transaction as financial effects. Exact retries return that receipt; changed content under a key conflicts. Do not generate a whole-API client or add a generic API schema before date and decimal behavior are proven.

6. **Use local component drafts and a small typed fetch client.** Add Vue Router routes for accounts, activity, and monthly budget. Add Pinia Colada for cached server snapshots; editable forms remain local drafts with a baseline revision and are not bound to cached objects. Invalidate affected queries after success, and refetch authoritative balances and budgets. Store pending exact request bytes, route, method, idempotency key, and database-generation binding in origin storage before sending. Unknown outcomes survive reload and can only be retried exactly; if persistence fails, do not send. A timeout is not presented as a definite rejection.

7. **Build the UI with native controls and basic CSS.** Use native forms and tables with clear field-level validation. Defer formal accessibility review until the final UI design; it is outside this MVP. Keep budgets savable when their coverage is incomplete, but label unknown derived values and block close as specified. Use server pagination at the accepted bounds. Do not add a spreadsheet grid or general UI library.

8. **Match currency wallets to account type.** Bank, savings, and investment accounts have one currency wallet. Card and cash accounts may have one wallet per tracked currency; cash may also be split across separate accounts. Enforce this at the account API boundary and in the creation form; changing a multi-wallet draft to a single-currency type must not silently discard extra wallet entries.

## Risks / Trade-offs

- [The first slice combines ledger corrections, event-derived budgets, and rollover, so a superficially simple schema can misstate money] → Keep facts, derived values, and closed checkpoints distinct; implement in vertical stages and validate the accepted worked cases against PostgreSQL.
- [External rate availability can make initial funding incomplete] → Persist observations and provenance, use stale stored rates only as specified, and expose a clear unavailable state without partial totals.
- [Workspace-wide serialization reduces concurrent write throughput] → Use it for correctness at the expected single-owner scale and revisit only with measured contention.
- [Browser storage can be cleared or unavailable] → Refuse to send a mutation if its exact retry record cannot be persisted, and surface the pending command state clearly.
- [Large change increases review surface] → Keep tasks grouped by backend, frontend, and integrated workflow; require a reviewer and tester cycle before archive, and keep advanced product areas out of scope.

## Migration Plan

Add forward-only SQLx migrations; the existing startup migration path applies them before readiness. No financial production data exists yet, so rollback is to restore the prior development database snapshot or recreate the local database, not to mutate or reverse applied migration files. Keep each migration additive and safe for repeated backend startup. Validate clean install, upgrade from the current workspace/category schema, and persistence across restart.

## Open Questions

None for this first slice. Later deployment, identity, CLI, import, and advanced finance questions remain outside this design.
