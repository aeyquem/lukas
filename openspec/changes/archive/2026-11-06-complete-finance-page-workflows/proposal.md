# Proposal

## Why

The current web app has useful account, budget, and transaction workflows, but the navigation does not expose the pages in the owner's checklist and the home page does not summarize the available financial data. The existing pages also need more of the requested information and review controls so users can move between account, budget, and ledger facts without relying on placeholders.

## What Changes

- Add primary navigation and period context for dashboard, budget, accounts, transactions, payees, projections, and objectives.
- Turn the home route into a summary of authoritative data already returned by the API.
- Complete budget category readback with available-to-budget, period income, non-rolling negatives, uncategorized expense totals, per-category assignment/activity/availability, a usage indicator, and category transaction access where supported.
- Improve account selection and transaction review, retaining separate currency balances and existing account edit behavior.
- Present transactions in a searchable, sortable table with an add action and existing edit workflow.
- Add clear route destinations for payees, projections, and objectives; expose only data and actions backed by accepted API contracts.

## Capabilities

### New Capabilities

- `finance-workspace-navigation`: Navigation, period context, dashboard summaries, and explicit feature destinations.

### Modified Capabilities

- `monthly-budget`: Show the period and category figures needed to review the budget and reach its activity.
- `accounts`: Support account selection and account-scoped transaction review while preserving wallet balances.
- `transaction-ledger`: Provide searchable, sortable transaction table review and access to add/edit workflows.

## Impact

- Frontend Vue routes, shared application navigation, and dashboard/budget/accounts/transactions views.
- Existing account, transaction, bootstrap, and budget read APIs; no new financial calculations are moved into the browser.
- Payees, tags, reconciliation state, projections, objectives, and category rollover type remain unavailable until their data contracts and domain behavior are defined.
