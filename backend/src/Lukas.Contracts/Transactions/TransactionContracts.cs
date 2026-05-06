using System.ComponentModel.DataAnnotations;

namespace Lukas.Contracts.Transactions;

public sealed record CreateManualTransactionRequest(
    [property: Required, RegularExpression("^(income|expense|adjustment)$")] string Kind,
    [property: Required] Guid AccountId,
    [property: Required] DateOnly TransactionDate,
    [property: Required, RegularExpression(@"^\d+(\.\d+)?$")] string Amount,
    [property: Required, RegularExpression("^[A-Z]{3}$")] string Currency,
    [property: Required, RegularExpression("^(increase|decrease)$")] string AccountEffect,
    [property: Required, MinLength(1), MaxLength(240)] string Description,
    [property: MaxLength(1000)] string? Notes);

public sealed record TransactionDto(
    Guid Id,
    string Kind,
    Guid AccountId,
    DateOnly TransactionDate,
    string Amount,
    string Currency,
    string AccountEffect,
    string Description,
    string? Notes);
