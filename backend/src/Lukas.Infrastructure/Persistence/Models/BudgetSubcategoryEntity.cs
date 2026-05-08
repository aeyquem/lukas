namespace Lukas.Infrastructure.Persistence.Models;

public sealed class BudgetSubcategoryEntity
{
    public Guid Id { get; set; }

    public Guid WorkspaceId { get; set; }

    public Guid CategoryId { get; set; }

    public string Name { get; set; } = string.Empty;

    public int SortOrder { get; set; }

    public WorkspaceEntity? Workspace { get; set; }

    public BudgetCategoryEntity? Category { get; set; }

    public ICollection<BudgetEnvelopeEntity> Envelopes { get; } = new List<BudgetEnvelopeEntity>();
}
