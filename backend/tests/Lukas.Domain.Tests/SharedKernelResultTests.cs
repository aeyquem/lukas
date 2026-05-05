using Lukas.SharedKernel.Results;
using Shouldly;

namespace Lukas.Domain.Tests;

public sealed class SharedKernelResultTests
{
    [Fact]
    public void SuccessResultExposesValue()
    {
        var result = Result<string>.Success("ok");

        result.IsSuccess.ShouldBeTrue();
        result.Value.ShouldBe("ok");
    }
}
