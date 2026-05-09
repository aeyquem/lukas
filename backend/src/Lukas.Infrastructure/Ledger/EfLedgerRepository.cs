using System.Globalization;
using System.Text.Json;
using Lukas.Application.Ledger;
using Lukas.Infrastructure.Persistence;
using Lukas.Infrastructure.Persistence.Models;
using Microsoft.EntityFrameworkCore;

namespace Lukas.Infrastructure.Ledger;

public sealed class EfLedgerRepository(LukasDbContext dbContext) : ILedgerRepository
{
    private static readonly Guid DefaultWorkspaceId = Guid.Parse("018f9df8-6c1f-7c1a-9a32-700000000001");
    private static readonly Guid DefaultCategoryId = Guid.Parse("018f9df8-6c1f-7c1a-9a32-700000000101");
    private static readonly Guid DefaultSubcategoryId = Guid.Parse("018f9df8-6c1f-7c1a-9a32-700000000201");
    private static readonly Guid UncategorizedCategoryId = Guid.Parse("018f9df8-6c1f-7c1a-9a32-700000000102");
    private static readonly Guid UncategorizedSubcategoryId = Guid.Parse("018f9df8-6c1f-7c1a-9a32-700000000202");
    private static readonly Guid TrashCategoryId = Guid.Parse("018f9df8-6c1f-7c1a-9a32-700000000103");
    private static readonly Guid TrashSubcategoryId = Guid.Parse("018f9df8-6c1f-7c1a-9a32-700000000203");

    public async Task<AccountReadModel> CreateAccountAsync(
        CreateAccountCommand command,
        CancellationToken cancellationToken)
    {
        var workspace = await GetOrCreateDefaultWorkspaceAsync(cancellationToken).ConfigureAwait(false);
        var now = DateTimeOffset.UtcNow;
        var account = new AccountEntity
        {
            Id = Guid.CreateVersion7(),
            WorkspaceId = workspace.Id,
            DisplayName = command.DisplayName.Trim(),
            Type = command.Type,
            Currency = command.Currency,
            CreatedAt = now,
        };

        dbContext.Accounts.Add(account);

        if (command.OpeningBalance is not null
            && TryParseAmount(command.OpeningBalance, out var openingBalance)
            && openingBalance != 0)
        {
            AddManualTransaction(
                workspace.Id,
                account.Id,
                null,
                "adjustment",
                DateOnly.FromDateTime(now.DateTime),
                Math.Abs(openingBalance),
                account.Currency,
                openingBalance > 0 ? "increase" : "decrease",
                "Saldo inicial",
                null,
                null,
                null,
                now);
        }

        await dbContext.SaveChangesAsync(cancellationToken).ConfigureAwait(false);
        return await ToAccountReadModelAsync(account, cancellationToken).ConfigureAwait(false);
    }

    public async Task<IReadOnlyList<AccountReadModel>> GetAccountsAsync(CancellationToken cancellationToken)
    {
        var workspace = await GetOrCreateDefaultWorkspaceAsync(cancellationToken).ConfigureAwait(false);
        var accounts = await dbContext.Accounts
            .AsNoTracking()
            .Where(account => account.WorkspaceId == workspace.Id)
            .OrderBy(account => account.DisplayName)
            .ToListAsync(cancellationToken)
            .ConfigureAwait(false);

        var result = new List<AccountReadModel>(accounts.Count);
        foreach (var account in accounts)
        {
            result.Add(await ToAccountReadModelAsync(account, cancellationToken).ConfigureAwait(false));
        }

        return result;
    }

    public async Task<TransactionReadModel> CreateManualTransactionAsync(
        CreateManualTransactionCommand command,
        CancellationToken cancellationToken)
    {
        var workspace = await GetOrCreateDefaultWorkspaceAsync(cancellationToken).ConfigureAwait(false);
        var account = await dbContext.Accounts
            .SingleOrDefaultAsync(
                candidate => candidate.WorkspaceId == workspace.Id && candidate.Id == command.AccountId,
                cancellationToken)
            .ConfigureAwait(false);

        if (account is null)
        {
            throw new LedgerRuleException("account.not_found", "Account was not found.");
        }

        if (account.Currency != command.Currency)
        {
            throw new LedgerRuleException("transaction.currency_mismatch", "Transaction currency must match the account currency.");
        }

        var destinationAccount = await GetDestinationAccountAsync(
            workspace.Id,
            command.DestinationAccountId,
            command.Kind,
            command.Currency,
            cancellationToken).ConfigureAwait(false);
        var envelopeId = TransferEnvelopeId(command.Kind, command.EnvelopeId);
        var envelopeName = await GetEnvelopeNameAsync(workspace.Id, envelopeId, cancellationToken).ConfigureAwait(false);
        var categoryName = TransferCategoryName(command.Kind, envelopeName, command.CategoryName);
        _ = TryParseAmount(command.Amount, out var amount);
        var transaction = AddManualTransaction(
            workspace.Id,
            account.Id,
            destinationAccount?.Id,
            command.Kind,
            command.TransactionDate,
            amount,
            command.Currency,
            command.AccountEffect,
            command.Description.Trim(),
            envelopeId,
            categoryName,
            string.IsNullOrWhiteSpace(command.Notes) ? null : command.Notes.Trim(),
            DateTimeOffset.UtcNow);

        await dbContext.SaveChangesAsync(cancellationToken).ConfigureAwait(false);
        return ToTransactionReadModel(transaction);
    }

