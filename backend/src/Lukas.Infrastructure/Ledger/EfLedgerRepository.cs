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
            string.IsNullOrWhiteSpace(command.Notes) ? null : command.Notes.Trim(),
            DateTimeOffset.UtcNow);

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
                entry.AccountEffect,
                entry.AmountValue,
                entry.Currency,
                entry.LedgerTransaction.Description,
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
                    entry.AccountEffect == "decrease" ? "out" : "in",
                    amount,
                    entry.Currency,
                    amount,
                    entry.Currency,
                    entry.Description,
                    null,
                    entry.Notes);
            })
            .ToList();
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

    private LedgerTransactionEntity AddManualTransaction(
        Guid workspaceId,
        Guid accountId,
        string kind,
        DateOnly transactionDate,
        decimal amount,
        string currency,
        string accountEffect,
        string description,
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
            transaction.Notes);
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
