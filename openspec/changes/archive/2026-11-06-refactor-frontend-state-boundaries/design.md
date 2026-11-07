# Design

## Context

See [proposal](proposal.md) for motivation and scope. The accepted [frontend architecture](../../../desc/11-frontend-architecture.md) already selects feature-first organization, local drafts and Colada server snapshots. The current implementation installs Colada but uses manual request state. The app shell owns navigation, dashboard period and appearance; Accounts owns several independent editing workflows. The working tree includes completed finance-page and appearance changes that must remain intact.

## Goals / Non-Goals

**Goals:** Establish a usable Accounts-first pattern with explicit state ownership, authoritative readback, bounded component contracts and shared account cache consumers.

**Non-Goals:** No new product behavior, full Budget/Transactions rewrite, generic repository/controller framework, UI suite, dependency upgrade or financial calculation in the browser. No persisted query cache or offline queue. No arbitrary file-size targets.

## Decisions

### Feature-first boundaries

Use `src/features/accounts/{api,components,composables}` for account types, query definitions, pure draft rules, forms and workflows. Keep routes thin in `views/`. Expose an explicit public accounts API for legitimate consumers rather than importing account component internals. Transaction contracts/query definitions belong to `features/transactions/api`; existing transaction components may remain in place until their own refactor. Shared transport, exact-command persistence and formatting stay in `shared/`.

AccountCreateForm owns its draft and wallet editor; AccountEditPanel owns name-edit state and composes wallet readback, OpeningCorrectionForm and AccountActivity. OpeningCorrectionForm owns amount, signed basis, reason and baseline revisions. A feature pending-command panel owns notice presentation and uses a focused recovery composable. Each form emits narrow typed outcomes, not a whole mutable page controller. Route coordination owns selection by ID, not another mutable copy of the selected server record.

Alternative: merely move the entire view script to `useAccountsPage`. Rejected because it hides rather than separates responsibilities. Moving all frontend folders at once is also deferred.

### Colada owns server snapshots

Use feature-owned query definitions calling the existing typed fetch client. Keys include every result-changing input: accounts list; account ID for detail; account ID, page size and cursor for account activity. Invalid IDs disable detail/activity queries. A changing selection must not publish an older request as the new selection's results. Keep cursor pages isolated and reset accumulated pages when the account changes. Preserve the current activity paging behavior; do not add a new paging UI in this refactor.

Dashboard and transaction-form account options consume the same account-list definition and project the fields they need. Do not copy query data into Pinia. Preserve independent dashboard section errors and URL-owned dashboard/budget periods. Explicit refresh performs a network read; stale cached data may remain visible with an updating/error indication, never as a fabricated zero or a confirmed successful refresh.

TransactionForm currently watches account options as part of draft initialization. Narrow that initialization to the actual edit/new-intent transition: a refreshed options array must not rebuild a dirty transaction draft, clear opening choices or advance its baseline revision. Keep this small compatibility adjustment within the shared account-query migration rather than refactoring the transaction controller.

Alternative: manual loading flags and global accounts stores. Rejected because they duplicate cache ownership and request coordination. Use supported APIs in the installed Colada version; do not assume defaults for refetch, retries or freshness.

### Client state and app shell

Local form/composable state owns drafts, dirty flags and captured revisions. A background read never overwrites a dirty draft or silently advances its expected revision. A 409 preserves the draft and offers explicit review/reload. Independent form instances do not share drafts. Existing reset/cancel behavior remains unchanged.

Extract AppNavigation, DashboardPeriodControl and AppearanceMenu under `app/components`; App.vue remains shell composition and route outlet. Period/filter state stays in the current URL query parameters, not a second store. Retain clock cleanup with its owner. A small Pinia setup store owns shared appearance selection/notice and persistence actions, preserving the existing storage key, allowed themes, restricted-storage fallback and pre-mount theme application. Palette and geometry tokens stay in the current CSS themes. Avoid promoting all selection/form state to global Pinia merely because Pinia exists.

### Financial mutations and cache coherence

Colada mutations wrap, rather than replace, `sendFinancialWrite` and `retryPendingWrite`. No automatic write retries or optimistic financial updates. Persist exact bytes/key/generation before dispatch; storage errors fail closed; unknown outcomes survive dismissal/reload; explicit Retry uses the stored command and verifies installation generation with the uncached bootstrap read. Keep existing pending storage format and keys compatible.

Model write receipt success separately from follow-up read failure: a confirmed receipt remains success and cannot become a retryable write because a refetch failed. Report a readback failure separately and offer read refresh. Retry actions obey the same separation. Map backend field errors to the submitting form.

On create/rename/correction, invalidate account list and affected detail; correction also invalidates account activity. All confirmed financial commands invalidate affected account data and dependent budget/dashboard readers, including when an existing transaction workflow writes. Since Budget, dashboard budget summaries and unfiltered transaction lists remain manually loaded, use a small shared invalidation subscription bridge: cache invalidation for migrated resources, existing read-refresh functions for mounted legacy readers, and normal initial fetch for unmounted views. Subscriptions clean up on unmount and never reset drafts. New period reads keep their current request guards. Budget-dependent corrections conservatively refresh visible budget readers; future cached periods must be invalidated if introduced later. Do not create a second cache inside the bridge.

Alternative: migrate every finance query now. Deferred to keep the pilot bounded. A bridge is temporary and must be documented for removal as remaining workflows adopt Colada.

## Risks / Trade-offs

- Partial migration can leave stale consumers → migrate every account-list consumer together and test the legacy refresh bridge after writes and exact retry.
- Cached refreshes can replace form input → retain separate drafts and baseline revisions; test refetch and conflict while dirty.
- Extracted scoped CSS can lose selectors → move selectors with their owning markup, preserve theme tokens and verify supported layouts/themes.
- Concurrent finance-page work can be overwritten → use the completed change/current working tree as baseline; preserve unrelated edits and do not archive that change here.
- One composable can become another monolith → review cohesive responsibility and trace a validation change to its form/rules without unrelated handler forwarding.

## Migration Plan

Land baseline regression coverage, typed query infrastructure, Accounts components/workflows, shared account consumers/refresh bridge, then shell extraction. Keep each stage type-checkable with existing routes and API shapes. Document boundaries and temporary bridge in a frontend architecture note. Run existing unit/build and applicable Accounts, liability, retry, transaction, budget, appearance and compatibility browser checks against the real API. An independent frontend reviewer audits ownership, request races and command semantics; resolve findings before completion. Rollback only this change's frontend commits; no database migration or browser-storage conversion is required. Do not discard unrelated worktree changes.
