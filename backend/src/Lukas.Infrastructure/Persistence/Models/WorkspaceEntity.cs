namespace Lukas.Infrastructure.Persistence.Models;

public sealed class WorkspaceEntity
{
    public Guid Id { get; set; }

    public string DisplayName { get; set; } = string.Empty;

    public string BaseCurrency { get; set; } = string.Empty;

    public string TimeZone { get; set; } = string.Empty;

    public DateTimeOffset CreatedAt { get; set; }
}
