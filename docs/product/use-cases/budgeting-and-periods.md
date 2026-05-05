# Budgeting, periods and currency

Status: Accepted

## UC-007: View monthly budget and cashflow

The user sees income, spending, transfers, goals, reserves and remaining money
for a selected period. Reports distinguish actual, pending, projected, adjusted
and reconciled values when that distinction affects trust.

## UC-010: Handle multicurrency accounts and transactions

Accounts and transactions keep their real currency. Reports use a workspace base
currency, initially CLP, with conversion observations that explain the source
and lifecycle of each converted value.

The system must preserve institution estimates, final billed amounts, manual
corrections, reconciled values, provider corrections and fallback estimates when
they have different meanings.

## UC-011: Configure budget period

The user can work with calendar months first, while the model leaves room for
credit-card statement periods, payroll timing, cashflow periods and future tax
periods.

Period assignment must not rewrite the real transaction date.

## UC-012: Move transactions to another period

The user can assign a transaction to the period it should affect for budgeting
or reporting. The original date and source history remain intact.

Changes that affect reviewed or locked periods require explicit intent and must
leave a traceable adjustment or decision.

