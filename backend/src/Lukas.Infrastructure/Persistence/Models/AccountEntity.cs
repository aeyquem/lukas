namespace Lukas.Infrastructure.Persistence.Models;

public sealed class AccountEntity
{
    public Guid Id { get; set; }

    public Guid WorkspaceId { get; set; }

    public string DisplayName { get; set; } = string.Empty;

    public string Type { get; set; } = string.Empty;

    public string Currency { get; set; } = string.Empty;

    public DateTimeOffset CreatedAt { get; set; }

    public WorkspaceEntity? Workspace { get; set; }
}
