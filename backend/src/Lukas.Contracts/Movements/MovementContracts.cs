namespace Lukas.Contracts.Movements;

public sealed record MovementDto(
    Guid Id,
    Guid TransactionId,
    DateOnly TransactionDate,
    Guid AccountId,
    string AccountName,
    Guid? DestinationAccountId,
    string Kind,
    string AccountEffect,
    string Direction,
    string OriginalAmount,
    string OriginalCurrency,
    string BaseAmount,
    string BaseCurrency,
    string Description,
    Guid? EnvelopeId,
    string? EnvelopeName,
    string? CategoryName,
    string? Notes);
