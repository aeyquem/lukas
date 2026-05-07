namespace Lukas.Contracts.Movements;

public sealed record MovementDto(
    Guid Id,
    Guid TransactionId,
    DateOnly TransactionDate,
    Guid AccountId,
    string AccountName,
    string Direction,
    string OriginalAmount,
    string OriginalCurrency,
    string BaseAmount,
    string BaseCurrency,
    string Description);
