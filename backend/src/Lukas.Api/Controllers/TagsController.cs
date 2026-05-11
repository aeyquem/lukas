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
        return Ok(tags.Select(tag => new TagDto(tag.Id, tag.Name, tag.NormalizedName)).ToList());
    }
}
