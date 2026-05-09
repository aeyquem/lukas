using Lukas.Application.Ledger;
using Lukas.Contracts.Budget;
using Microsoft.AspNetCore.Mvc;

namespace Lukas.Api.Controllers;

[ApiController]
[Route("api/budget")]
public sealed class BudgetController(LedgerService ledgerService) : ControllerBase
{
    [HttpGet]
    public async Task<ActionResult<BudgetTreeDto>> GetBudgetTree(
        [FromQuery] string period,
        CancellationToken cancellationToken)
    {
        var result = await ledgerService.GetBudgetTreeAsync(period, cancellationToken).ConfigureAwait(false);
        if (result.IsFailure)
        {
            return Problem(statusCode: StatusCodes.Status400BadRequest, title: result.Error.Message, type: result.Error.Code);
        }

        return Ok(ToDto(result.Value));
    }

    [HttpPut]
    public async Task<ActionResult<BudgetTreeDto>> SaveBudgetTree(
        [FromQuery] string period,
        SaveBudgetTreeRequest request,
        CancellationToken cancellationToken)
    {
        var result = await ledgerService
            .SaveBudgetTreeAsync(period, ToCommand(request), cancellationToken)
            .ConfigureAwait(false);

        if (result.IsFailure)
        {
            return Problem(statusCode: StatusCodes.Status400BadRequest, title: result.Error.Message, type: result.Error.Code);
        }

        return Ok(ToDto(result.Value));
    }

    private static SaveBudgetTreeCommand ToCommand(SaveBudgetTreeRequest request)
    {
        return new SaveBudgetTreeCommand(
            request.Categories.Select(category => new SaveBudgetCategoryCommand(category.Id, category.Name, category.SortOrder)).ToList(),
            request.Subcategories.Select(subcategory => new SaveBudgetSubcategoryCommand(subcategory.Id, subcategory.CategoryId, subcategory.Name, subcategory.SortOrder)).ToList(),
            request.Envelopes.Select(envelope => new SaveBudgetEnvelopeCommand(envelope.Id, envelope.SubcategoryId, envelope.Name, envelope.AssignedAmount, envelope.SortOrder)).ToList(),
            request.Goals.Select(goal => new SaveBudgetGoalCommand(goal.Id, goal.EnvelopeId, goal.TargetAmount, goal.DeadlineDate, goal.Status)).ToList());
    }

    private static BudgetTreeDto ToDto(BudgetTreeReadModel tree)
    {
        return new BudgetTreeDto(
            tree.Period,
            tree.Categories.Select(category => new BudgetCategoryDto(category.Id, category.Name, category.SortOrder)).ToList(),
            tree.Subcategories.Select(subcategory => new BudgetSubcategoryDto(subcategory.Id, subcategory.CategoryId, subcategory.Name, subcategory.SortOrder)).ToList(),
            tree.Envelopes.Select(envelope => new BudgetEnvelopeDto(envelope.Id, envelope.SubcategoryId, envelope.Name, FormatAmount(envelope.AssignedAmount), FormatAmount(envelope.ActivityAmount), FormatAmount(envelope.RemainingAmount), envelope.SortOrder)).ToList(),
            tree.Goals.Select(goal => new BudgetGoalDto(goal.Id, goal.EnvelopeId, FormatAmount(goal.TargetAmount), goal.DeadlineDate, goal.Status)).ToList());
    }

    private static string FormatAmount(decimal amount)
    {
        return amount.ToString("0.#############################", System.Globalization.CultureInfo.InvariantCulture);
    }
}
