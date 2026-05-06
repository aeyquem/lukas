namespace Lukas.Infrastructure.Persistence.Models;

public sealed class TransactionSourceLinkEntity
{
    public Guid LedgerTransactionId { get; set; }

    public Guid SourceRecordId { get; set; }

    public LedgerTransactionEntity? LedgerTransaction { get; set; }

    public SourceRecordEntity? SourceRecord { get; set; }
}
