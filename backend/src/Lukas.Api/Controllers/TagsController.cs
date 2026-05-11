using Lukas.Application.Ledger;
using Lukas.Contracts.Tags;
using Microsoft.AspNetCore.Mvc;

namespace Lukas.Api.Controllers;

[ApiController]
[Route("api/tags")]
public sealed class TagsController(LedgerService ledgerService) : ControllerBase
{
    [HttpGet]
    public async Task<ActionResult<IReadOnlyList<TagDto>>> GetTags(CancellationToken cancellationToken)
    {
        var tags = await ledgerService.GetTagsAsync(cancellationToken).ConfigureAwait(false);
        return Ok(tags.Select(ToDto).ToList());
    }

    [HttpPost]
    public async Task<ActionResult<TagDto>> CreateTag(
        CreateTagRequest request,
        CancellationToken cancellationToken)
    {
        var result = await ledgerService
            .CreateTagAsync(new CreateTagCommand(request.Name, request.Color), cancellationToken)
            .ConfigureAwait(false);

        if (result.IsFailure)
        {
            return Problem(statusCode: StatusCodes.Status400BadRequest, title: result.Error.Message, type: result.Error.Code);
        }

        var dto = ToDto(result.Value);
        return CreatedAtAction(nameof(GetTags), new { }, dto);
    }

    [HttpPut("{id:guid}")]
    public async Task<ActionResult<TagDto>> UpdateTag(
        Guid id,
        UpdateTagRequest request,
        CancellationToken cancellationToken)
    {
        var result = await ledgerService
            .UpdateTagAsync(new UpdateTagCommand(id, request.Name, request.Color), cancellationToken)
            .ConfigureAwait(false);

        if (result.IsFailure)
        {
            return Problem(statusCode: StatusCodes.Status400BadRequest, title: result.Error.Message, type: result.Error.Code);
        }

        return Ok(ToDto(result.Value));
    }

    [HttpDelete("{id:guid}")]
    public async Task<IActionResult> DeleteTag(Guid id, CancellationToken cancellationToken)
    {
        var result = await ledgerService.DeleteTagAsync(id, cancellationToken).ConfigureAwait(false);
        if (result.IsFailure)
        {
            return Problem(statusCode: StatusCodes.Status400BadRequest, title: result.Error.Message, type: result.Error.Code);
        }

        return NoContent();
    }

    private static TagDto ToDto(TagReadModel tag)
    {
        return new TagDto(tag.Id, tag.Name, tag.NormalizedName, tag.Color);
    }
}
