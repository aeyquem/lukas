# Tasks

## 1. Navigation and dashboard

- [x] 1.1 Add named routes and primary links for all requested areas, with active-route state and truthful unavailable pages; verify every nav link resolves to its intended page.
- [x] 1.2 Add independent dashboard month URL state and read-only account, budget, and recent-transaction summaries; verify the selected month drives only dashboard summaries and failed sections never show zero as a fallback.

## 2. Budget review

- [x] 2.1 Keep the selected budget month in its own URL state and preserve unsaved assignment drafts when refreshing; verify direct links load the requested month without changing dashboard context.
- [x] 2.2 Present category usage/overuse and explicit unavailable states for income and non-rolling-negative totals until authoritative fields exist; verify exact values remain decimal-safe and incomplete categories have no misleading percentage.

## 3. Account review

- [x] 3.1 Add account-filtered cursor paging to the transaction API while retaining complete transfer rows; verify unfiltered pagination remains unchanged and filtering matches either side of a transfer.
- [x] 3.2 Add the selected account's transaction list to account review with currency-separated balances; verify account switching updates only the selected account's activity.

## 4. Transaction review

- [x] 4.1 Replace transaction cards with an accessible sortable table containing the requested review columns and truthful unavailable labels; verify currency-specific in/out values and edit actions remain connected to the existing form.
- [x] 4.2 Add search over loaded rows, a clear loaded-page scope, and a visible add action; verify search/sort do not remove access to subsequent pages.

## 5. Integration

- [x] 5.1 Build the frontend and backend and resolve compile errors; verify the OpenSpec change passes validation and all task checkboxes reflect delivered behavior.
