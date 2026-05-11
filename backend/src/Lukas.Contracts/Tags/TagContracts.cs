namespace Lukas.Contracts.Tags;

public sealed record TagDto(
    Guid Id,
    string Name,
    string NormalizedName,
    string Color);

public sealed record CreateTagRequest(
    string Name,
    string Color);

public sealed record UpdateTagRequest(
    string Name,
    string Color);
