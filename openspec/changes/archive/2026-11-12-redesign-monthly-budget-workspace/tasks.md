# Tasks

## 1. Persist budget groups and category organization

- [x] 1.1 Add workspace-scoped budget groups and category membership/order with a forward-only PostgreSQL migration; verify existing categories and transaction references survive migration and starter categories receive deterministic groups.
- [x] 1.2 Add group read/create/update/archive and guarded category delete behavior to the Rust API; verify validation, workspace scoping, archive rules, transaction/history protection, and ordering with API tests.
- [x] 1.3 Add group and category-order contracts and mutation/query helpers to the frontend API layer; verify serialization and error handling with focused unit tests.

## 2. Explain available-to-budget

- [x] 2.1 Extend the backend budget read model with typed calculation components for prior unassigned carry, distinguishable funding (including income when available), actual signed category remainders released by the existing close choice, and assignments; verify components reconcile to the authoritative total in complete worked cases and preserve incomplete states.
- [x] 2.2 Render the prominent available-to-budget card and its accessible detail disclosure; verify category-to-category rollover displays zero and does not affect available-to-budget.

## 3. Build the grouped budget table and category drawer

- [x] 3.1 Render categories under persisted groups with assigned, used, and remaining columns in the Presupuesto tab’s single table titled “Presupuesto”; move “Administrar categorías” into its own tab without further administration refinement; verify ordering, archived-item visibility rules, and incomplete values in frontend checks.
- [x] 3.2 Replace the separate assignment action with a revision-safe inline assigned-value editor; verify decimal-string handling, keyboard commit, closed-period correction reasons, pending-write recovery, and stale-draft retention.
- [x] 3.3 Add an explicit organization edit mode with drag handles and keyboard move controls; verify reordering persists, refreshes authoritative state, and stays locked outside edit mode.
- [x] 3.4 Add a responsive right-side category detail drawer for editing, archive, and eligible permanent deletion; verify focus management, narrow-screen behavior, category-history protections, and authoritative refresh.

## 4. Verify the budget workflow end to end

- [x] 4.1 Add or update integrated browser checks for actual release versus ordinary carry explanation, the single Presupuesto table and separate administration tab, group management/rearrangement, inline assignment, category drawer actions, and archived history; verify the full workflow against the Rust API and PostgreSQL.
