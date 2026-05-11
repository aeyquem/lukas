namespace Lukas.Infrastructure.Persistence.Models;

public sealed class TagEntity
{
    public Guid Id { get; set; }

    public Guid WorkspaceId { get; set; }

    public string Name { get; set; } = string.Empty;

    public string NormalizedName { get; set; } = string.Empty;

    public DateTimeOffset CreatedAt { get; set; }

    public WorkspaceEntity? Workspace { get; set; }
}
