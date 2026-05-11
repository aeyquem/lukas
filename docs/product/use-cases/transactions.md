# Transactions and reconciliation

Status: Accepted

## UC-001: Create manual transaction

The user records income, expense, transfer, fee, refund or adjustment activity
without needing an import. The system stores the transaction with source
provenance, amount, currency, account, date, description and optional category.

Manual transactions also create a lightweight source record so manual and
imported activity share the same history model.

## UC-002: Edit and categorize transactions

The user can correct descriptions, categories, notes, dates, account assignment
and business meaning without losing the original source context.

Corrections that affect reviewed or locked periods require explicit user intent
and must remain traceable.

Transactions can also carry optional tags. Tags are workspace-managed labels
that can be added while editing a transaction. Tag matching is normalized so
case, extra spaces and word separators do not create duplicate meanings.
Categories remain budget-oriented; tags support cross-cutting review and
reporting.

## UC-013: Register transfers between owned accounts

The user records movement between their own accounts as one logical transfer
with the required ledger entries. Transfers must avoid double-counting income or
expense in reports.

Cross-currency transfers keep both account currencies and any conversion
observations available from the source, user or fallback provider.

## UC-014: Handle cash and withdrawals

The user can model cash as an account and represent ATM withdrawals or cash
usage without mixing cash spending with bank movement.

Cash withdrawals are transfers from a bank account to a cash account. Cash
spending is recorded separately when the user wants that detail.

## UC-018: Review and resolve duplicate transactions

The system detects likely duplicates from repeated imports, overlapping files,
provider updates or manual/import overlap.

The user can merge, keep separate, ignore or discard candidates. The system
keeps the source records, matching reason, date and decision history.

## UC-019: Reconcile accounts

The user compares calculated account balances against real institution balances
or statements. Differences must be explainable through missing transactions,
duplicates, pending activity, conversion changes or explicit adjustments.

Reconciliation should not hide mismatches by silently rewriting history.
