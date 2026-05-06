using Lukas.Application.Ledger;
using Lukas.Contracts.Accounts;
using Microsoft.AspNetCore.Mvc;

namespace Lukas.Api.Controllers;

[ApiController]
[Route("api/accounts")]
public sealed class AccountsController(LedgerService ledgerService) : ControllerBase
{
    [HttpGet]
    public async Task<ActionResult<IReadOnlyList<AccountDto>>> GetAccounts(CancellationToken cancellationToken)
    {
        var accounts = await ledgerService.GetAccountsAsync(cancellationToken).ConfigureAwait(false);
        return Ok(accounts.Select(ToDto).ToList());
    }

    [HttpPost]
    public async Task<ActionResult<AccountDto>> CreateAccount(
        CreateAccountRequest request,
        CancellationToken cancellationToken)
    {
        var result = await ledgerService
            .CreateAccountAsync(
                new CreateAccountCommand(
                    request.DisplayName,
                    request.Type,
                    request.Currency,
                    request.OpeningBalance),
                cancellationToken)
            .ConfigureAwait(false);

        if (result.IsFailure)
        {
            return Problem(statusCode: StatusCodes.Status400BadRequest, title: result.Error.Message, type: result.Error.Code);
        }

        var dto = ToDto(result.Value);
        return CreatedAtAction(nameof(GetAccounts), new { id = dto.Id }, dto);
    }

    private static AccountDto ToDto(AccountReadModel account)
    {
        return new AccountDto(
            account.Id,
            account.DisplayName,
            account.Type,
            account.Currency,
            FormatAmount(account.CurrentBalance));
    }

    private static string FormatAmount(decimal amount)
    {
        return amount.ToString("0.#############################", System.Globalization.CultureInfo.InvariantCulture);
    }
}
