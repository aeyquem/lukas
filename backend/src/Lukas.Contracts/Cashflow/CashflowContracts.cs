using Lukas.Contracts.Accounts;

namespace Lukas.Contracts.Cashflow;

public sealed record MonthlyCashflowDto(
    string Period,
    string Currency,
    string IncomeTotal,
    string ExpenseTotal,
    string NetCashflow,
    IReadOnlyList<AccountDto> Accounts);
