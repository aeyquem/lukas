namespace Lukas.Application.Ledger;

public sealed record CreateAccountCommand(
    string DisplayName,
    string Type,
    string Currency,
    string? OpeningBalance);

public sealed record AccountReadModel(
    Guid Id,
    string DisplayName,
    string Type,
    string Currency,
    decimal CurrentBalance);

public sealed record CreateManualTransactionCommand(
    string Kind,
    Guid AccountId,
    Guid? DestinationAccountId,
    DateOnly TransactionDate,
    string Amount,
    string Currency,
    string AccountEffect,
    string Description,
    Guid? EnvelopeId,
    string? CategoryName,
    string? Notes,
    IReadOnlyList<string>? Tags = null);

public sealed record UpdateManualTransactionCommand(
    Guid Id,
    string Kind,
    Guid AccountId,
    Guid? DestinationAccountId,
    DateOnly TransactionDate,
    string Amount,
    string Currency,
    string AccountEffect,
    string Description,
    Guid? EnvelopeId,
    string? CategoryName,
    string? Notes,
    IReadOnlyList<string>? Tags = null);

public sealed record TagReadModel(
    Guid Id,
    string Name,
    string NormalizedName);

public sealed record TransactionReadModel(
    Guid Id,
    string Kind,
    Guid AccountId,
    Guid? DestinationAccountId,
    DateOnly TransactionDate,
    decimal Amount,
    string Currency,
    string AccountEffect,
    string Description,
    Guid? EnvelopeId,
    string? EnvelopeName,
    string? CategoryName,
    string? Notes,
    IReadOnlyList<TagReadModel> Tags);

public sealed record MovementReadModel(
    Guid Id,
    Guid TransactionId,
    DateOnly TransactionDate,
    Guid AccountId,
    string AccountName,
    Guid? DestinationAccountId,
    string Kind,
    string AccountEffect,
    string Direction,
    decimal OriginalAmount,
    string OriginalCurrency,
    decimal BaseAmount,
    string BaseCurrency,
    string Description,
    Guid? EnvelopeId,
    string? EnvelopeName,
    string? CategoryName,
    string? Notes,
    IReadOnlyList<TagReadModel> Tags);

public sealed record MonthlyTagReportReadModel(
    string Period,
    string Currency,
    IReadOnlyList<TagReportRowReadModel> Tags);

public sealed record TagReportRowReadModel(
    TagReadModel Tag,
    decimal IncomeTotal,
    decimal ExpenseTotal,
    decimal NetTotal,
    int TransactionCount);

public sealed record MonthlyCashflowReadModel(
    string Period,
    string Currency,
    decimal IncomeTotal,
    decimal ExpenseTotal,
    decimal NetCashflow,
    IReadOnlyList<AccountReadModel> Accounts);

public sealed record BudgetTreeReadModel(
    string Period,
    IReadOnlyList<BudgetCategoryReadModel> Categories,
    IReadOnlyList<BudgetSubcategoryReadModel> Subcategories,
    IReadOnlyList<BudgetEnvelopeReadModel> Envelopes,
    IReadOnlyList<BudgetGoalReadModel> Goals);

public sealed record BudgetCategoryReadModel(
    Guid Id,
    string Name,
    int SortOrder);

public sealed record BudgetSubcategoryReadModel(
    Guid Id,
    Guid CategoryId,
    string Name,
    int SortOrder);

public sealed record BudgetEnvelopeReadModel(
    Guid Id,
    Guid SubcategoryId,
    string Name,
    decimal AssignedAmount,
    decimal ActivityAmount,
    decimal RemainingAmount,
    int SortOrder);

public sealed record BudgetGoalReadModel(
    Guid Id,
    Guid EnvelopeId,
    decimal TargetAmount,
    string DeadlineDate,
    string Status);

public sealed record SaveBudgetTreeCommand(
    IReadOnlyList<SaveBudgetCategoryCommand> Categories,
    IReadOnlyList<SaveBudgetSubcategoryCommand> Subcategories,
    IReadOnlyList<SaveBudgetEnvelopeCommand> Envelopes,
    IReadOnlyList<SaveBudgetGoalCommand> Goals);

public sealed record SaveBudgetCategoryCommand(
    Guid Id,
    string Name,
    int SortOrder);

public sealed record SaveBudgetSubcategoryCommand(
    Guid Id,
    Guid CategoryId,
    string Name,
    int SortOrder);

public sealed record SaveBudgetEnvelopeCommand(
    Guid Id,
    Guid SubcategoryId,
    string Name,
    string AssignedAmount,
    int SortOrder);

public sealed record SaveBudgetGoalCommand(
    Guid Id,
    Guid EnvelopeId,
    string TargetAmount,
    string DeadlineDate,
    string Status);
