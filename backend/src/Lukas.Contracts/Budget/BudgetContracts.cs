using System.ComponentModel.DataAnnotations;

namespace Lukas.Contracts.Budget;

public sealed record BudgetTreeDto(
    string Period,
    IReadOnlyList<BudgetCategoryDto> Categories,
    IReadOnlyList<BudgetSubcategoryDto> Subcategories,
    IReadOnlyList<BudgetEnvelopeDto> Envelopes,
    IReadOnlyList<BudgetGoalDto> Goals);

public sealed record BudgetCategoryDto(
    Guid Id,
    string Name,
    int SortOrder);

public sealed record BudgetSubcategoryDto(
    Guid Id,
    Guid CategoryId,
    string Name,
    int SortOrder);

public sealed record BudgetEnvelopeDto(
    Guid Id,
    Guid SubcategoryId,
    string Name,
    string AssignedAmount,
    string ActivityAmount,
    string RemainingAmount,
    int SortOrder);

public sealed record BudgetGoalDto(
    Guid Id,
    Guid EnvelopeId,
    string TargetAmount,
    string DeadlineDate,
    string Status);

public sealed record SaveBudgetTreeRequest(
    [Required] IReadOnlyList<SaveBudgetCategoryRequest> Categories,
    [Required] IReadOnlyList<SaveBudgetSubcategoryRequest> Subcategories,
    [Required] IReadOnlyList<SaveBudgetEnvelopeRequest> Envelopes,
    [Required] IReadOnlyList<SaveBudgetGoalRequest> Goals);

public sealed record SaveBudgetCategoryRequest(
    Guid Id,
    [Required, MinLength(1), MaxLength(120)] string Name,
    int SortOrder);

public sealed record SaveBudgetSubcategoryRequest(
    Guid Id,
    Guid CategoryId,
    [Required, MinLength(1), MaxLength(120)] string Name,
    int SortOrder);

public sealed record SaveBudgetEnvelopeRequest(
    Guid Id,
    Guid SubcategoryId,
    [Required, MinLength(1), MaxLength(120)] string Name,
    [Required, RegularExpression(@"^\d+(\.\d+)?$")] string AssignedAmount,
    int SortOrder);

public sealed record SaveBudgetGoalRequest(
    Guid Id,
    Guid EnvelopeId,
    [Required, RegularExpression(@"^\d+(\.\d+)?$")] string TargetAmount,
    [Required, RegularExpression(@"^$|^\d{4}-\d{2}-\d{2}$")] string DeadlineDate,
    [Required, RegularExpression(@"^(active|completed|archived)$")] string Status);
