namespace Lukas.Infrastructure.Persistence.Models;

public sealed class BudgetCategoryEntity
{
    public Guid Id { get; set; }

    public Guid WorkspaceId { get; set; }

    public string Name { get; set; } = string.Empty;

    public int SortOrder { get; set; }

    public WorkspaceEntity? Workspace { get; set; }

    public ICollection<BudgetSubcategoryEntity> Subcategories { get; } = new List<BudgetSubcategoryEntity>();
}
