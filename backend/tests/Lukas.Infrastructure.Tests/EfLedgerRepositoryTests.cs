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
        var transactions = await repository.GetTransactionsAsync(Period(currentPeriod), TestContext.Current.CancellationToken);

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
            new CreateManualTransactionCommand("income", account.Id, null, new DateOnly(2026, 5, 1), "500000", "CLP", "increase", "Sueldo", null, "Ingresos", null),
            TestContext.Current.CancellationToken);
        await repository.CreateManualTransactionAsync(
            new CreateManualTransactionCommand("expense", account.Id, null, new DateOnly(2026, 5, 2), "125000", "CLP", "decrease", "Arriendo", null, "Casa", "Contrato anual"),
            TestContext.Current.CancellationToken);

        var cashflow = await repository.GetMonthlyCashflowAsync(Period("2026-05"), TestContext.Current.CancellationToken);

        cashflow.IncomeTotal.ShouldBe(500000m);
        cashflow.ExpenseTotal.ShouldBe(125000m);
        cashflow.NetCashflow.ShouldBe(375000m);
        cashflow.Accounts.Single().CurrentBalance.ShouldBe(475000m);
    }

    [Fact]
    public async Task TransactionsUseCalendarMonthBoundaries()
    {
        await using var database = await CreateDatabaseAsync();
        var repository = new EfLedgerRepository(database.Context);
        var account = await repository.CreateAccountAsync(
            new CreateAccountCommand("Banco", "bank", "CLP", null),
            TestContext.Current.CancellationToken);

        await repository.CreateManualTransactionAsync(
            new CreateManualTransactionCommand("expense", account.Id, null, new DateOnly(2026, 4, 30), "1000", "CLP", "decrease", "Abril", null, null, null),
            TestContext.Current.CancellationToken);
        await repository.CreateManualTransactionAsync(
            new CreateManualTransactionCommand("expense", account.Id, null, new DateOnly(2026, 5, 1), "2000", "CLP", "decrease", "Inicio mayo", null, null, null),
            TestContext.Current.CancellationToken);
        await repository.CreateManualTransactionAsync(
            new CreateManualTransactionCommand("expense", account.Id, null, new DateOnly(2026, 5, 31), "3000", "CLP", "decrease", "Fin mayo", null, null, null),
            TestContext.Current.CancellationToken);
        await repository.CreateManualTransactionAsync(
            new CreateManualTransactionCommand("expense", account.Id, null, new DateOnly(2026, 6, 1), "4000", "CLP", "decrease", "Junio", null, null, null),
            TestContext.Current.CancellationToken);

        var transactions = await repository.GetTransactionsAsync(Period("2026-05"), TestContext.Current.CancellationToken);

        transactions.Select(transaction => transaction.Description).ShouldBe(["Fin mayo", "Inicio mayo"]);
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
            new CreateManualTransactionCommand("expense", account.Id, null, new DateOnly(2026, 5, 2), "125000", "CLP", "decrease", "Arriendo", null, "Casa", "Contrato anual"),
            TestContext.Current.CancellationToken);

        var movements = await repository.GetMovementsAsync(TestContext.Current.CancellationToken);
        var movement = movements.Single();

        movement.AccountId.ShouldBe(account.Id);
        movement.AccountName.ShouldBe("Banco");
        movement.Kind.ShouldBe("expense");
        movement.AccountEffect.ShouldBe("decrease");
        movement.Direction.ShouldBe("out");
        movement.OriginalAmount.ShouldBe(125000m);
        movement.OriginalCurrency.ShouldBe("CLP");
        movement.Description.ShouldBe("Arriendo");
        movement.CategoryName.ShouldBe("Casa");
        movement.Notes.ShouldBe("Contrato anual");
    }

    [Fact]
    public async Task UpdateManualTransactionChangesMovementAndBalance()
    {
        await using var database = await CreateDatabaseAsync();
        var repository = new EfLedgerRepository(database.Context);
        var account = await repository.CreateAccountAsync(
            new CreateAccountCommand("Banco", "bank", "CLP", null),
            TestContext.Current.CancellationToken);

        var transaction = await repository.CreateManualTransactionAsync(
            new CreateManualTransactionCommand("expense", account.Id, null, new DateOnly(2026, 5, 2), "125000", "CLP", "decrease", "Arriendo", null, null, null),
            TestContext.Current.CancellationToken);

        await repository.UpdateManualTransactionAsync(
            new UpdateManualTransactionCommand(transaction.Id, "income", account.Id, null, new DateOnly(2026, 5, 3), "150000", "CLP", "increase", "Reembolso", null, "Ingresos", "Corregido"),
            TestContext.Current.CancellationToken);

        var movements = await repository.GetMovementsAsync(TestContext.Current.CancellationToken);
        var accounts = await repository.GetAccountsAsync(TestContext.Current.CancellationToken);
        var movement = movements.Single();

        movement.TransactionId.ShouldBe(transaction.Id);
        movement.TransactionDate.ShouldBe(new DateOnly(2026, 5, 3));
        movement.Kind.ShouldBe("income");
        movement.AccountEffect.ShouldBe("increase");
        movement.Direction.ShouldBe("in");
        movement.OriginalAmount.ShouldBe(150000m);
        movement.Description.ShouldBe("Reembolso");
        movement.CategoryName.ShouldBe("Ingresos");
        movement.Notes.ShouldBe("Corregido");
        accounts.Single().CurrentBalance.ShouldBe(150000m);
    }

    [Fact]
    public async Task TransactionTagsAreCreatedNormalizedAndReturnedWithMovements()
    {
        await using var database = await CreateDatabaseAsync();
        var repository = new EfLedgerRepository(database.Context);
        var account = await repository.CreateAccountAsync(
            new CreateAccountCommand("Banco", "bank", "CLP", null),
            TestContext.Current.CancellationToken);

        await repository.CreateManualTransactionAsync(
            new CreateManualTransactionCommand("expense", account.Id, null, new DateOnly(2026, 5, 2), "1500", "CLP", "decrease", "Micro", null, null, null, ["Transport", "Home-Party"]),
            TestContext.Current.CancellationToken);
        await repository.CreateManualTransactionAsync(
            new CreateManualTransactionCommand("expense", account.Id, null, new DateOnly(2026, 5, 3), "2000", "CLP", "decrease", "Metro", null, null, null, ["tranSPort "]),
            TestContext.Current.CancellationToken);

        var tags = await repository.GetTagsAsync(TestContext.Current.CancellationToken);
        var movements = await repository.GetMovementsAsync(TestContext.Current.CancellationToken);

        tags.Count.ShouldBe(2);
        tags.ShouldContain(tag => tag.Name == "Transport" && tag.NormalizedName == "transport");
        tags.ShouldContain(tag => tag.Name == "Home Party" && tag.NormalizedName == "home party");
        movements.ShouldContain(movement => movement.Description == "Micro" && movement.Tags.Count == 2);
        movements.ShouldContain(movement => movement.Description == "Metro" && movement.Tags.Single().NormalizedName == "transport");
    }

    [Fact]
    public async Task TagReportAggregatesIncomeAndExpenseWithoutTransfers()
    {
        await using var database = await CreateDatabaseAsync();
        var repository = new EfLedgerRepository(database.Context);
        var checking = await repository.CreateAccountAsync(
            new CreateAccountCommand("Cuenta corriente", "bank", "CLP", null),
            TestContext.Current.CancellationToken);
        var savings = await repository.CreateAccountAsync(
            new CreateAccountCommand("Ahorro", "bank", "CLP", null),
            TestContext.Current.CancellationToken);

        await repository.CreateManualTransactionAsync(
            new CreateManualTransactionCommand("income", checking.Id, null, new DateOnly(2026, 5, 1), "500000", "CLP", "increase", "Sueldo", null, null, null, ["Trabajo"]),
            TestContext.Current.CancellationToken);
        await repository.CreateManualTransactionAsync(
            new CreateManualTransactionCommand("expense", checking.Id, null, new DateOnly(2026, 5, 2), "12000", "CLP", "decrease", "Bus", null, null, null, ["Transport"]),
            TestContext.Current.CancellationToken);
        await repository.CreateManualTransactionAsync(
            new CreateManualTransactionCommand("transfer", checking.Id, savings.Id, new DateOnly(2026, 5, 3), "30000", "CLP", "decrease", "Ahorro", null, null, null, ["Transport"]),
            TestContext.Current.CancellationToken);

        var report = await repository.GetMonthlyTagReportAsync(Period("2026-05"), TestContext.Current.CancellationToken);
        var transport = report.Tags.Single(row => row.Tag.NormalizedName == "transport");
        var work = report.Tags.Single(row => row.Tag.NormalizedName == "trabajo");

        transport.ExpenseTotal.ShouldBe(12000m);
        transport.IncomeTotal.ShouldBe(0m);
        transport.TransactionCount.ShouldBe(1);
        work.IncomeTotal.ShouldBe(500000m);
    }

    [Fact]
    public async Task TransferCreatesOutAndInMovementsWithoutCashflowImpact()
    {
        await using var database = await CreateDatabaseAsync();
        var repository = new EfLedgerRepository(database.Context);
        var checking = await repository.CreateAccountAsync(
            new CreateAccountCommand("Cuenta corriente", "bank", "CLP", "100000"),
            TestContext.Current.CancellationToken);
        var savings = await repository.CreateAccountAsync(
            new CreateAccountCommand("Ahorro", "bank", "CLP", null),
            TestContext.Current.CancellationToken);

        await repository.CreateManualTransactionAsync(
            new CreateManualTransactionCommand("transfer", checking.Id, savings.Id, new DateOnly(2026, 5, 4), "30000", "CLP", "decrease", "Ahorro mensual", null, null, null),
            TestContext.Current.CancellationToken);

        var accounts = await repository.GetAccountsAsync(TestContext.Current.CancellationToken);
        var cashflow = await repository.GetMonthlyCashflowAsync(Period("2026-05"), TestContext.Current.CancellationToken);
        var movements = await repository.GetMovementsAsync(TestContext.Current.CancellationToken);
        var budgetTree = await repository.GetBudgetTreeAsync(Period("2026-05"), TestContext.Current.CancellationToken);

        accounts.Single(account => account.Id == checking.Id).CurrentBalance.ShouldBe(70000m);
        accounts.Single(account => account.Id == savings.Id).CurrentBalance.ShouldBe(30000m);
        cashflow.IncomeTotal.ShouldBe(0m);
        cashflow.ExpenseTotal.ShouldBe(0m);
        budgetTree.Envelopes.ShouldNotContain(envelope => envelope.Name == "Transferencia");
        movements.Count(movement => movement.Kind == "transfer").ShouldBe(2);
        movements.ShouldContain(movement =>
            movement.AccountId == checking.Id
            && movement.DestinationAccountId == savings.Id
            && movement.CategoryName == "Transferencia"
            && movement.Direction == "out");
        movements.ShouldContain(movement =>
            movement.AccountId == savings.Id
            && movement.DestinationAccountId == checking.Id
            && movement.CategoryName == "Transferencia"
            && movement.Direction == "in");
    }

    [Fact]
    public async Task BudgetTreePromotesExistingMovementCategoriesToEnvelopes()
    {
        await using var database = await CreateDatabaseAsync();
        var repository = new EfLedgerRepository(database.Context);
        var account = await repository.CreateAccountAsync(
            new CreateAccountCommand("Banco", "bank", "CLP", null),
            TestContext.Current.CancellationToken);

        await repository.CreateManualTransactionAsync(
            new CreateManualTransactionCommand("expense", account.Id, null, new DateOnly(2026, 5, 2), "125000", "CLP", "decrease", "Arriendo", null, "Arriendo", null),
            TestContext.Current.CancellationToken);

        var tree = await repository.GetBudgetTreeAsync(Period("2026-05"), TestContext.Current.CancellationToken);
        var movements = await repository.GetMovementsAsync(TestContext.Current.CancellationToken);
        var envelope = tree.Envelopes.Single(candidate => candidate.Name == "Arriendo");

        movements.Single().EnvelopeId.ShouldBe(envelope.Id);
        movements.Single().EnvelopeName.ShouldBe("Arriendo");
    }

    [Fact]
    public async Task BudgetTreeIncludesSelectedPeriodActivity()
    {
        await using var database = await CreateDatabaseAsync();
        var repository = new EfLedgerRepository(database.Context);
        var account = await repository.CreateAccountAsync(
            new CreateAccountCommand("Banco", "bank", "CLP", null),
            TestContext.Current.CancellationToken);

        await repository.CreateManualTransactionAsync(
            new CreateManualTransactionCommand("income", account.Id, null, new DateOnly(2026, 5, 1), "500000", "CLP", "increase", "Sueldo", null, null, null),
            TestContext.Current.CancellationToken);
        await repository.CreateManualTransactionAsync(
            new CreateManualTransactionCommand("expense", account.Id, null, new DateOnly(2026, 5, 2), "125000", "CLP", "decrease", "Arriendo", null, "Arriendo", null),
            TestContext.Current.CancellationToken);
        await repository.CreateManualTransactionAsync(
            new CreateManualTransactionCommand("expense", account.Id, null, new DateOnly(2026, 6, 2), "90000", "CLP", "decrease", "Arriendo junio", null, "Arriendo", null),
            TestContext.Current.CancellationToken);

        var tree = await repository.GetBudgetTreeAsync(Period("2026-05"), TestContext.Current.CancellationToken);
        var incomeEnvelope = tree.Envelopes.Single(candidate => candidate.Name == "Ingresos");
        var rentEnvelope = tree.Envelopes.Single(candidate => candidate.Name == "Arriendo");

        incomeEnvelope.ActivityAmount.ShouldBe(500000m);
        rentEnvelope.ActivityAmount.ShouldBe(125000m);
        rentEnvelope.RemainingAmount.ShouldBe(-125000m);
    }

    [Fact]
    public async Task IncomeWithoutEnvelopeUsesSystemIncomeEnvelope()
    {
        await using var database = await CreateDatabaseAsync();
        var repository = new EfLedgerRepository(database.Context);
        var account = await repository.CreateAccountAsync(
            new CreateAccountCommand("Banco", "bank", "CLP", null),
            TestContext.Current.CancellationToken);

        await repository.CreateManualTransactionAsync(
            new CreateManualTransactionCommand("income", account.Id, null, new DateOnly(2026, 5, 1), "500000", "CLP", "increase", "Sueldo", null, null, null),
            TestContext.Current.CancellationToken);

        var tree = await repository.GetBudgetTreeAsync(Period("2026-05"), TestContext.Current.CancellationToken);
        var movements = await repository.GetMovementsAsync(TestContext.Current.CancellationToken);
        var envelope = tree.Envelopes.Single(candidate => candidate.Name == "Ingresos");

        movements.Single().EnvelopeId.ShouldBe(envelope.Id);
        movements.Single().EnvelopeName.ShouldBe("Ingresos");
    }

    [Fact]
    public async Task IncomeEnvelopeCannotBeRemovedByBudgetSave()
    {
        await using var database = await CreateDatabaseAsync();
        var repository = new EfLedgerRepository(database.Context);
        var tree = await repository.GetBudgetTreeAsync(Period("2026-05"), TestContext.Current.CancellationToken);
        var incomeEnvelope = tree.Envelopes.Single(candidate => candidate.Name == "Ingresos");
        var withoutIncomeEnvelope = new SaveBudgetTreeCommand(
            tree.Categories.Select(category => new SaveBudgetCategoryCommand(category.Id, category.Name, category.SortOrder)).ToList(),
            tree.Subcategories.Select(subcategory => new SaveBudgetSubcategoryCommand(subcategory.Id, subcategory.CategoryId, subcategory.Name, subcategory.SortOrder)).ToList(),
            tree.Envelopes
                .Where(candidate => candidate.Id != incomeEnvelope.Id)
                .Select(candidate => new SaveBudgetEnvelopeCommand(candidate.Id, candidate.SubcategoryId, candidate.Name, candidate.AssignedAmount.ToString(System.Globalization.CultureInfo.InvariantCulture), candidate.SortOrder))
                .ToList(),
            tree.Goals.Select(goal => new SaveBudgetGoalCommand(goal.Id, goal.EnvelopeId, goal.TargetAmount.ToString(System.Globalization.CultureInfo.InvariantCulture), goal.DeadlineDate, goal.Status)).ToList());

        var savedTree = await repository.SaveBudgetTreeAsync(Period("2026-05"), withoutIncomeEnvelope, TestContext.Current.CancellationToken);

        savedTree.Envelopes.ShouldContain(envelope => envelope.Id == incomeEnvelope.Id);
    }

    [Fact]
    public async Task DeletedBudgetEnvelopeMovesToTrashAndKeepsMovementsCategorized()
    {
        await using var database = await CreateDatabaseAsync();
        var repository = new EfLedgerRepository(database.Context);
        var account = await repository.CreateAccountAsync(
            new CreateAccountCommand("Banco", "bank", "CLP", null),
            TestContext.Current.CancellationToken);

        await repository.CreateManualTransactionAsync(
            new CreateManualTransactionCommand("expense", account.Id, null, new DateOnly(2026, 5, 2), "125000", "CLP", "decrease", "Arriendo", null, "Arriendo", null),
            TestContext.Current.CancellationToken);

        var tree = await repository.GetBudgetTreeAsync(Period("2026-05"), TestContext.Current.CancellationToken);
        var envelope = tree.Envelopes.Single(candidate => candidate.Name == "Arriendo");
        var withoutEnvelope = new SaveBudgetTreeCommand(
            tree.Categories.Select(category => new SaveBudgetCategoryCommand(category.Id, category.Name, category.SortOrder)).ToList(),
            tree.Subcategories.Select(subcategory => new SaveBudgetSubcategoryCommand(subcategory.Id, subcategory.CategoryId, subcategory.Name, subcategory.SortOrder)).ToList(),
            tree.Envelopes
                .Where(candidate => candidate.Id != envelope.Id)
                .Select(candidate => new SaveBudgetEnvelopeCommand(candidate.Id, candidate.SubcategoryId, candidate.Name, candidate.AssignedAmount.ToString(System.Globalization.CultureInfo.InvariantCulture), candidate.SortOrder))
                .ToList(),
            tree.Goals.Select(goal => new SaveBudgetGoalCommand(goal.Id, goal.EnvelopeId, goal.TargetAmount.ToString(System.Globalization.CultureInfo.InvariantCulture), goal.DeadlineDate, goal.Status)).ToList());

        var savedTree = await repository.SaveBudgetTreeAsync(Period("2026-05"), withoutEnvelope, TestContext.Current.CancellationToken);
        var movements = await repository.GetMovementsAsync(TestContext.Current.CancellationToken);
        var trashSubcategory = savedTree.Subcategories.Single(candidate => candidate.Name == "Categorías eliminadas");
        var trashedEnvelope = savedTree.Envelopes.Single(candidate => candidate.Id == envelope.Id);

        trashedEnvelope.SubcategoryId.ShouldBe(trashSubcategory.Id);
        movements.Single().EnvelopeId.ShouldBe(envelope.Id);
        movements.Single().EnvelopeName.ShouldBe("Arriendo");
    }

    [Fact]
    public async Task TrashedBudgetEnvelopeCannotBeRemovedByBudgetSave()
    {
        await using var database = await CreateDatabaseAsync();
        var repository = new EfLedgerRepository(database.Context);
        var account = await repository.CreateAccountAsync(
            new CreateAccountCommand("Banco", "bank", "CLP", null),
            TestContext.Current.CancellationToken);

        await repository.CreateManualTransactionAsync(
            new CreateManualTransactionCommand("expense", account.Id, null, new DateOnly(2026, 5, 2), "125000", "CLP", "decrease", "Arriendo", null, "Arriendo", null),
            TestContext.Current.CancellationToken);

        var tree = await repository.GetBudgetTreeAsync(Period("2026-05"), TestContext.Current.CancellationToken);
        var envelope = tree.Envelopes.Single(candidate => candidate.Name == "Arriendo");
        var withoutEnvelope = new SaveBudgetTreeCommand(
            tree.Categories.Select(category => new SaveBudgetCategoryCommand(category.Id, category.Name, category.SortOrder)).ToList(),
            tree.Subcategories.Select(subcategory => new SaveBudgetSubcategoryCommand(subcategory.Id, subcategory.CategoryId, subcategory.Name, subcategory.SortOrder)).ToList(),
            tree.Envelopes
                .Where(candidate => candidate.Id != envelope.Id)
                .Select(candidate => new SaveBudgetEnvelopeCommand(candidate.Id, candidate.SubcategoryId, candidate.Name, candidate.AssignedAmount.ToString(System.Globalization.CultureInfo.InvariantCulture), candidate.SortOrder))
                .ToList(),
            tree.Goals.Select(goal => new SaveBudgetGoalCommand(goal.Id, goal.EnvelopeId, goal.TargetAmount.ToString(System.Globalization.CultureInfo.InvariantCulture), goal.DeadlineDate, goal.Status)).ToList());

        var treeWithTrash = await repository.SaveBudgetTreeAsync(Period("2026-05"), withoutEnvelope, TestContext.Current.CancellationToken);
        var withoutTrashedEnvelope = new SaveBudgetTreeCommand(
            treeWithTrash.Categories.Select(category => new SaveBudgetCategoryCommand(category.Id, category.Name, category.SortOrder)).ToList(),
            treeWithTrash.Subcategories.Select(subcategory => new SaveBudgetSubcategoryCommand(subcategory.Id, subcategory.CategoryId, subcategory.Name, subcategory.SortOrder)).ToList(),
            treeWithTrash.Envelopes
                .Where(candidate => candidate.Id != envelope.Id)
                .Select(candidate => new SaveBudgetEnvelopeCommand(candidate.Id, candidate.SubcategoryId, candidate.Name, candidate.AssignedAmount.ToString(System.Globalization.CultureInfo.InvariantCulture), candidate.SortOrder))
                .ToList(),
            treeWithTrash.Goals.Select(goal => new SaveBudgetGoalCommand(goal.Id, goal.EnvelopeId, goal.TargetAmount.ToString(System.Globalization.CultureInfo.InvariantCulture), goal.DeadlineDate, goal.Status)).ToList());

        var savedTree = await repository.SaveBudgetTreeAsync(Period("2026-05"), withoutTrashedEnvelope, TestContext.Current.CancellationToken);
        var trashSubcategory = savedTree.Subcategories.Single(candidate => candidate.Name == "Categorías eliminadas");
        var trashedEnvelope = savedTree.Envelopes.Single(candidate => candidate.Id == envelope.Id);

        trashedEnvelope.SubcategoryId.ShouldBe(trashSubcategory.Id);
    }

    [Fact]
    public async Task OrphanedMovementEnvelopeIsRestoredToTrash()
    {
        await using var database = await CreateDatabaseAsync();
        var repository = new EfLedgerRepository(database.Context);
        var account = await repository.CreateAccountAsync(
            new CreateAccountCommand("Banco", "bank", "CLP", null),
            TestContext.Current.CancellationToken);

        await repository.CreateManualTransactionAsync(
            new CreateManualTransactionCommand("expense", account.Id, null, new DateOnly(2026, 5, 2), "125000", "CLP", "decrease", "Arriendo", null, "Arriendo", null),
            TestContext.Current.CancellationToken);

        var tree = await repository.GetBudgetTreeAsync(Period("2026-05"), TestContext.Current.CancellationToken);
        var envelope = tree.Envelopes.Single(candidate => candidate.Name == "Arriendo");
        await database.Context.Database.ExecuteSqlRawAsync(
            "PRAGMA foreign_keys = OFF;",
            TestContext.Current.CancellationToken);
        await database.Context.Database.ExecuteSqlInterpolatedAsync(
            $"DELETE FROM budget_envelopes WHERE id = {envelope.Id}",
            TestContext.Current.CancellationToken);
        await database.Context.Database.ExecuteSqlRawAsync(
            "PRAGMA foreign_keys = ON;",
            TestContext.Current.CancellationToken);
        database.Context.ChangeTracker.Clear();

        var movements = await repository.GetMovementsAsync(TestContext.Current.CancellationToken);
        var repairedTree = await repository.GetBudgetTreeAsync(Period("2026-05"), TestContext.Current.CancellationToken);
        var trashSubcategory = repairedTree.Subcategories.Single(candidate => candidate.Name == "Categorías eliminadas");
        var restoredEnvelope = repairedTree.Envelopes.Single(candidate => candidate.Id == envelope.Id);

        restoredEnvelope.SubcategoryId.ShouldBe(trashSubcategory.Id);
        restoredEnvelope.Name.ShouldBe("Arriendo");
        movements.Single().EnvelopeName.ShouldBe("Arriendo");
    }

    [Fact]
    public async Task BudgetTreePersistsGoalMetadataLinkedToEnvelope()
    {
        await using var database = await CreateDatabaseAsync();
        var repository = new EfLedgerRepository(database.Context);
        var tree = await repository.GetBudgetTreeAsync(Period("2026-05"), TestContext.Current.CancellationToken);
        var goalsSubcategory = tree.Subcategories.Single(candidate => candidate.Name == "Reservas y metas");
        var envelopeId = Guid.CreateVersion7();
        var goalId = Guid.CreateVersion7();
        var command = new SaveBudgetTreeCommand(
            tree.Categories.Select(category => new SaveBudgetCategoryCommand(category.Id, category.Name, category.SortOrder)).ToList(),
            tree.Subcategories.Select(subcategory => new SaveBudgetSubcategoryCommand(subcategory.Id, subcategory.CategoryId, subcategory.Name, subcategory.SortOrder)).ToList(),
            tree.Envelopes.Select(envelope => new SaveBudgetEnvelopeCommand(envelope.Id, envelope.SubcategoryId, envelope.Name, envelope.AssignedAmount.ToString(System.Globalization.CultureInfo.InvariantCulture), envelope.SortOrder))
                .Append(new SaveBudgetEnvelopeCommand(envelopeId, goalsSubcategory.Id, "Fondo emergencia", "100000", 1))
                .ToList(),
            [new SaveBudgetGoalCommand(goalId, envelopeId, "600000", "2026-12-31", "active")]);

        var savedTree = await repository.SaveBudgetTreeAsync(Period("2026-05"), command, TestContext.Current.CancellationToken);
        var savedGoal = savedTree.Goals.Single();

        savedTree.Envelopes.ShouldContain(envelope => envelope.Id == envelopeId && envelope.AssignedAmount == 100000m);
        savedGoal.Id.ShouldBe(goalId);
        savedGoal.EnvelopeId.ShouldBe(envelopeId);
        savedGoal.TargetAmount.ShouldBe(600000m);
        savedGoal.DeadlineDate.ShouldBe("2026-12-31");
        savedGoal.Status.ShouldBe("active");
    }

    [Fact]
    public async Task BudgetAssignedAmountsAreSpecificToPeriod()
    {
        await using var database = await CreateDatabaseAsync();
        var repository = new EfLedgerRepository(database.Context);
        var tree = await repository.GetBudgetTreeAsync(Period("2026-05"), TestContext.Current.CancellationToken);
        var subcategory = tree.Subcategories.Single(candidate => candidate.Name == "General");
        var envelopeId = Guid.CreateVersion7();
        var mayCommand = BudgetCommandWithEnvelope(tree, envelopeId, subcategory.Id, "Vacaciones", "100000");

        await repository.SaveBudgetTreeAsync(Period("2026-05"), mayCommand, TestContext.Current.CancellationToken);
        var juneTree = await repository.GetBudgetTreeAsync(Period("2026-05"), TestContext.Current.CancellationToken);
        var juneCommand = BudgetCommandWithEnvelope(juneTree, envelopeId, subcategory.Id, "Vacaciones", "250000");
        await repository.SaveBudgetTreeAsync(Period("2026-06"), juneCommand, TestContext.Current.CancellationToken);

        var mayTree = await repository.GetBudgetTreeAsync(Period("2026-05"), TestContext.Current.CancellationToken);
        var savedJuneTree = await repository.GetBudgetTreeAsync(Period("2026-06"), TestContext.Current.CancellationToken);

        mayTree.Envelopes.Single(envelope => envelope.Id == envelopeId).AssignedAmount.ShouldBe(100000m);
        savedJuneTree.Envelopes.Single(envelope => envelope.Id == envelopeId).AssignedAmount.ShouldBe(250000m);
    }

    [Fact]
    public async Task BudgetWithoutPeriodAllocationIgnoresLegacyEnvelopeAmount()
    {
        await using var database = await CreateDatabaseAsync();
        var repository = new EfLedgerRepository(database.Context);
        var tree = await repository.GetBudgetTreeAsync(Period("2026-05"), TestContext.Current.CancellationToken);
        var envelope = tree.Envelopes.Single(candidate => candidate.Name == "Ingresos");
        var entity = await database.Context.BudgetEnvelopes.FindAsync(
            [envelope.Id],
            TestContext.Current.CancellationToken);

        entity.ShouldNotBeNull();
        entity.AssignedAmount = "750000";
        await database.Context.SaveChangesAsync(TestContext.Current.CancellationToken);

        var nextMonthTree = await repository.GetBudgetTreeAsync(Period("2026-06"), TestContext.Current.CancellationToken);

        nextMonthTree.Envelopes.Single(candidate => candidate.Id == envelope.Id).AssignedAmount.ShouldBe(0m);
    }

    private static SaveBudgetTreeCommand BudgetCommandWithEnvelope(
        BudgetTreeReadModel tree,
        Guid envelopeId,
        Guid subcategoryId,
        string envelopeName,
        string assignedAmount)
    {
        var envelopes = tree.Envelopes
            .Where(envelope => envelope.Id != envelopeId)
            .Select(envelope => new SaveBudgetEnvelopeCommand(envelope.Id, envelope.SubcategoryId, envelope.Name, envelope.AssignedAmount.ToString(System.Globalization.CultureInfo.InvariantCulture), envelope.SortOrder))
            .Append(new SaveBudgetEnvelopeCommand(envelopeId, subcategoryId, envelopeName, assignedAmount, tree.Envelopes.Count))
            .ToList();

        return new SaveBudgetTreeCommand(
            tree.Categories.Select(category => new SaveBudgetCategoryCommand(category.Id, category.Name, category.SortOrder)).ToList(),
            tree.Subcategories.Select(subcategory => new SaveBudgetSubcategoryCommand(subcategory.Id, subcategory.CategoryId, subcategory.Name, subcategory.SortOrder)).ToList(),
            envelopes,
            tree.Goals.Select(goal => new SaveBudgetGoalCommand(goal.Id, goal.EnvelopeId, goal.TargetAmount.ToString(System.Globalization.CultureInfo.InvariantCulture), goal.DeadlineDate, goal.Status)).ToList());
    }

    private static MonthlyPeriod Period(string key)
    {
        MonthlyPeriod.TryParse(key, out var period).ShouldBeTrue();
        return period;
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
