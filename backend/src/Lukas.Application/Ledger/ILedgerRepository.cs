namespace Lukas.Application.Ledger;

public interface ILedgerRepository
{
    Task<AccountReadModel> CreateAccountAsync(CreateAccountCommand command, CancellationToken cancellationToken);

    Task<IReadOnlyList<AccountReadModel>> GetAccountsAsync(CancellationToken cancellationToken);

    Task<IReadOnlyList<TagReadModel>> GetTagsAsync(CancellationToken cancellationToken);

    Task<TransactionReadModel> CreateManualTransactionAsync(
        CreateManualTransactionCommand command,
        CancellationToken cancellationToken);

    Task<TransactionReadModel> UpdateManualTransactionAsync(
        UpdateManualTransactionCommand command,
        CancellationToken cancellationToken);

    Task<IReadOnlyList<MovementReadModel>> GetMovementsAsync(CancellationToken cancellationToken);

    Task<BudgetTreeReadModel> GetBudgetTreeAsync(MonthlyPeriod period, CancellationToken cancellationToken);

    Task<BudgetTreeReadModel> SaveBudgetTreeAsync(
        MonthlyPeriod period,
        SaveBudgetTreeCommand command,
        CancellationToken cancellationToken);

    Task<IReadOnlyList<TransactionReadModel>> GetTransactionsAsync(MonthlyPeriod period, CancellationToken cancellationToken);

    Task<MonthlyCashflowReadModel> GetMonthlyCashflowAsync(MonthlyPeriod period, CancellationToken cancellationToken);

    Task<MonthlyTagReportReadModel> GetMonthlyTagReportAsync(MonthlyPeriod period, CancellationToken cancellationToken);
}
