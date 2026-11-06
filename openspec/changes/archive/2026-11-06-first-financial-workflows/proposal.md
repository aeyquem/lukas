# Proposal

## Why

The rewrite currently has a running Rust/PostgreSQL foundation and a static Vue shell, but the owner cannot yet record or inspect financial data. Implementing the reviewed first web slice now will connect accounts, manual activity, and monthly budget decisions through the real browser-to-PostgreSQL path.

## What Changes

- Add workspace-scoped account creation with one currency per bank, savings, or investment account, and multiple currency wallets for cash and card accounts; include dated signed openings and account reads/edits.
- Add manual income, expense, refund, and transfer entry, transaction reads, and supported ledger corrections.
- Add starter and owner-managed expense categories, current-month budget reads and assignments, deterministic funding/activity calculations, and explicit month close with immutable snapshots.
- Add Vue workflows with authoritative API-backed values, local drafts, clear validation, pagination, and exact retry behavior for unknown write outcomes.
- Keep the no-auth deployment same-machine only and preserve the accepted money, date, provenance, and correction rules.

## Capabilities

### New Capabilities

- `accounts`: Workspace-scoped account and wallet lifecycle, dated openings, and account reads/edits.
- `transaction-ledger`: Manual financial activity, linked transfers, transaction review, and supported corrections.
- `monthly-budget`: Current-month funding, assignments, category activity, incomplete values, and explicit chronological close snapshots.

### Modified Capabilities

None.

## Impact

- Backend: new PostgreSQL migrations, domain and application logic, persistence queries, request/response types, routes, idempotency handling, and integration coverage under `backend/`.
- Frontend: Vue Router views, a typed `fetch` client, query/cache integration, forms, budget editing and close interactions under `frontend/app/`.
- Runtime evidence: validate real PostgreSQL persistence and browser-to-API reads/writes through the existing local Compose proxy. No new deployment topology or authentication behavior is introduced.
