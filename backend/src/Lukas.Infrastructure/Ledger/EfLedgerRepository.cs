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
    private static readonly Guid IncomeCategoryId = Guid.Parse("018f9df8-6c1f-7c1a-9a32-700000000104");
    private static readonly Guid IncomeSubcategoryId = Guid.Parse("018f9df8-6c1f-7c1a-9a32-700000000204");
    private static readonly Guid IncomeEnvelopeId = Guid.Parse("018f9df8-6c1f-7c1a-9a32-700000000304");
    private static readonly Guid GoalsCategoryId = Guid.Parse("018f9df8-6c1f-7c1a-9a32-700000000105");
    private static readonly Guid GoalsSubcategoryId = Guid.Parse("018f9df8-6c1f-7c1a-9a32-700000000205");

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

    public async Task<IReadOnlyList<TagReadModel>> GetTagsAsync(CancellationToken cancellationToken)
    {
        var workspace = await GetOrCreateDefaultWorkspaceAsync(cancellationToken).ConfigureAwait(false);
        var tags = await dbContext.Tags
            .AsNoTracking()
            .Where(tag => tag.WorkspaceId == workspace.Id)
            .OrderBy(tag => tag.Name)
            .ToListAsync(cancellationToken)
            .ConfigureAwait(false);

        return tags.Select(ToTagReadModel).ToList();
    }

    public async Task<TagReadModel> CreateTagAsync(
        CreateTagCommand command,
        CancellationToken cancellationToken)
    {
        var workspace = await GetOrCreateDefaultWorkspaceAsync(cancellationToken).ConfigureAwait(false);
        var normalizedName = TransactionTagNames.Normalize(command.Name);
        var exists = await dbContext.Tags
            .AnyAsync(
                tag => tag.WorkspaceId == workspace.Id && tag.NormalizedName == normalizedName,
                cancellationToken)
            .ConfigureAwait(false);
        if (exists)
        {
            throw new LedgerRuleException("tag.duplicate", "Tag already exists.");
        }

        var tag = new TagEntity
        {
            Id = Guid.CreateVersion7(),
            WorkspaceId = workspace.Id,
            Name = TransactionTagNames.DisplayName(command.Name),
            NormalizedName = normalizedName,
            Color = command.Color.ToUpperInvariant(),
            CreatedAt = DateTimeOffset.UtcNow,
        };

        dbContext.Tags.Add(tag);
        await dbContext.SaveChangesAsync(cancellationToken).ConfigureAwait(false);
        return ToTagReadModel(tag);
    }

    public async Task<TagReadModel> UpdateTagAsync(
        UpdateTagCommand command,
        CancellationToken cancellationToken)
    {
        var workspace = await GetOrCreateDefaultWorkspaceAsync(cancellationToken).ConfigureAwait(false);
        var tag = await dbContext.Tags
            .SingleOrDefaultAsync(
                candidate => candidate.WorkspaceId == workspace.Id && candidate.Id == command.Id,
                cancellationToken)
            .ConfigureAwait(false);
        if (tag is null)
        {
            throw new LedgerRuleException("tag.not_found", "Tag was not found.");
        }

        var normalizedName = TransactionTagNames.Normalize(command.Name);
        var duplicateExists = await dbContext.Tags
            .AnyAsync(
                candidate =>
                    candidate.WorkspaceId == workspace.Id
                    && candidate.Id != command.Id
                    && candidate.NormalizedName == normalizedName,
                cancellationToken)
            .ConfigureAwait(false);
        if (duplicateExists)
        {
            throw new LedgerRuleException("tag.duplicate", "Tag already exists.");
        }

        tag.Name = TransactionTagNames.DisplayName(command.Name);
        tag.NormalizedName = normalizedName;
        tag.Color = command.Color.ToUpperInvariant();

        await dbContext.SaveChangesAsync(cancellationToken).ConfigureAwait(false);
        return ToTagReadModel(tag);
    }

    public async Task DeleteTagAsync(Guid id, CancellationToken cancellationToken)
    {
        var workspace = await GetOrCreateDefaultWorkspaceAsync(cancellationToken).ConfigureAwait(false);
        var tag = await dbContext.Tags
            .SingleOrDefaultAsync(
                candidate => candidate.WorkspaceId == workspace.Id && candidate.Id == id,
                cancellationToken)
            .ConfigureAwait(false);
        if (tag is null)
        {
            throw new LedgerRuleException("tag.not_found", "Tag was not found.");
        }

        dbContext.Tags.Remove(tag);
        await dbContext.SaveChangesAsync(cancellationToken).ConfigureAwait(false);
    }

    public async Task<TransactionReadModel> CreateManualTransactionAsync(
        CreateManualTransactionCommand command,
        CancellationToken cancellationToken)
    {
        var workspace = await GetOrCreateDefaultWorkspaceAsync(cancellationToken).ConfigureAwait(false);
        await EnsureIncomeBudgetNodesAsync(workspace.Id, cancellationToken).ConfigureAwait(false);
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
        var envelopeId = SystemEnvelopeId(command.Kind, command.EnvelopeId);
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
            command.Tags,
            DateTimeOffset.UtcNow);
        await ReplaceTransactionTagsAsync(
            workspace.Id,
            transaction,
            command.Tags,
            DateTimeOffset.UtcNow,
            cancellationToken).ConfigureAwait(false);

        await dbContext.SaveChangesAsync(cancellationToken).ConfigureAwait(false);
        return ToTransactionReadModel(transaction);
    }

    public async Task<TransactionReadModel> UpdateManualTransactionAsync(
        UpdateManualTransactionCommand command,
        CancellationToken cancellationToken)
    {
        var workspace = await GetOrCreateDefaultWorkspaceAsync(cancellationToken).ConfigureAwait(false);
        await EnsureIncomeBudgetNodesAsync(workspace.Id, cancellationToken).ConfigureAwait(false);
        var transaction = await dbContext.LedgerTransactions
            .Include(candidate => candidate.Entries)
            .Include(candidate => candidate.Tags)
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

        var envelopeId = SystemEnvelopeId(command.Kind, command.EnvelopeId);
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
        await ReplaceTransactionTagsAsync(
            workspace.Id,
            transaction,
            command.Tags,
            DateTimeOffset.UtcNow,
            cancellationToken).ConfigureAwait(false);

        await dbContext.SaveChangesAsync(cancellationToken).ConfigureAwait(false);
        return await ReadTransactionAsync(workspace.Id, transaction.Id, cancellationToken).ConfigureAwait(false);
    }

    public async Task<IReadOnlyList<MovementReadModel>> GetMovementsAsync(CancellationToken cancellationToken)
    {
        var workspace = await GetOrCreateDefaultWorkspaceAsync(cancellationToken).ConfigureAwait(false);
        await EnsureTrashBudgetNodesAsync(workspace.Id, cancellationToken).ConfigureAwait(false);
        await RestoreOrphanedEnvelopeReferencesAsync(workspace.Id, cancellationToken).ConfigureAwait(false);

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
                Tags = entry.LedgerTransaction.Tags
                    .Select(transactionTag => transactionTag.Tag!)
                    .OrderBy(tag => tag.Name)
                    .Select(tag => new TagReadModel(tag.Id, tag.Name, tag.NormalizedName, tag.Color))
                    .ToList(),
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
                    entry.Notes,
                    entry.Tags);
            })
            .ToList();
    }

    public async Task<BudgetTreeReadModel> GetBudgetTreeAsync(
        MonthlyPeriod period,
        CancellationToken cancellationToken)
    {
        var workspace = await GetOrCreateDefaultWorkspaceAsync(cancellationToken).ConfigureAwait(false);
        await EnsureBudgetTreeAsync(workspace.Id, cancellationToken).ConfigureAwait(false);
        await EnsureIncomeBudgetNodesAsync(workspace.Id, cancellationToken).ConfigureAwait(false);
        await EnsureTrashBudgetNodesAsync(workspace.Id, cancellationToken).ConfigureAwait(false);
        await EnsureGoalsBudgetNodesAsync(workspace.Id, cancellationToken).ConfigureAwait(false);
        await RestoreOrphanedEnvelopeReferencesAsync(workspace.Id, cancellationToken).ConfigureAwait(false);
        await AttachLegacyCategoryNamesAsync(workspace.Id, cancellationToken).ConfigureAwait(false);
        return await ReadBudgetTreeAsync(workspace.Id, period, cancellationToken).ConfigureAwait(false);
    }

    public async Task<BudgetTreeReadModel> SaveBudgetTreeAsync(
        MonthlyPeriod period,
        SaveBudgetTreeCommand command,
        CancellationToken cancellationToken)
    {
        var workspace = await GetOrCreateDefaultWorkspaceAsync(cancellationToken).ConfigureAwait(false);
        await EnsureIncomeBudgetNodesAsync(workspace.Id, cancellationToken).ConfigureAwait(false);
        await EnsureTrashBudgetNodesAsync(workspace.Id, cancellationToken).ConfigureAwait(false);
        await EnsureGoalsBudgetNodesAsync(workspace.Id, cancellationToken).ConfigureAwait(false);
        await RestoreOrphanedEnvelopeReferencesAsync(workspace.Id, cancellationToken).ConfigureAwait(false);
        await ReplaceBudgetTreeAsync(workspace.Id, command, cancellationToken).ConfigureAwait(false);
        await SaveBudgetPeriodAllocationsAsync(workspace.Id, period, command.Envelopes, cancellationToken).ConfigureAwait(false);
        await dbContext.SaveChangesAsync(cancellationToken).ConfigureAwait(false);
        await AttachLegacyCategoryNamesAsync(workspace.Id, cancellationToken).ConfigureAwait(false);
        return await ReadBudgetTreeAsync(workspace.Id, period, cancellationToken).ConfigureAwait(false);
    }

    public async Task<IReadOnlyList<TransactionReadModel>> GetTransactionsAsync(
        MonthlyPeriod period,
        CancellationToken cancellationToken)
    {
        var workspace = await GetOrCreateDefaultWorkspaceAsync(cancellationToken).ConfigureAwait(false);

        var transactions = await dbContext.LedgerTransactions
            .AsNoTracking()
            .Include(transaction => transaction.Entries)
            .Include(transaction => transaction.Tags)
                .ThenInclude(transactionTag => transactionTag.Tag)
            .Where(transaction =>
                transaction.WorkspaceId == workspace.Id
                && transaction.TransactionDate >= period.StartDate
                && transaction.TransactionDate < period.EndDate)
            .OrderByDescending(transaction => transaction.TransactionDate)
            .ThenByDescending(transaction => transaction.Id)
            .ToListAsync(cancellationToken)
            .ConfigureAwait(false);

        return transactions.Select(ToTransactionReadModel).ToList();
    }

    public async Task<MonthlyCashflowReadModel> GetMonthlyCashflowAsync(
        MonthlyPeriod period,
        CancellationToken cancellationToken)
    {
        var workspace = await GetOrCreateDefaultWorkspaceAsync(cancellationToken).ConfigureAwait(false);

        var transactions = await dbContext.LedgerTransactions
            .AsNoTracking()
            .Include(transaction => transaction.Entries)
            .Where(transaction =>
                transaction.WorkspaceId == workspace.Id
                && transaction.TransactionDate >= period.StartDate
                && transaction.TransactionDate < period.EndDate)
            .ToListAsync(cancellationToken)
            .ConfigureAwait(false);

        var income = SumClpTransactions(transactions, "income", "increase");
        var expenses = SumClpTransactions(transactions, "expense", "decrease");
        var accounts = await GetAccountsAsync(cancellationToken).ConfigureAwait(false);

        return new MonthlyCashflowReadModel(
            period.Key,
            workspace.BaseCurrency,
            income,
            expenses,
            income - expenses,
            accounts);
    }

    public async Task<MonthlyTagReportReadModel> GetMonthlyTagReportAsync(
        MonthlyPeriod period,
        CancellationToken cancellationToken)
    {
        var workspace = await GetOrCreateDefaultWorkspaceAsync(cancellationToken).ConfigureAwait(false);
        var transactions = await dbContext.LedgerTransactions
            .AsNoTracking()
            .Include(transaction => transaction.Entries)
            .Include(transaction => transaction.Tags)
                .ThenInclude(transactionTag => transactionTag.Tag)
            .Where(transaction =>
                transaction.WorkspaceId == workspace.Id
                && transaction.TransactionDate >= period.StartDate
                && transaction.TransactionDate < period.EndDate
                && (transaction.Kind == "income" || transaction.Kind == "expense"))
            .ToListAsync(cancellationToken)
            .ConfigureAwait(false);

        var rows = transactions
            .SelectMany(transaction => transaction.Tags.Select(transactionTag => new { Transaction = transaction, Tag = transactionTag.Tag! }))
            .Where(row => row.Tag is not null && TransactionReportAmount(row.Transaction) > 0)
            .GroupBy(row => row.Tag.Id)
            .Select(group =>
            {
                var tag = group.First().Tag;
                var income = group
                    .Where(row => row.Transaction.Kind == "income")
                    .Sum(row => TransactionReportAmount(row.Transaction));
                var expenses = group
                    .Where(row => row.Transaction.Kind == "expense")
                    .Sum(row => TransactionReportAmount(row.Transaction));

                return new TagReportRowReadModel(
                    ToTagReadModel(tag),
                    income,
                    expenses,
                    income - expenses,
                    group.Select(row => row.Transaction.Id).Distinct().Count());
            })
            .OrderByDescending(row => row.ExpenseTotal)
            .ThenBy(row => row.Tag.Name)
            .ToList();

        return new MonthlyTagReportReadModel(period.Key, workspace.BaseCurrency, rows);
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

    private async Task RestoreOrphanedEnvelopeReferencesAsync(Guid workspaceId, CancellationToken cancellationToken)
    {
        var orphanTransactions = await dbContext.LedgerTransactions
            .Where(transaction =>
                transaction.WorkspaceId == workspaceId
                && transaction.EnvelopeId != null
                && !dbContext.BudgetEnvelopes.Any(envelope =>
                    envelope.WorkspaceId == workspaceId
                    && envelope.Id == transaction.EnvelopeId.Value))
            .Select(transaction => new
            {
                EnvelopeId = transaction.EnvelopeId!.Value,
                transaction.CategoryName,
            })
            .ToListAsync(cancellationToken)
            .ConfigureAwait(false);

        if (orphanTransactions.Count == 0)
        {
            return;
        }

        var nextSortOrder = await dbContext.BudgetEnvelopes
            .Where(envelope => envelope.WorkspaceId == workspaceId)
            .Select(envelope => (int?)envelope.SortOrder)
            .MaxAsync(cancellationToken)
            .ConfigureAwait(false) ?? 0;

        foreach (var group in orphanTransactions.GroupBy(transaction => transaction.EnvelopeId))
        {
            nextSortOrder++;
            dbContext.BudgetEnvelopes.Add(new BudgetEnvelopeEntity
            {
                Id = group.Key,
                WorkspaceId = workspaceId,
                SubcategoryId = TrashSubcategoryId,
                Name = OrphanEnvelopeName(group.Select(transaction => transaction.CategoryName)),
                AssignedAmount = "0",
                SortOrder = nextSortOrder,
            });
        }

        await dbContext.SaveChangesAsync(cancellationToken).ConfigureAwait(false);
    }

    private static string OrphanEnvelopeName(IEnumerable<string?> categoryNames)
    {
        var categoryName = categoryNames.FirstOrDefault(candidate => !string.IsNullOrWhiteSpace(candidate));
        return categoryName is null ? "Sobre eliminado" : categoryName.Trim();
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

    private async Task EnsureGoalsBudgetNodesAsync(Guid workspaceId, CancellationToken cancellationToken)
    {
        var category = await dbContext.BudgetCategories
            .SingleOrDefaultAsync(
                candidate => candidate.WorkspaceId == workspaceId && candidate.Id == GoalsCategoryId,
                cancellationToken)
            .ConfigureAwait(false);
        if (category is null)
        {
            dbContext.BudgetCategories.Add(new BudgetCategoryEntity
            {
                Id = GoalsCategoryId,
                WorkspaceId = workspaceId,
                Name = "Metas",
                SortOrder = 2,
            });
        }

        var subcategory = await dbContext.BudgetSubcategories
            .SingleOrDefaultAsync(
                candidate => candidate.WorkspaceId == workspaceId && candidate.Id == GoalsSubcategoryId,
                cancellationToken)
            .ConfigureAwait(false);
        if (subcategory is null)
        {
            dbContext.BudgetSubcategories.Add(new BudgetSubcategoryEntity
            {
                Id = GoalsSubcategoryId,
                WorkspaceId = workspaceId,
                CategoryId = GoalsCategoryId,
                Name = "Reservas y metas",
                SortOrder = 0,
            });
        }

        await dbContext.SaveChangesAsync(cancellationToken).ConfigureAwait(false);
    }

    private async Task EnsureIncomeBudgetNodesAsync(Guid workspaceId, CancellationToken cancellationToken)
    {
        var category = await dbContext.BudgetCategories
            .SingleOrDefaultAsync(
                candidate => candidate.WorkspaceId == workspaceId && candidate.Id == IncomeCategoryId,
                cancellationToken)
            .ConfigureAwait(false);
        if (category is null)
        {
            dbContext.BudgetCategories.Add(new BudgetCategoryEntity
            {
                Id = IncomeCategoryId,
                WorkspaceId = workspaceId,
                Name = "Ingresos",
                SortOrder = 0,
            });
        }

        var subcategory = await dbContext.BudgetSubcategories
            .SingleOrDefaultAsync(
                candidate => candidate.WorkspaceId == workspaceId && candidate.Id == IncomeSubcategoryId,
                cancellationToken)
            .ConfigureAwait(false);
        if (subcategory is null)
        {
            dbContext.BudgetSubcategories.Add(new BudgetSubcategoryEntity
            {
                Id = IncomeSubcategoryId,
                WorkspaceId = workspaceId,
                CategoryId = IncomeCategoryId,
                Name = "Ingresos recibidos",
                SortOrder = 0,
            });
        }

        var envelope = await dbContext.BudgetEnvelopes
            .SingleOrDefaultAsync(
                candidate => candidate.WorkspaceId == workspaceId && candidate.Id == IncomeEnvelopeId,
                cancellationToken)
            .ConfigureAwait(false);
        if (envelope is null)
        {
            dbContext.BudgetEnvelopes.Add(new BudgetEnvelopeEntity
            {
                Id = IncomeEnvelopeId,
                WorkspaceId = workspaceId,
                SubcategoryId = IncomeSubcategoryId,
                Name = "Ingresos",
                AssignedAmount = "0",
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
        MonthlyPeriod period,
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
        var goals = await dbContext.BudgetGoals
            .AsNoTracking()
            .Where(goal => goal.WorkspaceId == workspaceId)
            .OrderBy(goal => goal.Status)
            .ThenBy(goal => goal.Id)
            .ToListAsync(cancellationToken)
            .ConfigureAwait(false);
        var allocations = await dbContext.BudgetPeriodAllocations
            .AsNoTracking()
            .Where(allocation => allocation.WorkspaceId == workspaceId && allocation.PeriodKey == period.Key)
            .ToDictionaryAsync(
                allocation => allocation.EnvelopeId,
                allocation => allocation.AssignedAmount,
                cancellationToken)
            .ConfigureAwait(false);
        var activityAmounts = await ReadBudgetActivityAmountsAsync(workspaceId, period, cancellationToken).ConfigureAwait(false);

        return new BudgetTreeReadModel(
            period.Key,
            categories.Select(ToBudgetCategoryReadModel).ToList(),
            subcategories.Select(ToBudgetSubcategoryReadModel).ToList(),
            envelopes.Select(envelope => ToBudgetEnvelopeReadModel(envelope, allocations, activityAmounts)).ToList(),
            goals.Select(ToBudgetGoalReadModel).ToList());
    }

    private async Task<IReadOnlyDictionary<Guid, decimal>> ReadBudgetActivityAmountsAsync(
        Guid workspaceId,
        MonthlyPeriod period,
        CancellationToken cancellationToken)
    {
        var transactions = await dbContext.LedgerTransactions
            .AsNoTracking()
            .Include(transaction => transaction.Entries)
            .Where(transaction =>
                transaction.WorkspaceId == workspaceId
                && transaction.EnvelopeId != null
                && transaction.TransactionDate >= period.StartDate
                && transaction.TransactionDate < period.EndDate
                && (transaction.Kind == "income" || transaction.Kind == "expense"))
            .ToListAsync(cancellationToken)
            .ConfigureAwait(false);

        return transactions
            .GroupBy(transaction => transaction.EnvelopeId!.Value)
            .ToDictionary(
                group => group.Key,
                group => group.Sum(BudgetActivityAmount));
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
        await UpsertBudgetGoalsAsync(workspaceId, command.Goals, cancellationToken).ConfigureAwait(false);
    }

    private async Task RemoveDeletedBudgetNodesAsync(
        Guid workspaceId,
        SaveBudgetTreeCommand command,
        CancellationToken cancellationToken)
    {
        var categoryIds = command.Categories.Select(category => category.Id).ToHashSet();
        var subcategoryIds = command.Subcategories.Select(subcategory => subcategory.Id).ToHashSet();
        var envelopeIds = command.Envelopes.Select(envelope => envelope.Id).ToHashSet();
        categoryIds.Add(IncomeCategoryId);
        categoryIds.Add(TrashCategoryId);
        categoryIds.Add(GoalsCategoryId);
        subcategoryIds.Add(IncomeSubcategoryId);
        subcategoryIds.Add(TrashSubcategoryId);
        subcategoryIds.Add(GoalsSubcategoryId);
        envelopeIds.Add(IncomeEnvelopeId);
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
        await RemoveDeletedBudgetGoalsAsync(workspaceId, command.Goals, cancellationToken).ConfigureAwait(false);
        dbContext.BudgetSubcategories.RemoveRange(dbContext.BudgetSubcategories.Where(subcategory => subcategory.WorkspaceId == workspaceId && !subcategoryIds.Contains(subcategory.Id)));
        dbContext.BudgetCategories.RemoveRange(dbContext.BudgetCategories.Where(category => category.WorkspaceId == workspaceId && !categoryIds.Contains(category.Id)));
    }

    private async Task RemoveDeletedBudgetGoalsAsync(
        Guid workspaceId,
        IReadOnlyList<SaveBudgetGoalCommand> commands,
        CancellationToken cancellationToken)
    {
        var goalIds = commands.Select(goal => goal.Id).ToHashSet();
        var goals = await dbContext.BudgetGoals
            .Where(goal => goal.WorkspaceId == workspaceId && !goalIds.Contains(goal.Id))
            .ToListAsync(cancellationToken)
            .ConfigureAwait(false);

        dbContext.BudgetGoals.RemoveRange(goals);
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
            if (command.Id == IncomeCategoryId || command.Id == TrashCategoryId)
            {
                continue;
            }

            if (command.Id == GoalsCategoryId)
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
            if (command.Id == IncomeSubcategoryId || command.Id == TrashSubcategoryId)
            {
                continue;
            }

            if (command.Id == GoalsSubcategoryId)
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
            if (command.Id == IncomeEnvelopeId)
            {
                continue;
            }

            var envelope = await dbContext.BudgetEnvelopes
                .SingleOrDefaultAsync(candidate => candidate.WorkspaceId == workspaceId && candidate.Id == command.Id, cancellationToken)
                .ConfigureAwait(false);
            if (envelope is null)
            {
                dbContext.BudgetEnvelopes.Add(new BudgetEnvelopeEntity { Id = command.Id, WorkspaceId = workspaceId, SubcategoryId = command.SubcategoryId, Name = command.Name.Trim(), AssignedAmount = "0", SortOrder = command.SortOrder });
                continue;
            }

            envelope.SubcategoryId = command.SubcategoryId;
            envelope.Name = command.Name.Trim();
            envelope.SortOrder = command.SortOrder;
        }
    }

    private async Task SaveBudgetPeriodAllocationsAsync(
        Guid workspaceId,
        MonthlyPeriod period,
        IReadOnlyList<SaveBudgetEnvelopeCommand> commands,
        CancellationToken cancellationToken)
    {
        var allocations = await dbContext.BudgetPeriodAllocations
            .Where(allocation => allocation.WorkspaceId == workspaceId && allocation.PeriodKey == period.Key)
            .ToListAsync(cancellationToken)
            .ConfigureAwait(false);
        var now = DateTimeOffset.UtcNow;

        foreach (var command in commands)
        {
            var assignedAmount = FormatAmount(decimal.Parse(command.AssignedAmount, CultureInfo.InvariantCulture));
            var allocation = allocations.SingleOrDefault(candidate => candidate.EnvelopeId == command.Id);
            if (allocation is null)
            {
                dbContext.BudgetPeriodAllocations.Add(new BudgetPeriodAllocationEntity
                {
                    Id = Guid.CreateVersion7(),
                    WorkspaceId = workspaceId,
                    PeriodKey = period.Key,
                    EnvelopeId = command.Id,
                    AssignedAmount = assignedAmount,
                    CreatedAt = now,
                    UpdatedAt = now,
                });
                continue;
            }

            allocation.AssignedAmount = assignedAmount;
            allocation.UpdatedAt = now;
        }
    }

    private async Task UpsertBudgetGoalsAsync(
        Guid workspaceId,
        IReadOnlyList<SaveBudgetGoalCommand> commands,
        CancellationToken cancellationToken)
    {
        foreach (var command in commands)
        {
            var goal = await dbContext.BudgetGoals
                .SingleOrDefaultAsync(candidate => candidate.WorkspaceId == workspaceId && candidate.Id == command.Id, cancellationToken)
                .ConfigureAwait(false);
            var targetAmount = FormatAmount(decimal.Parse(command.TargetAmount, CultureInfo.InvariantCulture));
            if (goal is null)
            {
                dbContext.BudgetGoals.Add(new BudgetGoalEntity
                {
                    Id = command.Id,
                    WorkspaceId = workspaceId,
                    EnvelopeId = command.EnvelopeId,
                    TargetAmount = targetAmount,
                    DeadlineDate = command.DeadlineDate.Trim(),
                    Status = command.Status,
                });
                continue;
            }

            goal.EnvelopeId = command.EnvelopeId;
            goal.TargetAmount = targetAmount;
            goal.DeadlineDate = command.DeadlineDate.Trim();
            goal.Status = command.Status;
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
        IReadOnlyList<string>? tags,
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
            tags,
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

    private async Task ReplaceTransactionTagsAsync(
        Guid workspaceId,
        LedgerTransactionEntity transaction,
        IReadOnlyList<string>? tagNames,
        DateTimeOffset now,
        CancellationToken cancellationToken)
    {
        dbContext.TransactionTags.RemoveRange(transaction.Tags);
        transaction.Tags.Clear();
        if (tagNames is null || tagNames.Count == 0)
        {
            return;
        }

        var normalizedNames = tagNames.Select(TransactionTagNames.Normalize).ToList();
        var tags = await dbContext.Tags
            .Where(tag => tag.WorkspaceId == workspaceId && normalizedNames.Contains(tag.NormalizedName))
            .ToListAsync(cancellationToken)
            .ConfigureAwait(false);

        foreach (var tagName in tagNames)
        {
            var normalizedName = TransactionTagNames.Normalize(tagName);
            var tag = tags.SingleOrDefault(candidate => candidate.NormalizedName == normalizedName);
            if (tag is null)
            {
                    tag = new TagEntity
                    {
                        Id = Guid.CreateVersion7(),
                        WorkspaceId = workspaceId,
                        Name = TransactionTagNames.DisplayName(tagName),
                        NormalizedName = normalizedName,
                        Color = "#64748B",
                        CreatedAt = now,
                    };
                tags.Add(tag);
                dbContext.Tags.Add(tag);
            }

            var transactionTag = new TransactionTagEntity
            {
                LedgerTransactionId = transaction.Id,
                TagId = tag.Id,
                LedgerTransaction = transaction,
                Tag = tag,
            };
            transaction.Tags.Add(transactionTag);
            dbContext.TransactionTags.Add(transactionTag);
        }
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
            transaction.Notes,
            transaction.Tags
                .Where(transactionTag => transactionTag.Tag is not null)
                .Select(transactionTag => ToTagReadModel(transactionTag.Tag!))
                .OrderBy(tag => tag.Name)
                .ToList());
    }

    private async Task<TransactionReadModel> ReadTransactionAsync(
        Guid workspaceId,
        Guid transactionId,
        CancellationToken cancellationToken)
    {
        var transaction = await dbContext.LedgerTransactions
            .AsNoTracking()
            .Include(candidate => candidate.Entries)
            .Include(candidate => candidate.Envelope)
            .Include(candidate => candidate.Tags)
                .ThenInclude(transactionTag => transactionTag.Tag)
            .SingleAsync(
                candidate => candidate.WorkspaceId == workspaceId && candidate.Id == transactionId,
                cancellationToken)
            .ConfigureAwait(false);

        return ToTransactionReadModel(transaction);
    }

    private static TagReadModel ToTagReadModel(TagEntity tag)
    {
        return new TagReadModel(tag.Id, tag.Name, tag.NormalizedName, tag.Color);
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

    private static BudgetEnvelopeReadModel ToBudgetEnvelopeReadModel(
        BudgetEnvelopeEntity envelope,
        IReadOnlyDictionary<Guid, string> assignedAmounts,
        IReadOnlyDictionary<Guid, decimal> activityAmounts)
    {
        var assignedAmount = assignedAmounts.TryGetValue(envelope.Id, out var assignedValue)
            ? decimal.Parse(assignedValue, CultureInfo.InvariantCulture)
            : 0;
        var activityAmount = activityAmounts.GetValueOrDefault(envelope.Id);

        return new BudgetEnvelopeReadModel(
            envelope.Id,
            envelope.SubcategoryId,
            envelope.Name,
            assignedAmount,
            activityAmount,
            assignedAmount - activityAmount,
            envelope.SortOrder);
    }

    private static BudgetGoalReadModel ToBudgetGoalReadModel(BudgetGoalEntity goal)
    {
        return new BudgetGoalReadModel(
            goal.Id,
            goal.EnvelopeId,
            decimal.Parse(goal.TargetAmount, CultureInfo.InvariantCulture),
            goal.DeadlineDate,
            goal.Status);
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

    private static decimal TransactionReportAmount(LedgerTransactionEntity transaction)
    {
        var entry = PrimaryEntry(transaction);
        if (entry.Currency != "CLP")
        {
            return 0;
        }

        if (transaction.Kind == "income" && entry.AccountEffect == "increase")
        {
            return decimal.Parse(entry.AmountValue, CultureInfo.InvariantCulture);
        }

        if (transaction.Kind == "expense" && entry.AccountEffect == "decrease")
        {
            return decimal.Parse(entry.AmountValue, CultureInfo.InvariantCulture);
        }

        return 0;
    }

    private static decimal BudgetActivityAmount(LedgerTransactionEntity transaction)
    {
        var entry = PrimaryEntry(transaction);
        if (entry.Currency != "CLP")
        {
            return 0;
        }

        if (transaction.Kind == "income" && entry.AccountEffect == "increase")
        {
            return decimal.Parse(entry.AmountValue, CultureInfo.InvariantCulture);
        }

        if (transaction.Kind == "expense" && entry.AccountEffect == "decrease")
        {
            return decimal.Parse(entry.AmountValue, CultureInfo.InvariantCulture);
        }

        return 0;
    }

    private static bool TryParseAmount(string value, out decimal amount)
    {
        return decimal.TryParse(value, NumberStyles.Number, CultureInfo.InvariantCulture, out amount);
    }

    private static string? EmptyToNull(string? value)
    {
        return string.IsNullOrWhiteSpace(value) ? null : value.Trim();
    }

    private static Guid? SystemEnvelopeId(string kind, Guid? envelopeId)
    {
        if (kind == "transfer")
        {
            return null;
        }

        if (kind == "income" && envelopeId is null)
        {
            return IncomeEnvelopeId;
        }

        return envelopeId;
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

}