    public async Task<TransactionReadModel> UpdateManualTransactionAsync(
        UpdateManualTransactionCommand command,
        CancellationToken cancellationToken)
    {
        var workspace = await GetOrCreateDefaultWorkspaceAsync(cancellationToken).ConfigureAwait(false);
        var transaction = await dbContext.LedgerTransactions
            .Include(candidate => candidate.Entries)
            .SingleOrDefaultAsync(
                candidate => candidate.WorkspaceId == workspace.Id && candidate.Id == command.Id,
                cancellationToken)
            .ConfigureAwait(false);

        if (transaction is null)
        {
            throw new LedgerRuleException("transaction.not_found", "Transaction was not found.");
        }

        var account = await dbContext.Accounts
            .SingleOrDefaultAsync(
                candidate => candidate.WorkspaceId == workspace.Id && candidate.Id == command.AccountId,
                cancellationToken)
            .ConfigureAwait(false);

        if (account is null)
        {
            throw new LedgerRuleException("account.not_found", "Account was not found.");
        }

        if (account.Currency != command.Currency)
        {
            throw new LedgerRuleException("transaction.currency_mismatch", "Transaction currency must match the account currency.");
        }

        var envelopeId = TransferEnvelopeId(command.Kind, command.EnvelopeId);
        var envelopeName = await GetEnvelopeNameAsync(workspace.Id, envelopeId, cancellationToken).ConfigureAwait(false);
        var categoryName = TransferCategoryName(command.Kind, envelopeName, command.CategoryName);
        _ = TryParseAmount(command.Amount, out var amount);
        var destinationAccount = await GetDestinationAccountAsync(
            workspace.Id,
            command.DestinationAccountId,
            command.Kind,
            command.Currency,
            cancellationToken).ConfigureAwait(false);
        transaction.Kind = command.Kind;
        transaction.TransactionDate = command.TransactionDate;
        transaction.Description = command.Description.Trim();
        transaction.EnvelopeId = envelopeId;
        transaction.CategoryName = categoryName;
        transaction.Notes = string.IsNullOrWhiteSpace(command.Notes) ? null : command.Notes.Trim();
        dbContext.LedgerEntries.RemoveRange(transaction.Entries);
        transaction.Entries.Clear();
        AddManualEntries(transaction, account.Id, destinationAccount?.Id, amount, command.Currency, command.AccountEffect);

        await dbContext.SaveChangesAsync(cancellationToken).ConfigureAwait(false);
        return ToTransactionReadModel(transaction);
    }

    public async Task<IReadOnlyList<MovementReadModel>> GetMovementsAsync(CancellationToken cancellationToken)
    {
        var workspace = await GetOrCreateDefaultWorkspaceAsync(cancellationToken).ConfigureAwait(false);

        var entries = await dbContext.LedgerEntries
            .AsNoTracking()
            .Where(entry => entry.WorkspaceId == workspace.Id)
            .OrderByDescending(entry => entry.LedgerTransaction!.TransactionDate)
            .ThenByDescending(entry => entry.LedgerTransactionId)
            .ThenByDescending(entry => entry.Id)
            .Select(entry => new
            {
                entry.Id,
                TransactionId = entry.LedgerTransactionId,
                entry.LedgerTransaction!.TransactionDate,
                entry.AccountId,
                AccountName = entry.Account!.DisplayName,
                DestinationAccountId = entry.LedgerTransaction.Kind == "transfer"
                    ? entry.LedgerTransaction.Entries
                        .Where(candidate => candidate.Id != entry.Id)
                        .Select(candidate => (Guid?)candidate.AccountId)
                        .FirstOrDefault()
                    : null,
                entry.LedgerTransaction.Kind,
                entry.AccountEffect,
                entry.AmountValue,
                entry.Currency,
                entry.LedgerTransaction.Description,
                entry.LedgerTransaction.EnvelopeId,
                EnvelopeName = entry.LedgerTransaction.Envelope == null ? null : entry.LedgerTransaction.Envelope.Name,
                CategoryName = entry.LedgerTransaction.Kind == "transfer"
                    ? "Transferencia"
                    : entry.LedgerTransaction.CategoryName,
                entry.LedgerTransaction.Notes,
            })
            .ToListAsync(cancellationToken)
            .ConfigureAwait(false);

        return entries
            .Select(entry =>
            {
                var amount = decimal.Parse(entry.AmountValue, CultureInfo.InvariantCulture);
                return new MovementReadModel(
                    entry.Id,
                    entry.TransactionId,
                    entry.TransactionDate,
                    entry.AccountId,
                    entry.AccountName,
                    entry.DestinationAccountId,
                    entry.Kind,
                    entry.AccountEffect,
                    entry.AccountEffect == "decrease" ? "out" : "in",
                    amount,
                    entry.Currency,
                    amount,
                    entry.Currency,
                    entry.Description,
                    entry.EnvelopeId,
                    entry.EnvelopeName,
                    entry.CategoryName,
                    entry.Notes);
            })
            .ToList();
    }

