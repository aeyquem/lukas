namespace Lukas.Infrastructure.Persistence.Models;

public sealed class SourceRecordEntity
{
    public Guid Id { get; set; }

    public Guid WorkspaceId { get; set; }

    public string SourceType { get; set; } = string.Empty;

    public string RawSnapshot { get; set; } = string.Empty;

    public string NormalizedSnapshot { get; set; } = string.Empty;

    public DateTimeOffset CreatedAt { get; set; }

    public WorkspaceEntity? Workspace { get; set; }
}
