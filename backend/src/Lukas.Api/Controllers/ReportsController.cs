using Lukas.Application.Ledger;
using Lukas.Contracts.Reports;
using Lukas.Contracts.Tags;
using Microsoft.AspNetCore.Mvc;

namespace Lukas.Api.Controllers;

[ApiController]
[Route("api/reports")]
public sealed class ReportsController(LedgerService ledgerService) : ControllerBase
{
    [HttpGet("tags")]
    public async Task<ActionResult<MonthlyTagReportDto>> GetMonthlyTagReport(
        [FromQuery] string period,
        CancellationToken cancellationToken)
    {
        var result = await ledgerService.GetMonthlyTagReportAsync(period, cancellationToken).ConfigureAwait(false);

        if (result.IsFailure)
        {
            return Problem(statusCode: StatusCodes.Status400BadRequest, title: result.Error.Message, type: result.Error.Code);
        }

        var report = result.Value;
        return Ok(new MonthlyTagReportDto(
            report.Period,
            report.Currency,
            report.Tags.Select(ToDto).ToList()));
    }

    private static TagReportRowDto ToDto(TagReportRowReadModel row)
    {
        return new TagReportRowDto(
            new TagDto(row.Tag.Id, row.Tag.Name, row.Tag.NormalizedName, row.Tag.Color),
            FormatAmount(row.IncomeTotal),
            FormatAmount(row.ExpenseTotal),
            FormatAmount(row.NetTotal),
            row.TransactionCount);
    }

    private static string FormatAmount(decimal amount)
    {
        return amount.ToString("0.#############################", System.Globalization.CultureInfo.InvariantCulture);
    }
}
