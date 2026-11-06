# Design

## Context

The frontend is Vue 3 with Vue Router and semantic HTML. The backend exposes accounts, a cursor-paginated transaction list, bootstrap categories, and monthly budget snapshots. Transaction views currently use cards; budget already exposes authoritative category assignment, activity, remaining, uncategorized activity, and ready-to-assign. The current period control is decorative, budget period state is local, and payees/projections/objectives have no API contracts.

## Goals / Non-Goals

**Goals:**

- Reuse current API values and exact decimal strings for summaries.
- Keep dashboard month state separate from the budget's month state.
- Add a wallet/account filter to transaction paging for the selected account.
- Make table search/sort behavior clear about its loaded-page scope.
- Keep feature destinations navigable even when their data workflows are not yet available.

**Non-Goals:**

- Add financial calculations to the browser or invent values for missing API fields.
- Define payee aliases, tags, reconciliation behavior, category rollover semantics, projections, or objectives.
- Change account, transaction, budget persistence or correction rules.

## Decisions

- Keep the primary navigation in `App.vue`; use Vue Router route metadata for page labels and active-page state. Link payees, projections, and objectives to the existing unavailable-feature view until their product and API contracts exist.
- Use query state for the dashboard month and budget month independently. The dashboard reads `/api/accounts`, `/api/budgets/{period}`, and the first `/api/transactions` page; each section has independent loading/error presentation. It displays only server-provided money.
- Extend transaction list query parsing with an optional account identifier and filter through wallet effects scoped to that account. Preserve the existing date/id cursor ordering and page-size limit.
- Convert `TransactionList.vue` to a semantic table and retain its typed edit event. Search and sorting are derived from the currently loaded page; loading further pages appends results, and the table labels this scope.
- Keep category and account editing forms as owned drafts in their existing views. Use computed values for derived displays, and do not bind editable values to fetched API snapshots.
- Show income and non-rolling-negative summary slots as unavailable until an authoritative API field and rollover policy exist. Do not estimate those values from incomplete or truncated transaction pages.

Alternatives considered: a global selected month would couple dashboard and budget periods, contrary to their independent semantics; fetching every transaction to support global browser search would defeat cursor pagination; adding tags, reconciliation, payee aliases, or rollover types now would require new domain and persistence contracts beyond this page-focused change.

## Risks / Trade-offs

- [Dashboard requests can partially fail] → Render each section's loading/error state independently and never replace a failed response with zero.
- [Search/sort apply only to fetched rows] → State the loaded-row scope and keep the existing load-more control available.
- [Account filter can omit transfer rows if filtered by only one effect] → Match any wallet belonging to the selected account and retain the complete logical transaction, including both transfer effects.
- [Several checklist fields do not exist in the API] → Label them unavailable and leave their implementation to a separately reviewed domain/API change.

## Migration Plan

No database migration is required. The transaction list query gains an optional parameter, so existing callers retain their current behavior. The frontend can roll back independently to the current card list and route state.

## Open Questions

Payee alias rules, tag management, reconciliation lifecycle, category rollover policy, projection assumptions, objective funding behavior, and profile identity remain separate product decisions.