    public async Task<BudgetTreeReadModel> GetBudgetTreeAsync(CancellationToken cancellationToken)
    {
        var workspace = await GetOrCreateDefaultWorkspaceAsync(cancellationToken).ConfigureAwait(false);
        await EnsureBudgetTreeAsync(workspace.Id, cancellationToken).ConfigureAwait(false);
        await EnsureTrashBudgetNodesAsync(workspace.Id, cancellationToken).ConfigureAwait(false);
        await AttachLegacyCategoryNamesAsync(workspace.Id, cancellationToken).ConfigureAwait(false);
        return await ReadBudgetTreeAsync(workspace.Id, cancellationToken).ConfigureAwait(false);
    }

    public async Task<BudgetTreeReadModel> SaveBudgetTreeAsync(
        SaveBudgetTreeCommand command,
        CancellationToken cancellationToken)
    {
        var workspace = await GetOrCreateDefaultWorkspaceAsync(cancellationToken).ConfigureAwait(false);
        await EnsureTrashBudgetNodesAsync(workspace.Id, cancellationToken).ConfigureAwait(false);
        await ReplaceBudgetTreeAsync(workspace.Id, command, cancellationToken).ConfigureAwait(false);
        await dbContext.SaveChangesAsync(cancellationToken).ConfigureAwait(false);
        await AttachLegacyCategoryNamesAsync(workspace.Id, cancellationToken).ConfigureAwait(false);
        return await ReadBudgetTreeAsync(workspace.Id, cancellationToken).ConfigureAwait(false);
    }

    public async Task<IReadOnlyList<TransactionReadModel>> GetTransactionsAsync(
        string period,
        CancellationToken cancellationToken)
    {
        var workspace = await GetOrCreateDefaultWorkspaceAsync(cancellationToken).ConfigureAwait(false);
        var (start, end) = GetPeriodRange(period);

        var transactions = await dbContext.LedgerTransactions
            .AsNoTracking()
            .Include(transaction => transaction.Entries)
            .Where(transaction =>
                transaction.WorkspaceId == workspace.Id
                && transaction.TransactionDate >= start
                && transaction.TransactionDate < end)
            .OrderByDescending(transaction => transaction.TransactionDate)
            .ThenByDescending(transaction => transaction.Id)
            .ToListAsync(cancellationToken)
            .ConfigureAwait(false);

        return transactions.Select(ToTransactionReadModel).ToList();
    }

    public async Task<MonthlyCashflowReadModel> GetMonthlyCashflowAsync(
        string period,
        CancellationToken cancellationToken)
    {
        var workspace = await GetOrCreateDefaultWorkspaceAsync(cancellationToken).ConfigureAwait(false);
        var (start, end) = GetPeriodRange(period);

        var transactions = await dbContext.LedgerTransactions
            .AsNoTracking()
            .Include(transaction => transaction.Entries)
            .Where(transaction =>
                transaction.WorkspaceId == workspace.Id
                && transaction.TransactionDate >= start
                && transaction.TransactionDate < end)
            .ToListAsync(cancellationToken)
            .ConfigureAwait(false);

        var income = SumClpTransactions(transactions, "income", "increase");
        var expenses = SumClpTransactions(transactions, "expense", "decrease");
        var accounts = await GetAccountsAsync(cancellationToken).ConfigureAwait(false);

        return new MonthlyCashflowReadModel(
            period,
            workspace.BaseCurrency,
            income,
            expenses,
            income - expenses,
            accounts);
    }

