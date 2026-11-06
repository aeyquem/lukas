# Frontend state ownership

Accounts is the pilot for feature-owned state. Routes compose UI and own selection
by ID. `src/features/accounts/components` owns creation, editing, opening
correction, pending-command presentation and activity. Creation and correction
forms own independent local drafts; fetched account records are never editable
form models. Pure draft defaults and validation live in `account-draft.ts`.

Other features consume the explicit public exports in
`src/features/accounts/accounts-feature.ts`, rather than account component internals.
Account HTTP contracts and query/mutation definitions live in `accounts/api`;
transaction contracts and activity queries belong to `features/transactions/api`.
Shared transport, pending-command storage and formatting remain under `shared`.

Colada owns fetched account snapshots. Account keys are `['accounts', 'list']` and
`['accounts', 'detail', id]`. Activity keys begin with `account-activity` and
include account ID, page size and both cursor fields. Empty or invalid IDs disable dependent requests. Activity is scoped to
its selected account and retains the existing first 50 recent transactions UI;
there is no accumulated cross-account page array. Reads have an explicit 30-second
freshness window, with focus/reconnect refetch disabled. Explicit Refresh calls
`refetch`, which performs a network read. Errors remain visible alongside cached
values; a failed read never manufactures a zero balance.

Each draft captures its baseline revision on initialization. Refetch updates the
readback, without overwriting unsaved input or advancing the baseline. A 409
retains the draft; explicit review/reload adopts the current snapshot. Transaction
entry initializes on edit/new intent changes, rather than account-option array
replacement. Option refresh therefore cannot erase opening-treatment choices.

Account Colada mutations wrap the existing exact-command persistence layer.
They do not optimistically alter financial values or automatically retry writes.
The command bytes, idempotency key and installation generation are persisted
before dispatch; storage failure prevents dispatch. Explicit retry sends the
persisted command after an uncached generation check. Dismissal hides a notice
without deleting the unresolved command.

A successful receipt and its authoritative readback are distinct outcomes.
`writeThenRead` returns the confirmed receipt even when the read fails. Forms
report that separately and offer refresh, rather than suggesting another write.
Read values alone provide the displayed confirmed money.

`shared/api/finance-invalidation.ts` is a temporary notification bridge. It owns
no snapshots: mounted legacy budget/dashboard/transaction readers subscribe with
their existing guarded refresh functions, and clean up with their Vue scope.
Account invalidation refreshes Colada account/activity entries; legacy refreshes
retain local drafts. Every confirmed financial write and exact retry triggers
this coordination. Unmounted readers load normally on their next mount. Remove
the bridge subscription for each reader when that resource moves to Colada;
then replace the corresponding notification with query-key invalidation. Remove
the bridge entirely once the final legacy financial reader migrates.

App-shared client preferences belong in a Pinia setup store. Shareable periods
and filters remain in the URL. Theme palette and geometry tokens remain CSS;
the preference store applies the selected theme before mount and preserves the
existing storage format and restricted-storage notice.
