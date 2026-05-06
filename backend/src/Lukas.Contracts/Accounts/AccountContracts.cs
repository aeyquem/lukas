using System.ComponentModel.DataAnnotations;

namespace Lukas.Contracts.Accounts;

public sealed record CreateAccountRequest(
    [property: Required, MinLength(1), MaxLength(120)] string DisplayName,
    [property: Required, RegularExpression("^(cash|bank|credit_card)$")] string Type,
    [property: Required, RegularExpression("^[A-Z]{3}$")] string Currency,
    [property: RegularExpression(@"^-?\d+(\.\d+)?$")] string? OpeningBalance);

public sealed record AccountDto(
    Guid Id,
    string DisplayName,
    string Type,
    string Currency,
    string CurrentBalance);