    private async Task EnsureBudgetTreeAsync(Guid workspaceId, CancellationToken cancellationToken)
    {
        var hasBudgetTree = await dbContext.BudgetCategories
            .AnyAsync(category => category.WorkspaceId == workspaceId, cancellationToken)
            .ConfigureAwait(false);

        if (hasBudgetTree)
        {
            return;
        }

        dbContext.BudgetCategories.Add(new BudgetCategoryEntity
        {
            Id = DefaultCategoryId,
            WorkspaceId = workspaceId,
            Name = "Vida diaria",
            SortOrder = 0,
        });
        dbContext.BudgetSubcategories.Add(new BudgetSubcategoryEntity
        {
            Id = DefaultSubcategoryId,
            WorkspaceId = workspaceId,
            CategoryId = DefaultCategoryId,
            Name = "General",
            SortOrder = 0,
        });
        await dbContext.SaveChangesAsync(cancellationToken).ConfigureAwait(false);
    }

    private async Task AttachLegacyCategoryNamesAsync(Guid workspaceId, CancellationToken cancellationToken)
    {
        var categoryNames = await dbContext.LedgerTransactions
            .Where(transaction =>
                transaction.WorkspaceId == workspaceId
                && transaction.EnvelopeId == null
                && transaction.Kind != "transfer"
                && transaction.CategoryName != null)
            .Select(transaction => transaction.CategoryName!)
            .Distinct()
            .ToListAsync(cancellationToken)
            .ConfigureAwait(false);

        if (categoryNames.Count == 0)
        {
            return;
        }

        var subcategoryId = await GetOrCreateUncategorizedSubcategoryAsync(workspaceId, cancellationToken).ConfigureAwait(false);
        var existingEnvelopes = await dbContext.BudgetEnvelopes
            .Where(envelope => envelope.WorkspaceId == workspaceId)
            .ToListAsync(cancellationToken)
            .ConfigureAwait(false);
        var nextSortOrder = existingEnvelopes.Count == 0 ? 0 : existingEnvelopes.Max(envelope => envelope.SortOrder) + 1;

        foreach (var categoryName in categoryNames)
        {
            var envelope = existingEnvelopes.Find(candidate => candidate.Name == categoryName);
            if (envelope is null)
            {
                envelope = new BudgetEnvelopeEntity
                {
                    Id = Guid.CreateVersion7(),
                    WorkspaceId = workspaceId,
                    SubcategoryId = subcategoryId,
                    Name = categoryName,
                    AssignedAmount = "0",
                    SortOrder = nextSortOrder,
                };
                nextSortOrder++;
                existingEnvelopes.Add(envelope);
                dbContext.BudgetEnvelopes.Add(envelope);
            }

            await AttachTransactionsToEnvelopeAsync(workspaceId, categoryName, envelope.Id, cancellationToken).ConfigureAwait(false);
        }

        await dbContext.SaveChangesAsync(cancellationToken).ConfigureAwait(false);
    }

    private async Task<Guid> GetOrCreateUncategorizedSubcategoryAsync(
        Guid workspaceId,
        CancellationToken cancellationToken)
    {
        var subcategory = await dbContext.BudgetSubcategories
            .SingleOrDefaultAsync(
                candidate => candidate.WorkspaceId == workspaceId && candidate.Id == UncategorizedSubcategoryId,
                cancellationToken)
            .ConfigureAwait(false);

        if (subcategory is not null)
        {
            return subcategory.Id;
        }

        var category = await dbContext.BudgetCategories
            .SingleOrDefaultAsync(
                candidate => candidate.WorkspaceId == workspaceId && candidate.Id == UncategorizedCategoryId,
                cancellationToken)
            .ConfigureAwait(false);

        if (category is null)
        {
            category = new BudgetCategoryEntity
            {
                Id = UncategorizedCategoryId,
                WorkspaceId = workspaceId,
                Name = "Sin organizar",
                SortOrder = 1,
            };
            dbContext.BudgetCategories.Add(category);
        }

        dbContext.BudgetSubcategories.Add(new BudgetSubcategoryEntity
        {
            Id = UncategorizedSubcategoryId,
            WorkspaceId = workspaceId,
            CategoryId = category.Id,
            Name = "Movimientos existentes",
            SortOrder = 0,
        });

        return UncategorizedSubcategoryId;
    }

    private async Task EnsureTrashBudgetNodesAsync(Guid workspaceId, CancellationToken cancellationToken)
    {
        var category = await dbContext.BudgetCategories
            .SingleOrDefaultAsync(
                candidate => candidate.WorkspaceId == workspaceId && candidate.Id == TrashCategoryId,
                cancellationToken)
            .ConfigureAwait(false);
        if (category is null)
        {
            dbContext.BudgetCategories.Add(new BudgetCategoryEntity
            {
                Id = TrashCategoryId,
                WorkspaceId = workspaceId,
                Name = "Papelera",
                SortOrder = int.MaxValue,
            });
        }

        var subcategory = await dbContext.BudgetSubcategories
            .SingleOrDefaultAsync(
                candidate => candidate.WorkspaceId == workspaceId && candidate.Id == TrashSubcategoryId,
                cancellationToken)
            .ConfigureAwait(false);
        if (subcategory is null)
        {
            dbContext.BudgetSubcategories.Add(new BudgetSubcategoryEntity
            {
                Id = TrashSubcategoryId,
                WorkspaceId = workspaceId,
                CategoryId = TrashCategoryId,
                Name = "Categorías eliminadas",
                SortOrder = 0,
            });
        }

        await dbContext.SaveChangesAsync(cancellationToken).ConfigureAwait(false);
    }

