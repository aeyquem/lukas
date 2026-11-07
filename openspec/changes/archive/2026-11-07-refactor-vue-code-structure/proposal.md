# Proposal

## Why

The completed `refactor-frontend-state-boundaries` change established feature-owned state with Accounts as the pilot and extracted the app shell, but budget, dashboard summaries, and most transaction state still use direct requests and page-level controllers. Finishing the remaining migration will give the Vue application one coherent feature-first structure and remove temporary cross-page refresh coordination without changing the product users see or its behavior.

## What Changes

- Refactor the remaining budget, transaction, and dashboard code into feature-owned API, model, composable, and component modules, leaving route views as composition points.
- Move transaction activity components and budget models out of generic `components/` locations into their owning features.
- Use Pinia Colada for fetched server snapshots and cache invalidation across budget, transaction, and dashboard readers; keep unsaved drafts local and URL-owned period context in the route.
- Retain the existing exact-command persistence, explicit retry, authoritative readback, and mutation semantics while integrating writes with feature-owned cache invalidation.
- Remove the temporary finance invalidation bridge after all of its legacy readers have migrated.
- Preserve current routes, user-visible workflows, screen layout and information hierarchy, themes, API contracts, and financial behavior. The visual redesign is a later change.

## Capabilities

### New Capabilities

None. This is an implementation-only refactor; `skip_specs: true` deliberately omits delta specs because no externally observable requirements change.

### Modified Capabilities

None. Existing account, budget, transaction, and navigation requirements remain unchanged.

## Impact

Remaining Vue application code under `frontend/app/src/views`, `frontend/app/src/components/activity`, `frontend/app/src/components/budget`, and the `features/` and `shared/api/` modules, plus frontend tests and architecture documentation. Reuses the installed Vue, Pinia, and Pinia Colada dependencies. No backend/API, database, deployment, dependency, theme, or product-scope changes.
