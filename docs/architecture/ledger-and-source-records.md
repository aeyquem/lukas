# Ledger and source records

Status: Accepted

The ledger model separates imported or manually entered source facts from
user-facing financial movements.

## Core concepts

- `source_records`: raw or parsed facts from manual entry, CSV, Excel or future
  integrations.
- `ledger_transactions`: normalized financial movements used by balances,
  reports, goals and projections.
- `transaction_source_links`: links between one logical transaction and one or
  more source records.
- `import_batches`: each file import or sync attempt.
- `review_items`: duplicate, ambiguous, mutation or discard workflows.
- `goals` and `reserves`: logical allocation of money, not real money movement.

Source records are immutable facts, not workflow state containers. Do not use a
canonical `source_records.status` field as domain truth. State such as linked,
discarded, superseded or needs review is derived from identifiers, decisions,
supersession records, source links and review items.

Manual transactions should also create lightweight manual source records. This
keeps provenance consistent across manual entry, file import and future API
imports.

## Import identity

File imports must be idempotent across repeated and overlapping files. Imported
source records should persist:

- `strong_fingerprint`: account, financial source and trustworthy external ID.
- `weak_fingerprint`: account, date, amount, currency, normalized description,
  counterparty and source context.
- `raw_snapshot`: original row, sheet, headers, provider payload or manual input.
- `normalized_snapshot`: Lukas's parsed interpretation before ledger creation.

Fingerprints are matching signals, not user-facing identity. They support
deduplication, tombstones, discard decisions and provider mutation detection.

CSV and Excel imports map to one account before row parsing. If a file contains
multiple accounts, the user should split it into separate files or sheets for
the PoC.

## Ledger entries

Ledger entries use positive amounts plus explicit account effects:

```text
ledger_transaction
  kind: expense | income | transfer | adjustment | allocation | refund | fee

ledger_entry
  account_id
  amount_value
  currency
  account_effect: increase | decrease
```

This keeps business meaning separate from account-balance effect. A credit-card
purchase is still an expense even though it increases a liability account.

Tables should include a `workspace_id` or equivalent boundary from the
beginning, even while the app is personal-first.