    private async Task AttachTransactionsToEnvelopeAsync(
        Guid workspaceId,
        string categoryName,
        Guid envelopeId,
        CancellationToken cancellationToken)
    {
        var transactions = await dbContext.LedgerTransactions
            .Where(transaction =>
                transaction.WorkspaceId == workspaceId
                && transaction.EnvelopeId == null
                && transaction.Kind != "transfer"
                && transaction.CategoryName == categoryName)
            .ToListAsync(cancellationToken)
            .ConfigureAwait(false);

        foreach (var transaction in transactions)
        {
            transaction.EnvelopeId = envelopeId;
        }
    }

    private async Task<BudgetTreeReadModel> ReadBudgetTreeAsync(
        Guid workspaceId,
        CancellationToken cancellationToken)
    {
        var categories = await dbContext.BudgetCategories
            .AsNoTracking()
            .Where(category => category.WorkspaceId == workspaceId)
            .OrderBy(category => category.SortOrder)
            .ThenBy(category => category.Name)
            .ToListAsync(cancellationToken)
            .ConfigureAwait(false);
        var subcategories = await dbContext.BudgetSubcategories
            .AsNoTracking()
            .Where(subcategory => subcategory.WorkspaceId == workspaceId)
            .OrderBy(subcategory => subcategory.SortOrder)
            .ThenBy(subcategory => subcategory.Name)
            .ToListAsync(cancellationToken)
            .ConfigureAwait(false);
        var envelopes = await dbContext.BudgetEnvelopes
            .AsNoTracking()
            .Where(envelope => envelope.WorkspaceId == workspaceId)
            .OrderBy(envelope => envelope.SortOrder)
            .ThenBy(envelope => envelope.Name)
            .ToListAsync(cancellationToken)
            .ConfigureAwait(false);

        return new BudgetTreeReadModel(
            categories.Select(ToBudgetCategoryReadModel).ToList(),
            subcategories.Select(ToBudgetSubcategoryReadModel).ToList(),
            envelopes.Select(ToBudgetEnvelopeReadModel).ToList());
    }

    private async Task ReplaceBudgetTreeAsync(
        Guid workspaceId,
        SaveBudgetTreeCommand command,
        CancellationToken cancellationToken)
    {
        await RemoveDeletedBudgetNodesAsync(workspaceId, command, cancellationToken).ConfigureAwait(false);
        await UpsertBudgetCategoriesAsync(workspaceId, command.Categories, cancellationToken).ConfigureAwait(false);
        await UpsertBudgetSubcategoriesAsync(workspaceId, command.Subcategories, cancellationToken).ConfigureAwait(false);
        await UpsertBudgetEnvelopesAsync(workspaceId, command.Envelopes, cancellationToken).ConfigureAwait(false);
    }

    private async Task RemoveDeletedBudgetNodesAsync(
        Guid workspaceId,
        SaveBudgetTreeCommand command,
        CancellationToken cancellationToken)
    {
        var categoryIds = command.Categories.Select(category => category.Id).ToHashSet();
        var subcategoryIds = command.Subcategories.Select(subcategory => subcategory.Id).ToHashSet();
        var envelopeIds = command.Envelopes.Select(envelope => envelope.Id).ToHashSet();
        categoryIds.Add(TrashCategoryId);
        subcategoryIds.Add(TrashSubcategoryId);
        var deletedSubcategoryIds = await dbContext.BudgetSubcategories
            .Where(subcategory => subcategory.WorkspaceId == workspaceId && !subcategoryIds.Contains(subcategory.Id))
            .Select(subcategory => subcategory.Id)
            .ToListAsync(cancellationToken)
            .ConfigureAwait(false);
        var deletedEnvelopeIds = await dbContext.BudgetEnvelopes
            .Where(envelope =>
                envelope.WorkspaceId == workspaceId
                && !envelopeIds.Contains(envelope.Id)
                && envelope.SubcategoryId != TrashSubcategoryId)
            .Select(envelope => envelope.Id)
            .ToListAsync(cancellationToken)
            .ConfigureAwait(false);

        await MoveEnvelopesToTrashAsync(workspaceId, deletedEnvelopeIds, deletedSubcategoryIds, cancellationToken).ConfigureAwait(false);
        dbContext.BudgetSubcategories.RemoveRange(dbContext.BudgetSubcategories.Where(subcategory => subcategory.WorkspaceId == workspaceId && !subcategoryIds.Contains(subcategory.Id)));
        dbContext.BudgetCategories.RemoveRange(dbContext.BudgetCategories.Where(category => category.WorkspaceId == workspaceId && !categoryIds.Contains(category.Id)));
    }

