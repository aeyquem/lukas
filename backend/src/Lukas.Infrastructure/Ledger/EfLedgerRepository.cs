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

        var envelopeName = await GetEnvelopeNameAsync(workspace.Id, command.EnvelopeId, cancellationToken).ConfigureAwait(false);
        _ = TryParseAmount(command.Amount, out var amount);
        var transaction = AddManualTransaction(
            workspace.Id,
            account.Id,
            command.Kind,
            command.TransactionDate,
            amount,
            command.Currency,
            command.AccountEffect,
            command.Description.Trim(),
            command.EnvelopeId,
            envelopeName ?? EmptyToNull(command.CategoryName),
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

        var envelopeName = await GetEnvelopeNameAsync(workspace.Id, command.EnvelopeId, cancellationToken).ConfigureAwait(false);
        _ = TryParseAmount(command.Amount, out var amount);
        var entry = transaction.Entries.Single();
        transaction.Kind = command.Kind;
        transaction.TransactionDate = command.TransactionDate;
        transaction.Description = command.Description.Trim();
        transaction.EnvelopeId = command.EnvelopeId;
        transaction.CategoryName = envelopeName ?? EmptyToNull(command.CategoryName);
        transaction.Notes = string.IsNullOrWhiteSpace(command.Notes) ? null : command.Notes.Trim();
        entry.AccountId = account.Id;
        entry.AmountValue = FormatAmount(amount);
        entry.Currency = command.Currency;
        entry.AccountEffect = command.AccountEffect;

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
                entry.LedgerTransaction.Kind,
                entry.AccountEffect,
                entry.AmountValue,
                entry.Currency,
                entry.LedgerTransaction.Description,
                entry.LedgerTransaction.EnvelopeId,
                EnvelopeName = entry.LedgerTransaction.Envelope == null ? null : entry.LedgerTransaction.Envelope.Name,
                entry.LedgerTransaction.CategoryName,
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
        await AttachLegacyCategoryNamesAsync(workspace.Id, cancellationToken).ConfigureAwait(false);
        return await ReadBudgetTreeAsync(workspace.Id, cancellationToken).ConfigureAwait(false);
    }

    public async Task<BudgetTreeReadModel> SaveBudgetTreeAsync(
        SaveBudgetTreeCommand command,
        CancellationToken cancellationToken)
    {
        var workspace = await GetOrCreateDefaultWorkspaceAsync(cancellationToken).ConfigureAwait(false);
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
        var deletedEnvelopeIds = await dbContext.BudgetEnvelopes
            .Where(envelope => envelope.WorkspaceId == workspaceId && !envelopeIds.Contains(envelope.Id))
            .Select(envelope => envelope.Id)
            .ToListAsync(cancellationToken)
            .ConfigureAwait(false);
        var transactions = await dbContext.LedgerTransactions
            .Where(transaction => transaction.WorkspaceId == workspaceId && transaction.EnvelopeId != null && deletedEnvelopeIds.Contains(transaction.EnvelopeId.Value))
            .ToListAsync(cancellationToken)
            .ConfigureAwait(false);

        foreach (var transaction in transactions)
        {
            transaction.EnvelopeId = null;
        }

        dbContext.BudgetEnvelopes.RemoveRange(dbContext.BudgetEnvelopes.Where(envelope => envelope.WorkspaceId == workspaceId && !envelopeIds.Contains(envelope.Id)));
        dbContext.BudgetSubcategories.RemoveRange(dbContext.BudgetSubcategories.Where(subcategory => subcategory.WorkspaceId == workspaceId && !subcategoryIds.Contains(subcategory.Id)));
        dbContext.BudgetCategories.RemoveRange(dbContext.BudgetCategories.Where(category => category.WorkspaceId == workspaceId && !categoryIds.Contains(category.Id)));
    }

    private async Task UpsertBudgetCategoriesAsync(
        Guid workspaceId,
        IReadOnlyList<SaveBudgetCategoryCommand> commands,
        CancellationToken cancellationToken)
    {
        foreach (var command in commands)
        {
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

    private LedgerTransactionEntity AddManualTransaction(
        Guid workspaceId,
        Guid accountId,
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

        var entry = new LedgerEntryEntity
        {
            Id = Guid.CreateVersion7(),
            WorkspaceId = workspaceId,
            LedgerTransactionId = transaction.Id,
            AccountId = accountId,
            AmountValue = FormatAmount(amount),
            Currency = currency,
            AccountEffect = accountEffect,
        };

        var snapshots = JsonSerializer.Serialize(new
        {
            kind,
            accountId,
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
        dbContext.LedgerEntries.Add(entry);
        dbContext.SourceRecords.Add(sourceRecord);
        dbContext.TransactionSourceLinks.Add(new TransactionSourceLinkEntity
        {
            LedgerTransactionId = transaction.Id,
            SourceRecordId = sourceRecord.Id,
        });

        return transaction;
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
        var entry = transaction.Entries.Single();
        return new TransactionReadModel(
            transaction.Id,
            transaction.Kind,
            entry.AccountId,
            transaction.TransactionDate,
            decimal.Parse(entry.AmountValue, CultureInfo.InvariantCulture),
            entry.Currency,
            entry.AccountEffect,
            transaction.Description,
            transaction.EnvelopeId,
            transaction.Envelope?.Name,
            transaction.CategoryName,
            transaction.Notes);
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
