# Tasks

## 1. Baseline and contracts

- [x] 1.1 Preserve the completed finance-page working-tree baseline and add targeted Accounts regression coverage for create, rename, correction and selection; verify existing unit tests and document baseline browser results without resetting unrelated edits.
- [x] 1.2 Introduce feature-owned account/transaction API contracts and relocate pure account draft/validation rules without changing payloads; verify type checking and decimal/date, currency-count and signed-basis tests pass.

## 2. Query and mutation foundation

- [x] 2.1 Implement Colada account-list/detail and account-activity query definitions with explicit keys, enabled conditions and freshness policy using installed APIs; verify shared list requests, explicit refresh, late-response isolation and account-switch page reset with focused tests.
- [x] 2.2 Wrap account writes and exact retries in focused Colada mutations while retaining existing pending-command persistence; verify no automatic retry, fail-closed storage, generation mismatch, reload recovery, dismissal retention and receipt-success/read-failure separation with targeted tests.
- [x] 2.3 Add and document the small invalidation bridge for migrated cache entries and mounted legacy readers; verify create/rename/correction and transaction mutations/exact retries refresh relevant account and budget/dashboard readers without resetting drafts or creating duplicate cache ownership.

## 3. Accounts pilot

- [x] 3.1 Extract account creation and wallet editing with locally owned drafts, narrow typed contracts and relocated scoped styles; verify create validation, wallet preservation on type change, labels and existing default behavior.
- [x] 3.2 Extract account edit/readback and opening correction workflows with captured revisions and local draft ownership; verify dirty drafts survive refetch and 409, amount/basis corrections remain paired and confirmed money comes only from authoritative reads.
- [x] 3.3 Extract pending-command presentation and account activity, then reduce AccountsView to selection-by-ID and composition; verify rapid selection changes, recovery actions and existing account-filtered activity behavior using focused tests/browser checks.
- [x] 3.4 Switch dashboard and transaction-entry account consumers to the shared query definition, narrowing TransactionForm initialization watchers so option refetches cannot rebuild dirty drafts; verify dashboard partial failures, wallet choices, opening choices and baseline revisions survive account refetches after transaction writes and account edits.
- [x] 3.5 Add a frontend architecture note covering feature ownership, query keys, mutation/readback separation, draft lifetime and temporary bridge removal; verify examples match the delivered module boundaries and imports.

## 4. App shell

- [x] 4.1 Extract navigation and dashboard-period controls while keeping URL query names and clock cleanup; verify direct links, independent budget/dashboard periods, active navigation and current narrow-screen layout.
- [x] 4.2 Extract the appearance menu and move shared preferences into a Pinia setup store, preserving pre-mount initialization and storage compatibility; verify all current themes, reload persistence, restricted-storage notice and unchanged CSS tokens through focused checks.

## 5. Integration and independent audit

- [x] 5.1 Run frontend unit tests, type checking/build and applicable real-API Accounts/liability/retry/transaction/budget/appearance checks, plus supported-browser compatibility checks; record outcomes and resolve regressions without changing product contracts.
- [x] 5.2 Have an independent Vue/server-state reviewer audit cache ownership, request races, dirty draft revisions, exact-retry safety, CSS ownership and scope; resolve findings and repeat review until the reviewer passes the change.