    private async Task MoveEnvelopesToTrashAsync(
        Guid workspaceId,
        IReadOnlyCollection<Guid> deletedEnvelopeIds,
        IReadOnlyCollection<Guid> deletedSubcategoryIds,
        CancellationToken cancellationToken)
    {
        var envelopes = await dbContext.BudgetEnvelopes
            .Where(envelope =>
                envelope.WorkspaceId == workspaceId
                && (deletedEnvelopeIds.Contains(envelope.Id) || deletedSubcategoryIds.Contains(envelope.SubcategoryId)))
            .ToListAsync(cancellationToken)
            .ConfigureAwait(false);

        foreach (var envelope in envelopes)
        {
            envelope.SubcategoryId = TrashSubcategoryId;
            envelope.SortOrder = int.MaxValue;
        }
    }

    private async Task UpsertBudgetCategoriesAsync(
        Guid workspaceId,
        IReadOnlyList<SaveBudgetCategoryCommand> commands,
        CancellationToken cancellationToken)
    {
        foreach (var command in commands)
        {
            if (command.Id == TrashCategoryId)
            {
                continue;
            }

            var category = await dbContext.BudgetCategories
                .SingleOrDefaultAsync(candidate => candidate.WorkspaceId == workspaceId && candidate.Id == command.Id, cancellationToken)
                .ConfigureAwait(false);
            if (category is null)
            {
                dbContext.BudgetCategories.Add(new BudgetCategoryEntity { Id = command.Id, WorkspaceId = workspaceId, Name = command.Name.Trim(), SortOrder = command.SortOrder });
                continue;
            }

            category.Name = command.Name.Trim();
            category.SortOrder = command.SortOrder;
        }
    }

    private async Task UpsertBudgetSubcategoriesAsync(
        Guid workspaceId,
        IReadOnlyList<SaveBudgetSubcategoryCommand> commands,
        CancellationToken cancellationToken)
    {
        foreach (var command in commands)
        {
            if (command.Id == TrashSubcategoryId)
            {
                continue;
            }

            var subcategory = await dbContext.BudgetSubcategories
                .SingleOrDefaultAsync(candidate => candidate.WorkspaceId == workspaceId && candidate.Id == command.Id, cancellationToken)
                .ConfigureAwait(false);
            if (subcategory is null)
            {
                dbContext.BudgetSubcategories.Add(new BudgetSubcategoryEntity { Id = command.Id, WorkspaceId = workspaceId, CategoryId = command.CategoryId, Name = command.Name.Trim(), SortOrder = command.SortOrder });
                continue;
            }

            subcategory.CategoryId = command.CategoryId;
            subcategory.Name = command.Name.Trim();
            subcategory.SortOrder = command.SortOrder;
        }
    }

    private async Task UpsertBudgetEnvelopesAsync(
        Guid workspaceId,
        IReadOnlyList<SaveBudgetEnvelopeCommand> commands,
        CancellationToken cancellationToken)
    {
        foreach (var command in commands)
        {
            var envelope = await dbContext.BudgetEnvelopes
                .SingleOrDefaultAsync(candidate => candidate.WorkspaceId == workspaceId && candidate.Id == command.Id, cancellationToken)
                .ConfigureAwait(false);
            var assignedAmount = FormatAmount(decimal.Parse(command.AssignedAmount, CultureInfo.InvariantCulture));
            if (envelope is null)
            {
                dbContext.BudgetEnvelopes.Add(new BudgetEnvelopeEntity { Id = command.Id, WorkspaceId = workspaceId, SubcategoryId = command.SubcategoryId, Name = command.Name.Trim(), AssignedAmount = assignedAmount, SortOrder = command.SortOrder });
                continue;
            }

            envelope.SubcategoryId = command.SubcategoryId;
            envelope.Name = command.Name.Trim();
            envelope.AssignedAmount = assignedAmount;
            envelope.SortOrder = command.SortOrder;
        }
    }

