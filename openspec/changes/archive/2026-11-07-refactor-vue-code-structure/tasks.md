# Tasks

## 1. Transaction feature ownership

- [x] 1.1 Add feature-owned transaction query definitions and migrate bounded cursor pagination; verify query identity includes page size and cursor, every route entry refetches even with cached data, and tests preserve ordering, current-page search/sort, load-more, and stale-response behavior.
- [x] 1.2 Move transaction contracts, models, form/list components, and workflow state into `features/transactions`; reduce the route view to composition and verify existing create/edit fields, opening-treatment choices, validation, messages, and rendered hierarchy remain unchanged.
- [x] 1.3 Wrap transaction writes and exact retries in feature mutations using the existing pending-command layer; verify fail-closed persistence, unknown-outcome recovery, receipt/readback separation, and field errors with focused tests.

## 2. Budget feature ownership

- [x] 2.1 Add period-keyed budget and category queries, including close preview reads; verify every route entry refetches even with cached data, period changes remain URL-owned, and failures retain their current visible states without substituting zero.
- [x] 2.2 Implement budget assignment, category, close, and retry mutations with targeted cache invalidation; verify expected revisions, closed-period reason requirements, preview confirmation, dirty-draft retention, and stale-conflict behavior with focused tests.
- [x] 2.3 Move budget models, draft rules, and UI components into `features/budget` and reduce the route view to composition; verify current category editing, assignment, close flow, wording, layout, and period navigation remain unchanged.

## 3. Dashboard composition and bridge removal

- [x] 3.1 Migrate dashboard budget and recent-transaction reads to the public budget and transaction query definitions; verify those populated keys refetch on route re-entry while the shared account-list query keeps its existing freshness policy, period context remains independent, section failures remain independent, refresh stays explicit, and unavailable values never render as confirmed zero.
- [x] 3.2 Consolidate cross-feature write invalidation over explicit query keys and remove `finance-invalidation.ts` after all subscribers have migrated; verify account, transaction, and budget writes plus exact retries refresh mounted dashboard and finance readers without resetting drafts.
- [x] 3.3 Update `frontend/ARCHITECTURE.md` with completed feature ownership, query key, draft, and invalidation boundaries; verify all documented module paths and that no legacy bridge imports remain.

## 4. Integration and review

- [x] 4.1 Run frontend unit tests, type-check/build, and applicable real-API browser checks in Firefox and Chrome plus a Zen smoke check; verify routes, forms, pending writes, pagination, period URLs, themes, and existing layouts remain behaviorally and visually stable.
- [x] 4.2 Have an independent frontend reviewer audit feature boundaries, query freshness/invalidation, cursor races, dirty draft revisions, exact-retry safety, and visual preservation; resolve findings and repeat the review until it passes.
