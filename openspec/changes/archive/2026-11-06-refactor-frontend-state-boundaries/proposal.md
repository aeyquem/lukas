# Proposal

## Why

The frontend installs Pinia and Pinia Colada but finance views still manage fetched snapshots, request flags and refresh coordination manually. Accounts also combines creation, editing, corrections, pending-command recovery and presentation in one large view, making changes difficult to review safely.

## What Changes

- Refactor Accounts first into feature-owned components, focused composables, typed API contracts and Colada queries/mutations.
- Establish reusable query definitions for account lists/details and account-filtered transaction pages. Migrate existing account-list consumers in dashboard and transaction entry to that same cache; do not migrate all budget or transaction workflows in this change.
- Preserve the exact-request financial command layer beneath mutations, with explicit retry, authoritative readback and cross-feature freshness rules.
- Keep drafts local and independent of fetched snapshots; use Pinia only for genuinely app-shared client state, initially appearance preferences. Keep shareable filters and periods in URLs.
- Extract cohesive navigation, dashboard-period and appearance responsibilities from the app shell while preserving current URLs, styling, browser behavior and theme persistence.
- Document the feature-first organization and verify behavior with targeted regression checks and independent review.

## Capabilities

### New Capabilities

None. This is an implementation-only refactor; `skip_specs: true` deliberately omits delta specs.

### Modified Capabilities

None. Existing account, budget, transaction and navigation behavior remains unchanged, including the completed but unarchived `complete-finance-page-workflows` change.

## Impact

Frontend Accounts, app shell, theme initialization, shared account consumers, and their tests. Existing Vue, Pinia and Colada dependencies are reused without a dependency upgrade. No backend, schema, API, deployment, authentication, new product feature, visual redesign or offline-mode changes. Budget and transaction controllers remain later refactors except the minimal account-query integration needed for a single cache owner.
