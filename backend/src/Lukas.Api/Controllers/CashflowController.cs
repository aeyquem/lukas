using Lukas.Application.Ledger;
using Lukas.Contracts.Accounts;
using Lukas.Contracts.Cashflow;
using Microsoft.AspNetCore.Mvc;

namespace Lukas.Api.Controllers;

[ApiController]
[Route("api/cashflow")]
public sealed class CashflowController(LedgerService ledgerService) : ControllerBase
{
    [HttpGet("monthly")]
    public async Task<ActionResult<MonthlyCashflowDto>> GetMonthlyCashflow(
        [FromQuery] string period,
        CancellationToken cancellationToken)
    {
        var result = await ledgerService.GetMonthlyCashflowAsync(period, cancellationToken).ConfigureAwait(false);

        if (result.IsFailure)
        {
            return Problem(statusCode: StatusCodes.Status400BadRequest, title: result.Error.Message, type: result.Error.Code);
        }

        var cashflow = result.Value;
        return Ok(new MonthlyCashflowDto(
            cashflow.Period,
            cashflow.Currency,
            FormatAmount(cashflow.IncomeTotal),
            FormatAmount(cashflow.ExpenseTotal),
            FormatAmount(cashflow.NetCashflow),
            cashflow.Accounts.Select(account => new AccountDto(
                account.Id,
                account.DisplayName,
                account.Type,
                account.Currency,
                FormatAmount(account.CurrentBalance))).ToList()));
    }

    private static string FormatAmount(decimal amount)
    {
        return amount.ToString("0.#############################", System.Globalization.CultureInfo.InvariantCulture);
    }
}
