# Currency and periods

Status: Accepted

Money, exchange rates and period assignment are correctness-critical. The model
must avoid floating point math, implicit timezone conversion and a single
transaction date that tries to serve every report.

## Money conversions

Conversions are observations attached to ledger entries, not one mutable
converted amount. A ledger entry can have multiple conversion observations for
different lifecycle states:

- institution pending estimate.
- final billed or paid value.
- manual correction.
- reconciled value.
- provider correction.
- fallback historical estimate.

Reports choose the best conversion for their context. Final, manual or
reconciled conversions outrank pending or fallback estimates. If a fallback
estimate is used for a past transaction, store it so reports stay repeatable.

Fees, spreads, taxes and card charges should be separate ledger entries when the
source exposes them separately. If the source only provides a final charged
amount, the app should not invent separate fee rows.

## Periods

A transaction may belong to several period contexts: calendar period, budget
period, credit-card statement period, cashflow period or future tax period. A
transaction's real date should not be rewritten when it is assigned to a
different period context.

The default budget view can use calendar months starting on the first day of the
month. Source-specific rules remain necessary for credit cards, salary timing,
recurring income and future tax reporting.

Period locking is optional for the PoC. Its purpose is preserving a reviewed
historical baseline for comparison against plans, projections and later
corrections.

