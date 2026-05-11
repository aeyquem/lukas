namespace Lukas.Contracts.Tags;

public sealed record TagDto(
    Guid Id,
    string Name,
    string NormalizedName);
