using System.ComponentModel.DataAnnotations;

namespace Lukas.Contracts.Transactions;

public sealed record CreateManualTransactionRequest(
    [Required, RegularExpression("^(income|expense|adjustment)$")] string Kind,
    [Required] Guid AccountId,
    [Required] DateOnly TransactionDate,
    [Required, RegularExpression(@"^\d+(\.\d+)?$")] string Amount,
    [Required, RegularExpression("^[A-Z]{3}$")] string Currency,
    [Required, RegularExpression("^(increase|decrease)$")] string AccountEffect,
    [Required, MinLength(1), MaxLength(240)] string Description,
    [MaxLength(120)] string? CategoryName,
    [MaxLength(1000)] string? Notes);

public sealed record UpdateManualTransactionRequest(
    [Required, RegularExpression("^(income|expense|adjustment)$")] string Kind,
    [Required] Guid AccountId,
    [Required] DateOnly TransactionDate,
    [Required, RegularExpression(@"^\d+(\.\d+)?$")] string Amount,
    [Required, RegularExpression("^[A-Z]{3}$")] string Currency,
    [Required, RegularExpression("^(increase|decrease)$")] string AccountEffect,
    [Required, MinLength(1), MaxLength(240)] string Description,
    [MaxLength(120)] string? CategoryName,
    [MaxLength(1000)] string? Notes);

public sealed record TransactionDto(
    Guid Id,
    string Kind,
    Guid AccountId,
    DateOnly TransactionDate,
    string Amount,
    string Currency,
    string AccountEffect,
    string Description,
    string? CategoryName,
    string? Notes);
