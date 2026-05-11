namespace Lukas.Infrastructure.Persistence.Models;

public sealed class TransactionTagEntity
{
    public Guid LedgerTransactionId { get; set; }

    public Guid TagId { get; set; }

    public LedgerTransactionEntity? LedgerTransaction { get; set; }

    public TagEntity? Tag { get; set; }
}
