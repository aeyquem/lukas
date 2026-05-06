namespace Lukas.Infrastructure.Persistence.Models;

public sealed class LedgerTransactionEntity
{
    public Guid Id { get; set; }

    public Guid WorkspaceId { get; set; }

    public string Kind { get; set; } = string.Empty;

    public DateOnly TransactionDate { get; set; }

    public string Description { get; set; } = string.Empty;

    public string? Notes { get; set; }

    public DateTimeOffset CreatedAt { get; set; }

    public WorkspaceEntity? Workspace { get; set; }

    public ICollection<LedgerEntryEntity> Entries { get; } = new List<LedgerEntryEntity>();
}
