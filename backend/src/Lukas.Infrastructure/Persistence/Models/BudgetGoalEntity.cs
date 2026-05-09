namespace Lukas.Infrastructure.Persistence.Models;

public sealed class BudgetGoalEntity
{
    public Guid Id { get; set; }

    public Guid WorkspaceId { get; set; }

    public Guid EnvelopeId { get; set; }

    public string TargetAmount { get; set; } = "0";

    public string DeadlineDate { get; set; } = string.Empty;

    public string Status { get; set; } = "active";

    public WorkspaceEntity? Workspace { get; set; }

    public BudgetEnvelopeEntity? Envelope { get; set; }
}
