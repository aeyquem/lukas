using Lukas.Application.Ledger;
using Lukas.Infrastructure.Ledger;
using Lukas.Infrastructure.Persistence;
using Microsoft.Data.Sqlite;
using Microsoft.EntityFrameworkCore;
using Shouldly;

namespace Lukas.Infrastructure.Tests;

public sealed class EfLedgerRepositoryTests
{
    [Fact]
    public async Task OpeningBalanceCreatesAdjustmentAndAccountBalance()
    {
        await using var database = await CreateDatabaseAsync();
        var repository = new EfLedgerRepository(database.Context);

        var account = await repository.CreateAccountAsync(
            new CreateAccountCommand("Cuenta corriente", "bank", "CLP", "250000"),
            TestContext.Current.CancellationToken);

        var currentPeriod = DateTimeOffset.UtcNow.ToString("yyyy-MM");
        var transactions = await repository.GetTransactionsAsync(currentPeriod, TestContext.Current.CancellationToken);

        account.CurrentBalance.ShouldBe(250000m);
        transactions.ShouldContain(transaction =>
            transaction.Kind == "adjustment"
            && transaction.Amount == 250000m
            && transaction.AccountEffect == "increase");
    }

    [Fact]
    public async Task MonthlyCashflowUsesIncomeAndExpensesButNotAdjustments()
    {
        await using var database = await CreateDatabaseAsync();
        var repository = new EfLedgerRepository(database.Context);
        var account = await repository.CreateAccountAsync(
            new CreateAccountCommand("Banco", "bank", "CLP", "100000"),
            TestContext.Current.CancellationToken);

        await repository.CreateManualTransactionAsync(
            new CreateManualTransactionCommand("income", account.Id, new DateOnly(2026, 5, 1), "500000", "CLP", "increase", "Sueldo", null),
            TestContext.Current.CancellationToken);
        await repository.CreateManualTransactionAsync(
            new CreateManualTransactionCommand("expense", account.Id, new DateOnly(2026, 5, 2), "125000", "CLP", "decrease", "Arriendo", "Contrato anual"),
            TestContext.Current.CancellationToken);

        var cashflow = await repository.GetMonthlyCashflowAsync("2026-05", TestContext.Current.CancellationToken);

        cashflow.IncomeTotal.ShouldBe(500000m);
        cashflow.ExpenseTotal.ShouldBe(125000m);
        cashflow.NetCashflow.ShouldBe(375000m);
        cashflow.Accounts.Single().CurrentBalance.ShouldBe(475000m);
    }

    [Fact]
    public async Task GetMovementsIncludesAccountAndDirection()
    {
        await using var database = await CreateDatabaseAsync();
        var repository = new EfLedgerRepository(database.Context);
        var account = await repository.CreateAccountAsync(
            new CreateAccountCommand("Banco", "bank", "CLP", null),
            TestContext.Current.CancellationToken);

        await repository.CreateManualTransactionAsync(
            new CreateManualTransactionCommand("expense", account.Id, new DateOnly(2026, 5, 2), "125000", "CLP", "decrease", "Arriendo", "Contrato anual"),
            TestContext.Current.CancellationToken);

        var movements = await repository.GetMovementsAsync(TestContext.Current.CancellationToken);
        var movement = movements.Single();

        movement.AccountId.ShouldBe(account.Id);
        movement.AccountName.ShouldBe("Banco");
        movement.Direction.ShouldBe("out");
        movement.OriginalAmount.ShouldBe(125000m);
        movement.OriginalCurrency.ShouldBe("CLP");
        movement.Description.ShouldBe("Arriendo");
        movement.CategoryName.ShouldBeNull();
        movement.Notes.ShouldBe("Contrato anual");
    }

    private static async Task<TestDatabase> CreateDatabaseAsync()
    {
        var connection = new SqliteConnection("Data Source=:memory:");
        await connection.OpenAsync(TestContext.Current.CancellationToken);

        var options = new DbContextOptionsBuilder<LukasDbContext>()
            .UseSqlite(connection)
            .Options;
        var context = new LukasDbContext(options);
        await context.Database.EnsureCreatedAsync(TestContext.Current.CancellationToken);

        return new TestDatabase(connection, context);
    }

    private sealed class TestDatabase(SqliteConnection connection, LukasDbContext context) : IAsyncDisposable
    {
        public LukasDbContext Context { get; } = context;

        public async ValueTask DisposeAsync()
        {
            await Context.DisposeAsync();
            await connection.DisposeAsync();
        }
    }
}
