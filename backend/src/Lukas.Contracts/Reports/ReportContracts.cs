using Lukas.Contracts.Tags;

namespace Lukas.Contracts.Reports;

public sealed record MonthlyTagReportDto(
    string Period,
    string Currency,
    IReadOnlyList<TagReportRowDto> Tags);

public sealed record TagReportRowDto(
    TagDto Tag,
    string IncomeTotal,
    string ExpenseTotal,
    string NetTotal,
    int TransactionCount);
