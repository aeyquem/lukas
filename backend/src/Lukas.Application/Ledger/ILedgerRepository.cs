namespace Lukas.Application.Ledger;

public interface ILedgerRepository
{
    Task<AccountReadModel> CreateAccountAsync(CreateAccountCommand command, CancellationToken cancellationToken);

    Task<IReadOnlyList<AccountReadModel>> GetAccountsAsync(CancellationToken cancellationToken);

    Task<TransactionReadModel> CreateManualTransactionAsync(
        CreateManualTransactionCommand command,
        CancellationToken cancellationToken);

    Task<IReadOnlyList<TransactionReadModel>> GetTransactionsAsync(string period, CancellationToken cancellationToken);

    Task<MonthlyCashflowReadModel> GetMonthlyCashflowAsync(string period, CancellationToken cancellationToken);
}