    private async Task<WorkspaceEntity> GetOrCreateDefaultWorkspaceAsync(CancellationToken cancellationToken)
    {
        var workspace = await dbContext.Workspaces
            .SingleOrDefaultAsync(candidate => candidate.Id == DefaultWorkspaceId, cancellationToken)
            .ConfigureAwait(false);

        if (workspace is not null)
        {
            return workspace;
        }

        workspace = new WorkspaceEntity
        {
            Id = DefaultWorkspaceId,
            DisplayName = "Lukas",
            BaseCurrency = "CLP",
            TimeZone = "America/Santiago",
            CreatedAt = DateTimeOffset.UtcNow,
        };

        dbContext.Workspaces.Add(workspace);
        await dbContext.SaveChangesAsync(cancellationToken).ConfigureAwait(false);
        return workspace;
    }

    private async Task<string?> GetEnvelopeNameAsync(
        Guid workspaceId,
        Guid? envelopeId,
        CancellationToken cancellationToken)
    {
        if (envelopeId is null)
        {
            return null;
        }

        var envelope = await dbContext.BudgetEnvelopes
            .AsNoTracking()
            .SingleOrDefaultAsync(
                candidate => candidate.WorkspaceId == workspaceId && candidate.Id == envelopeId.Value,
                cancellationToken)
            .ConfigureAwait(false);

        if (envelope is null)
        {
            throw new LedgerRuleException("envelope.not_found", "Envelope was not found.");
        }

        return envelope.Name;
    }

    private async Task<AccountEntity?> GetDestinationAccountAsync(
        Guid workspaceId,
        Guid? destinationAccountId,
        string kind,
        string currency,
        CancellationToken cancellationToken)
    {
        if (kind != "transfer")
        {
            return null;
        }

        var account = await dbContext.Accounts
            .SingleOrDefaultAsync(
                candidate => candidate.WorkspaceId == workspaceId && candidate.Id == destinationAccountId,
                cancellationToken)
            .ConfigureAwait(false);

        if (account is null)
        {
            throw new LedgerRuleException("account.destination_not_found", "Destination account was not found.");
        }

        if (account.Currency != currency)
        {
            throw new LedgerRuleException("transaction.currency_mismatch", "Transaction currency must match both account currencies.");
        }

        return account;
    }

    private LedgerTransactionEntity AddManualTransaction(
        Guid workspaceId,
        Guid accountId,
        Guid? destinationAccountId,
        string kind,
        DateOnly transactionDate,
        decimal amount,
        string currency,
        string accountEffect,
        string description,
        Guid? envelopeId,
        string? categoryName,
        string? notes,
        DateTimeOffset now)
    {
        var transaction = new LedgerTransactionEntity
        {
            Id = Guid.CreateVersion7(),
            WorkspaceId = workspaceId,
            Kind = kind,
            TransactionDate = transactionDate,
            Description = description,
            EnvelopeId = envelopeId,
            CategoryName = categoryName,
            Notes = notes,
            CreatedAt = now,
        };

        var snapshots = JsonSerializer.Serialize(new
        {
            kind,
            accountId,
            destinationAccountId,
            transactionDate,
            amount = FormatAmount(amount),
            currency,
            accountEffect,
            description,
            envelopeId,
            categoryName,
            notes,
        });

        var sourceRecord = new SourceRecordEntity
        {
            Id = Guid.CreateVersion7(),
            WorkspaceId = workspaceId,
            SourceType = "manual",
            RawSnapshot = snapshots,
            NormalizedSnapshot = snapshots,
            CreatedAt = now,
        };

        dbContext.LedgerTransactions.Add(transaction);
        AddManualEntries(transaction, accountId, destinationAccountId, amount, currency, accountEffect);
        dbContext.SourceRecords.Add(sourceRecord);
        dbContext.TransactionSourceLinks.Add(new TransactionSourceLinkEntity
        {
            LedgerTransactionId = transaction.Id,
            SourceRecordId = sourceRecord.Id,
        });

        return transaction;
    }

    private void AddManualEntries(
        LedgerTransactionEntity transaction,
        Guid accountId,
        Guid? destinationAccountId,
        decimal amount,
        string currency,
        string accountEffect)
    {
        AddManualEntry(transaction, accountId, amount, currency, accountEffect);
        if (transaction.Kind == "transfer" && destinationAccountId is not null)
        {
            AddManualEntry(transaction, destinationAccountId.Value, amount, currency, "increase");
        }
    }

    private void AddManualEntry(
        LedgerTransactionEntity transaction,
        Guid accountId,
        decimal amount,
        string currency,
        string accountEffect)
    {
        var entry = CreateEntry(transaction, accountId, amount, currency, accountEffect);
        transaction.Entries.Add(entry);
        dbContext.LedgerEntries.Add(entry);
    }

    private static LedgerEntryEntity CreateEntry(
        LedgerTransactionEntity transaction,
        Guid accountId,
        decimal amount,
        string currency,
        string accountEffect)
    {
        return new LedgerEntryEntity
        {
            Id = Guid.CreateVersion7(),
            WorkspaceId = transaction.WorkspaceId,
            LedgerTransactionId = transaction.Id,
            AccountId = accountId,
            AmountValue = FormatAmount(amount),
            Currency = currency,
            AccountEffect = accountEffect,
        };
    }

