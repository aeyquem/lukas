# Design

## Context

See [proposal.md](proposal.md) for motivation and scope. The completed `refactor-frontend-state-boundaries` change established Accounts as the Colada and feature-ownership pilot, extracted the app shell, and introduced a temporary notification bridge for mounted legacy readers. Budget and the unfiltered transaction page still hold most query, mutation, draft, and recovery state in route views. The dashboard already reuses the account query but fetches budget and recent activity directly.

The accepted frontend architecture in `desc/11-frontend-architecture.md` requires server snapshots to remain separate from local drafts, URL-owned shareable context, backend-authoritative financial values, and exact persisted commands for unknown write outcomes. The implementation must preserve existing routes, templates' rendered hierarchy, styles, forms, messages, pagination, and API payloads.

## Goals / Non-Goals

**Goals:**

- Extend the established feature-first structure to all remaining financial screens.
- Make Colada the owner of fetched budget, category, transaction, and dashboard server snapshots.
- Remove duplicate request coordination and the temporary bridge after all of its readers use queries.
- Keep feature modules independently understandable and expose explicit public APIs for cross-feature reads.

**Non-Goals:**

- Redesigning page layout, changing theme or colors, or changing navigation and user-visible workflows.
- Changing financial rules, API contracts, backend behavior, or which values are authoritative.
- Adding a generic repository/controller framework, a new UI or state library, offline support, or optimistic financial updates.
- Refactoring deployment configuration or introducing new product features.

## Decisions

### Complete feature ownership rather than adding page controllers

Organize transaction code under `features/transactions` (API contracts, keyed queries, mutations, domain-facing models, forms/lists, and focused workflow composables) and budget code under `features/budget` (API contracts, period/category queries, mutations, draft rules, and view components). Move `components/activity` and `components/budget` into their owning features. Add a small `features/dashboard` composition layer for dashboard-only presentation and query composition. Keep `views/` as route entry points that assemble feature components and preserve current route behavior.

Cross-feature consumers, especially dashboard, import only explicit feature exports. The dashboard reuses account, budget, and transaction query definitions instead of issuing duplicate fetches. Account editing and the app shell from the completed change remain intact; only their invalidation integration changes as needed to remove the bridge.

Alternative: move each existing page script wholesale into a `use*Page` composable. Rejected because a page-sized composable would preserve the existing coupling behind a different filename. A generalized application-wide repository layer is also rejected; query and mutation definitions remain with their owning feature.

### Colada owns snapshots; drafts stay local

Define query keys within each feature and include every input that changes results, including the dashboard or budget period and transaction/account cursor and page size. Preserve the existing route-entry network reads: budget snapshot/category queries, full transaction-page queries, and the dashboard's budget/recent-transaction queries must force a network read on route entry, because those legacy readers fetch on every mount even when a previous route visit populated local data. Budget snapshot and category queries use an explicit activation action on route entry so the first request is forced before enabling their observers; this prevents both stale cache display as confirmed current data and a duplicate first request. The dashboard account list continues using the shared account query and its existing 30-second freshness and refetch policy established by the completed Accounts refactor. Close-preview reads remain explicit user-triggered network reads. On route entry, keep the existing initial loading/error presentation and do not present an older cached response as confirmed current data before the legacy reader's request resolves. Once a page has loaded, preserve its existing refresh behavior, including retaining the last good read alongside a refresh error where it does so today. Preserve the transaction page's existing bounded page loading and current-page search/sort behavior; do not introduce new pagination or URL semantics.

Keep editable values in component/composable drafts with their original baseline revisions. Query refetches may update readback but must not overwrite dirty input or advance a draft's expected revision. Conflicts retain the draft and the existing explicit review/reload path. Keep period context in the existing URL query parameters. Loading, stale, partial-failure, and empty states remain independently visible where they are today; failed reads must not become zero values.

Alternative: store responses and drafts together in Pinia or bind inputs directly to cached records. Rejected because refetch and conflict handling could silently discard user input or present unconfirmed values as saved.

