# Reports and external access

Status: Accepted

## Reports

Priority reports for the PoC:

- Monthly budget and cashflow.
- Spending by category.
- Account balances.
- Goals and reserves.
- Import/review status.
- Basic projection or runway.

Additional reports are deferred until the ledger, imports and period behavior
are stable.

## UC-020: Prepare Chile annual tax information

Chile annual tax preparation is a future module or adjacent project, not part of
the first core budgeting/ledger implementation.

The feature should read from ledger transactions, source records, snapshots and
reports while preserving source-document traceability. Output must be reviewable
and should not be presented as official tax advice.

## UC-021: Expose public API or MCP

Public API and MCP access are deferred from the initial desktop PoC. Future
external access should start read-only and use scoped permissions.

External tools must not receive bank credentials, provider tokens or raw
integration secrets. Any future write workflow must be explicit, auditable and
reversible at the product level.