    private async Task<AccountReadModel> ToAccountReadModelAsync(
        AccountEntity account,
        CancellationToken cancellationToken)
    {
        var entries = await dbContext.LedgerEntries
            .AsNoTracking()
            .Where(entry => entry.AccountId == account.Id && entry.Currency == account.Currency)
            .ToListAsync(cancellationToken)
            .ConfigureAwait(false);

        var balance = entries.Sum(entry =>
        {
            var amount = decimal.Parse(entry.AmountValue, CultureInfo.InvariantCulture);
            return entry.AccountEffect == "increase" ? amount : -amount;
        });

        return new AccountReadModel(account.Id, account.DisplayName, account.Type, account.Currency, balance);
    }

    private static TransactionReadModel ToTransactionReadModel(LedgerTransactionEntity transaction)
    {
        var entry = PrimaryEntry(transaction);
        return new TransactionReadModel(
            transaction.Id,
            transaction.Kind,
            entry.AccountId,
            DestinationAccountId(transaction, entry.Id),
            transaction.TransactionDate,
            decimal.Parse(entry.AmountValue, CultureInfo.InvariantCulture),
            entry.Currency,
            entry.AccountEffect,
            transaction.Description,
            transaction.EnvelopeId,
            transaction.Envelope?.Name,
            EffectiveCategoryName(transaction.Kind, transaction.CategoryName),
            transaction.Notes);
    }

    private static LedgerEntryEntity PrimaryEntry(LedgerTransactionEntity transaction)
    {
        if (transaction.Kind == "transfer")
        {
            return transaction.Entries.Single(entry => entry.AccountEffect == "decrease");
        }

        return transaction.Entries.Single();
    }

    private static Guid? DestinationAccountId(
        LedgerTransactionEntity transaction,
        Guid entryId)
    {
        if (transaction.Kind != "transfer")
        {
            return null;
        }

        return transaction.Entries.Single(entry => entry.Id != entryId).AccountId;
    }

    private static BudgetCategoryReadModel ToBudgetCategoryReadModel(BudgetCategoryEntity category)
    {
        return new BudgetCategoryReadModel(category.Id, category.Name, category.SortOrder);
    }

    private static BudgetSubcategoryReadModel ToBudgetSubcategoryReadModel(BudgetSubcategoryEntity subcategory)
    {
        return new BudgetSubcategoryReadModel(
            subcategory.Id,
            subcategory.CategoryId,
            subcategory.Name,
            subcategory.SortOrder);
    }

    private static BudgetEnvelopeReadModel ToBudgetEnvelopeReadModel(BudgetEnvelopeEntity envelope)
    {
        return new BudgetEnvelopeReadModel(
            envelope.Id,
            envelope.SubcategoryId,
            envelope.Name,
            decimal.Parse(envelope.AssignedAmount, CultureInfo.InvariantCulture),
            envelope.SortOrder);
    }

    private static decimal SumClpTransactions(
        IEnumerable<LedgerTransactionEntity> transactions,
        string kind,
        string accountEffect)
    {
        return transactions
            .Where(transaction => transaction.Kind == kind)
            .Select(transaction => transaction.Entries.Single())
            .Where(entry => entry.Currency == "CLP" && entry.AccountEffect == accountEffect)
            .Sum(entry => decimal.Parse(entry.AmountValue, CultureInfo.InvariantCulture));
    }

    private static bool TryParseAmount(string value, out decimal amount)
    {
        return decimal.TryParse(value, NumberStyles.Number, CultureInfo.InvariantCulture, out amount);
    }

    private static string? EmptyToNull(string? value)
    {
        return string.IsNullOrWhiteSpace(value) ? null : value.Trim();
    }

    private static Guid? TransferEnvelopeId(string kind, Guid? envelopeId)
    {
        return kind == "transfer" ? null : envelopeId;
    }

    private static string? TransferCategoryName(
        string kind,
        string? envelopeName,
        string? categoryName)
    {
        return kind == "transfer" ? "Transferencia" : envelopeName ?? EmptyToNull(categoryName);
    }

    private static string? EffectiveCategoryName(string kind, string? categoryName)
    {
        return kind == "transfer" ? "Transferencia" : categoryName;
    }

    private static string FormatAmount(decimal amount)
    {
        return amount.ToString("0.#############################", CultureInfo.InvariantCulture);
    }

    private static (DateOnly Start, DateOnly End) GetPeriodRange(string period)
    {
        var start = DateOnly.ParseExact($"{period}-01", "yyyy-MM-dd", CultureInfo.InvariantCulture);
        return (start, start.AddMonths(1));
    }
}