### Keep write safety in the shared command layer

Feature mutations call the existing `sendFinancialWrite` and `retryPendingWrite` functions. They do not generate parallel persistence formats, retry writes automatically, or update displayed financial values optimistically. A receipt remains a confirmed write even if its subsequent query refresh fails; the UI reports readback failure separately. Field errors, pending-command dismissal, exact retry after reload, and generation checks keep their current behavior.

Alternative: replace pending commands with Colada mutation retries. Rejected because unknown-outcome financial writes require an exact persisted request and explicit user retry, not automatic retry policy.

### Replace notifications with explicit cache invalidation

As each feature migrates, its successful writes invalidate the affected query keys. Keep invalidation targeted to affected accounts/activity, budget periods/categories, and transaction pages, with dashboard queries sharing those same keys. Once all readers in dashboard, budget, and transactions are Colada-owned, remove `finance-invalidation.ts` and the subscription calls. Use one small cross-feature invalidation entry point only where a mutation affects multiple feature resources; it coordinates query keys and stores no data.

Alternative: retain a global event bus or duplicate query snapshots in a shared store. Rejected because the current bridge has to invoke arbitrary refresh callbacks, while Colada can notify mounted consumers directly from its cache.

### Preserve presentation while moving ownership

Move scoped styles with their current markup and keep the rendered page hierarchy, text, CSS tokens, theme files, and responsive behavior stable. Extract components around existing user workflows rather than around an anticipated redesign. Keep the boundaries flexible enough for later page composition changes without moving server state or drafts back into route views.

### Update architecture documentation and keep dependency footprint fixed

Update `frontend/ARCHITECTURE.md` to describe the completed ownership boundaries, query key ownership, draft lifetime, cross-feature invalidation, and removal of the temporary bridge. Reuse the installed Vue, Pinia, and Pinia Colada versions; do not add a dependency or upgrade packages as part of the refactor.

## Risks / Trade-offs

- A mutation may fail to invalidate a dependent dashboard or period query → maintain a write-to-query impact map and verify each existing write and exact retry against mounted consumers.
- A shared cache may suppress a legacy route's mount-time network request or briefly expose old values → require `refetchOnMount: 'always'` for migrated route queries and verify route re-entry against a populated cache, including loading and failure presentation.
- Refetches may erase drafts or change baseline revisions → retain local draft ownership and verify dirty/refetch and conflict flows for budget assignments, categories, and transactions.
- Cursor-based pages may mix responses after a write or query change → include cursor and page size in query identity and preserve existing page-generation/reset rules.
- Moving scoped components may alter styles or responsive layout → move selectors with their markup and compare the same routes, themes, and viewport behavior before and after.
- A cross-feature invalidation helper may grow into a generic framework → keep it to explicit query-key invalidation and avoid data ownership or callback subscriptions.

## Migration Plan

1. Record current route and write/read behavior as regression cases, including pending exact-retry flows, period URLs, partial dashboard errors, transaction pagination, and dirty draft conflicts.
2. Establish transaction and budget feature contracts, query keys, and mutation entry points using existing transport and pending-command modules.
3. Move transaction and budget components/models into their features and reduce their route views to composition while migrating reads and writes without changing templates' visible hierarchy.
4. Compose dashboard reads from the exported account, budget, and transaction queries; remove its manual refresh state when equivalent query status is available.
5. Switch every legacy reader off the notification bridge, consolidate explicit query invalidation for cross-feature writes, then remove the bridge module and its exports.
6. Update frontend architecture notes and run the existing focused unit, build/type-check, and real-API/browser regression checks. Have an independent frontend reviewer audit query ownership, invalidation, request races, dirty drafts, write safety, and visual preservation; address findings and repeat review until passed.

If a stage regresses, revert only the files changed by this follow-up and retain the completed account/app-shell refactor and unrelated workspace edits. No database migration, API compatibility window, deployment change, or browser-storage migration is required.
