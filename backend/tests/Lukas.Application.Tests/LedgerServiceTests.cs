using Lukas.Application.Ledger;
using Shouldly;

namespace Lukas.Application.Tests;

public sealed class LedgerServiceTests
{
    [Fact]
    public async Task CreateAccountRejectsInvalidCurrency()
    {
        var service = new LedgerService(new ThrowingLedgerRepository());

        var result = await service.CreateAccountAsync(
            new CreateAccountCommand("Cuenta", "bank", "clp", null),
            TestContext.Current.CancellationToken);

        result.IsFailure.ShouldBeTrue();
        result.Error.Code.ShouldBe("money.invalid_currency");
    }

    [Fact]
    public async Task GetTransactionsRejectsInvalidPeriod()
    {
        var service = new LedgerService(new ThrowingLedgerRepository());

        var result = await service.GetTransactionsAsync("2026-99", TestContext.Current.CancellationToken);

        result.IsFailure.ShouldBeTrue();
        result.Error.Code.ShouldBe("period.invalid");
    }

    private sealed class ThrowingLedgerRepository : ILedgerRepository
    {
        public Task<AccountReadModel> CreateAccountAsync(
            CreateAccountCommand command,
            CancellationToken cancellationToken)
        {
            throw new InvalidOperationException("Repository should not be called.");
        }

        public Task<IReadOnlyList<AccountReadModel>> GetAccountsAsync(CancellationToken cancellationToken)
        {
            throw new InvalidOperationException("Repository should not be called.");
        }

        public Task<TransactionReadModel> CreateManualTransactionAsync(
            CreateManualTransactionCommand command,
            CancellationToken cancellationToken)
        {
            throw new InvalidOperationException("Repository should not be called.");
        }

        public Task<IReadOnlyList<TransactionReadModel>> GetTransactionsAsync(
            string period,
            CancellationToken cancellationToken)
        {
            throw new InvalidOperationException("Repository should not be called.");
        }

        public Task<MonthlyCashflowReadModel> GetMonthlyCashflowAsync(
            string period,
            CancellationToken cancellationToken)
        {
            throw new InvalidOperationException("Repository should not be called.");
        }
    }
}
