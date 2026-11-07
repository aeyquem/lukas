# Frontend state ownership

Routes compose feature UI and coordinate route context. Feature contracts,
queries, mutations, focused workflows, and domain-facing components live under
`src/features/<feature>`. Cross-feature consumers use a feature's public entry
file instead of importing its implementation details.

## Feature boundaries

- Accounts: `src/features/accounts/accounts-feature.ts` exports account queries,
  mutations, contracts, and account UI. Account forms own their drafts and
  baseline revisions; query readback never edits form state.
- Transactions: `src/features/transactions/transactions-feature.ts` exports
  transaction page/category/activity queries, exact-write mutations, cursor-page
  coordination, contracts, form/list components, and field messages. The route
  owns the selected transaction and page composition; page snapshots live in
  Colada while search, sort, and edits remain local.
- Budget: `src/features/budget/budget-feature.ts` exports budget/category/close
  queries, write and retry mutations, budget components, workflows, and public
  contracts. `useBudgetAssignments` owns period-keyed amount/reason drafts;
  `BudgetCategoryManager` owns category editing drafts; the route owns URL
  period selection and composes the summary, category table, and close panel.
- Dashboard: `src/views/home-view.vue` composes the public account, budget, and
  transaction query definitions. Its account list keeps the established
  30-second freshness policy; budget and recent transactions force a read on
  dashboard entry. The dashboard period remains independent in the
  `dashboardPeriod` URL query parameter.

Pure financial display and draft rules stay in feature model files. Formatting,
transport, and exact pending-command persistence stay under `src/shared`.
Components keep their existing markup hierarchy, scoped styles, wording, and
responsive behavior during the structural refactor.

## Query ownership and freshness

Colada owns fetched snapshots. Query keys include all result-changing inputs:

- Accounts: `['accounts', 'list']`, account details by ID, and activity by
  account ID, page size, and both cursor fields.
- Transactions: categories and ledger pages by page size and cursor. The full
  transaction route uses bounded 50-item pages; the dashboard uses the same
  page query at 20 items and displays the first six.
- Budget: snapshots by period, categories by their shared key, and close
  previews by period.

Budget route entry explicitly activates a forced read before exposing cached
data as current. Once successful, active budget/category observers respond to
targeted invalidation. The dashboard uses the same budget query and forces its
own entry read. Close previews are fetched only on explicit user action. The
transaction route's page coordinator forces each page read and protects page
ordering with its generation guard; dashboard recent activity uses the public
transaction query. Account list freshness and the explicit refresh behavior
remain unchanged.

Loading, error, incomplete, unavailable, and empty states stay distinct. A
failed request never turns an unavailable value into a confirmed zero. Local
drafts stay separate from server snapshots, and query invalidation cannot
overwrite dirty input. Budget assignments retain period/category keys and
expected revisions; conflicts keep drafts for review. Transaction cursor and
page size are part of query identity, and sorting/searching remain scoped to the
currently loaded ledger pages.

## Writes and invalidation

Feature mutations call the shared `sendFinancialWrite` and
`retryPendingWrite` command layer. Commands are persisted before dispatch,
storage failure prevents dispatch, and retry sends the exact stored bytes,
idempotency key, and installation generation. Writes are never optimistic or
automatically retried. A confirmed receipt remains confirmed if subsequent
readback fails; the relevant screen reports the refresh problem separately.

`src/shared/api/finance-query-invalidation.ts` is the single cross-feature
invalidation entry point. Account and transaction writes invalidate mounted
account/activity, transaction-page, and budget snapshot/preview readers.
Budget writes invalidate budget categories, snapshots, and previews. Dashboard
uses those same query keys, so mounted sections update without callback
subscriptions. Draft state remains in its feature workflow across those reads.

The former `finance-invalidation.ts` callback bridge has been removed. Query
keys and invalidation impact stay explicit; the shared invalidation module owns
no fetched data and does not host feature workflows.

## App-shared preferences

App-shared client preferences belong in a Pinia setup store. Shareable periods
and filters remain in the URL. Theme palette and geometry tokens remain CSS;
the preference store applies the selected theme before mount and preserves the
existing storage format and restricted-storage notice.
