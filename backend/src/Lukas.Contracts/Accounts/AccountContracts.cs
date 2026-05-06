using System.ComponentModel.DataAnnotations;

namespace Lukas.Contracts.Accounts;

public sealed record CreateAccountRequest(
    [Required, MinLength(1), MaxLength(120)] string DisplayName,
    [Required, RegularExpression("^(cash|bank|credit_card)$")] string Type,
    [Required, RegularExpression("^[A-Z]{3}$")] string Currency,
    [RegularExpression(@"^-?\d+(\.\d+)?$")] string? OpeningBalance);

public sealed record AccountDto(
    Guid Id,
    string DisplayName,
    string Type,
    string Currency,
    string CurrentBalance);
