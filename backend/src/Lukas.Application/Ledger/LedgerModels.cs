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
    DateOnly TransactionDate,
    string Amount,
    string Currency,
    string AccountEffect,
    string Description,
    Guid? EnvelopeId,
    string? CategoryName,
    string? Notes);

public sealed record UpdateManualTransactionCommand(
    Guid Id,
    string Kind,
    Guid AccountId,
    DateOnly TransactionDate,
    string Amount,
    string Currency,
    string AccountEffect,
    string Description,
    Guid? EnvelopeId,
    string? CategoryName,
    string? Notes);

public sealed record TransactionReadModel(
    Guid Id,
    string Kind,
    Guid AccountId,
    DateOnly TransactionDate,
    decimal Amount,
    string Currency,
    string AccountEffect,
    string Description,
    Guid? EnvelopeId,
    string? EnvelopeName,
    string? CategoryName,
    string? Notes);

public sealed record MovementReadModel(
    Guid Id,
    Guid TransactionId,
    DateOnly TransactionDate,
    Guid AccountId,
    string AccountName,
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
    string? Notes);

public sealed record MonthlyCashflowReadModel(
    string Period,
    string Currency,
    decimal IncomeTotal,
    decimal ExpenseTotal,
    decimal NetCashflow,
    IReadOnlyList<AccountReadModel> Accounts);

public sealed record BudgetTreeReadModel(
    IReadOnlyList<BudgetCategoryReadModel> Categories,
    IReadOnlyList<BudgetSubcategoryReadModel> Subcategories,
    IReadOnlyList<BudgetEnvelopeReadModel> Envelopes);

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
    int SortOrder);

public sealed record SaveBudgetTreeCommand(
    IReadOnlyList<SaveBudgetCategoryCommand> Categories,
    IReadOnlyList<SaveBudgetSubcategoryCommand> Subcategories,
    IReadOnlyList<SaveBudgetEnvelopeCommand> Envelopes);

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
