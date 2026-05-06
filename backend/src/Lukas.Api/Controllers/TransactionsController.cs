using Lukas.Application.Ledger;
using Lukas.Contracts.Transactions;
using Microsoft.AspNetCore.Mvc;

namespace Lukas.Api.Controllers;

[ApiController]
[Route("api/transactions")]
public sealed class TransactionsController(LedgerService ledgerService) : ControllerBase
{
    [HttpGet]
    public async Task<ActionResult<IReadOnlyList<TransactionDto>>> GetTransactions(
        [FromQuery] string period,
        CancellationToken cancellationToken)
    {
        var result = await ledgerService.GetTransactionsAsync(period, cancellationToken).ConfigureAwait(false);

        if (result.IsFailure)
        {
            return Problem(statusCode: StatusCodes.Status400BadRequest, title: result.Error.Message, type: result.Error.Code);
        }

        return Ok(result.Value.Select(ToDto).ToList());
    }

    [HttpPost]
    public async Task<ActionResult<TransactionDto>> CreateTransaction(
        CreateManualTransactionRequest request,
        CancellationToken cancellationToken)
    {
        var result = await ledgerService
            .CreateManualTransactionAsync(
                new CreateManualTransactionCommand(
                    request.Kind,
                    request.AccountId,
                    request.TransactionDate,
                    request.Amount,
                    request.Currency,
                    request.AccountEffect,
                    request.Description,
                    request.Notes),
                cancellationToken)
            .ConfigureAwait(false);

        if (result.IsFailure)
        {
            return Problem(statusCode: StatusCodes.Status400BadRequest, title: result.Error.Message, type: result.Error.Code);
        }

        var dto = ToDto(result.Value);
        return CreatedAtAction(nameof(GetTransactions), new { period = dto.TransactionDate.ToString("yyyy-MM") }, dto);
    }

    private static TransactionDto ToDto(TransactionReadModel transaction)
    {
        return new TransactionDto(
            transaction.Id,
            transaction.Kind,
            transaction.AccountId,
            transaction.TransactionDate,
            FormatAmount(transaction.Amount),
            transaction.Currency,
            transaction.AccountEffect,
            transaction.Description,
            transaction.Notes);
    }

    private static string FormatAmount(decimal amount)
    {
        return amount.ToString("0.#############################", System.Globalization.CultureInfo.InvariantCulture);
    }
}
