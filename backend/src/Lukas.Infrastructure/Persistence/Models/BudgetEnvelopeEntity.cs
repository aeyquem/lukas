namespace Lukas.Infrastructure.Persistence.Models;

public sealed class BudgetEnvelopeEntity
{
    public Guid Id { get; set; }

    public Guid WorkspaceId { get; set; }

    public Guid SubcategoryId { get; set; }

    public string Name { get; set; } = string.Empty;

    public string AssignedAmount { get; set; } = "0";

    public int SortOrder { get; set; }

    public WorkspaceEntity? Workspace { get; set; }

    public BudgetSubcategoryEntity? Subcategory { get; set; }
}
