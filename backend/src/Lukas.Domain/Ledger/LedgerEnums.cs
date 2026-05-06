namespace Lukas.Domain.Ledger;

public enum LedgerTransactionKind
{
    Income = 1,
    Expense = 2,
    Adjustment = 3,
}

public enum AccountEffect
{
    Increase = 1,
    Decrease = 2,
}
