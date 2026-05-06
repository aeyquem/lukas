namespace Lukas.Infrastructure.Persistence.Models;

public sealed class LedgerEntryEntity
{
    public Guid Id { get; set; }

    public Guid WorkspaceId { get; set; }

    public Guid LedgerTransactionId { get; set; }

    public Guid AccountId { get; set; }

    public string AmountValue { get; set; } = string.Empty;

    public string Currency { get; set; } = string.Empty;

    public string AccountEffect { get; set; } = string.Empty;

    public LedgerTransactionEntity? LedgerTransaction { get; set; }

    public AccountEntity? Account { get; set; }
}
