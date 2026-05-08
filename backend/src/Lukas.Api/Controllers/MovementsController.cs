using Lukas.Application.Ledger;
using Lukas.Contracts.Movements;
using Microsoft.AspNetCore.Mvc;

namespace Lukas.Api.Controllers;

[ApiController]
[Route("api/movements")]
public sealed class MovementsController(LedgerService ledgerService) : ControllerBase
{
    [HttpGet]
    public async Task<ActionResult<IReadOnlyList<MovementDto>>> GetMovements(CancellationToken cancellationToken)
    {
        var movements = await ledgerService.GetMovementsAsync(cancellationToken).ConfigureAwait(false);
        return Ok(movements.Select(ToDto).ToList());
    }

    private static MovementDto ToDto(MovementReadModel movement)
    {
        return new MovementDto(
            movement.Id,
            movement.TransactionId,
            movement.TransactionDate,
            movement.AccountId,
            movement.AccountName,
            movement.Kind,
            movement.AccountEffect,
            movement.Direction,
            FormatAmount(movement.OriginalAmount),
            movement.OriginalCurrency,
            FormatAmount(movement.BaseAmount),
            movement.BaseCurrency,
            movement.Description,
            movement.EnvelopeId,
            movement.EnvelopeName,
            movement.CategoryName,
            movement.Notes);
    }

    private static string FormatAmount(decimal amount)
    {
        return amount.ToString("0.#############################", System.Globalization.CultureInfo.InvariantCulture);
    }
}
