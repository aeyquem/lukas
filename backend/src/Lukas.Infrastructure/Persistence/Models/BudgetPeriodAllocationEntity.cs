namespace Lukas.Infrastructure.Persistence.Models;

public sealed class BudgetPeriodAllocationEntity
{
    public Guid Id { get; set; }

    public Guid WorkspaceId { get; set; }

    public string PeriodKey { get; set; } = string.Empty;

    public Guid EnvelopeId { get; set; }

    public string AssignedAmount { get; set; } = "0";

    public DateTimeOffset CreatedAt { get; set; }

    public DateTimeOffset UpdatedAt { get; set; }

    public WorkspaceEntity? Workspace { get; set; }

    public BudgetEnvelopeEntity? Envelope { get; set; }
}
