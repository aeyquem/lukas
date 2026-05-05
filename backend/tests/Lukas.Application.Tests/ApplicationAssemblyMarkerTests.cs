using Lukas.Application;
using Shouldly;

namespace Lukas.Application.Tests;

public sealed class ApplicationAssemblyMarkerTests
{
    [Fact]
    public void ApplicationAssemblyIsLoadable()
    {
        typeof(ApplicationAssemblyMarker).Assembly.GetName().Name.ShouldBe("Lukas.Application");
    }
}
