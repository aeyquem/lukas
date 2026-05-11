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

    [Fact]
    public async Task MonthlyPeriodExposesCalendarMonthRange()
    {
        var parsed = MonthlyPeriod.TryParse("2026-12", out var period);

        parsed.ShouldBeTrue();
        period.Key.ShouldBe("2026-12");
        period.StartDate.ShouldBe(new DateOnly(2026, 12, 1));
        period.EndDate.ShouldBe(new DateOnly(2027, 1, 1));
    }

    [Fact]
    public async Task UpdateTransactionRejectsInvalidAmount()
    {
        var service = new LedgerService(new ThrowingLedgerRepository());

        var result = await service.UpdateManualTransactionAsync(
            new UpdateManualTransactionCommand(
                Guid.CreateVersion7(),
                "expense",
                Guid.CreateVersion7(),
                null,
                new DateOnly(2026, 5, 2),
                "0",
                "CLP",
                "decrease",
                "Arriendo",
                null,
                null,
                null),
            TestContext.Current.CancellationToken);

        result.IsFailure.ShouldBeTrue();
        result.Error.Code.ShouldBe("money.invalid_amount");
    }

    [Fact]
    public async Task CreateTransactionRejectsDuplicateNormalizedTags()
    {
        var service = new LedgerService(new ThrowingLedgerRepository());

        var result = await service.CreateManualTransactionAsync(
            new CreateManualTransactionCommand(
                "expense",
                Guid.CreateVersion7(),
                null,
                new DateOnly(2026, 5, 2),
                "1000",
                "CLP",
                "decrease",
                "Micros",
                null,
                null,
                null,
                ["Transport", "tranSPort "]),
            TestContext.Current.CancellationToken);

        result.IsFailure.ShouldBeTrue();
        result.Error.Code.ShouldBe("tag.duplicate");
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

        public Task<IReadOnlyList<TagReadModel>> GetTagsAsync(CancellationToken cancellationToken)
        {
            throw new InvalidOperationException("Repository should not be called.");
        }

        public Task<TransactionReadModel> CreateManualTransactionAsync(
            CreateManualTransactionCommand command,
            CancellationToken cancellationToken)
        {
            throw new InvalidOperationException("Repository should not be called.");
        }

        public Task<TransactionReadModel> UpdateManualTransactionAsync(
            UpdateManualTransactionCommand command,
            CancellationToken cancellationToken)
        {
            throw new InvalidOperationException("Repository should not be called.");
        }

        public Task<IReadOnlyList<MovementReadModel>> GetMovementsAsync(CancellationToken cancellationToken)
        {
            throw new InvalidOperationException("Repository should not be called.");
        }

        public Task<BudgetTreeReadModel> GetBudgetTreeAsync(
            MonthlyPeriod period,
            CancellationToken cancellationToken)
        {
            throw new InvalidOperationException("Repository should not be called.");
        }

        public Task<BudgetTreeReadModel> SaveBudgetTreeAsync(
            MonthlyPeriod period,
            SaveBudgetTreeCommand command,
            CancellationToken cancellationToken)
        {
            throw new InvalidOperationException("Repository should not be called.");
        }

        public Task<IReadOnlyList<TransactionReadModel>> GetTransactionsAsync(
            MonthlyPeriod period,
            CancellationToken cancellationToken)
        {
            throw new InvalidOperationException("Repository should not be called.");
        }

        public Task<MonthlyCashflowReadModel> GetMonthlyCashflowAsync(
            MonthlyPeriod period,
            CancellationToken cancellationToken)
        {
            throw new InvalidOperationException("Repository should not be called.");
        }

        public Task<MonthlyTagReportReadModel> GetMonthlyTagReportAsync(
            MonthlyPeriod period,
            CancellationToken cancellationToken)
        {
            throw new InvalidOperationException("Repository should not be called.");
        }
    }
}
