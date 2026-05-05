using Lukas.Importers;
using Shouldly;

namespace Lukas.Importers.Tests;

public sealed class ImportersAssemblyMarkerTests
{
    [Fact]
    public void ImportersAssemblyIsLoadable()
    {
        typeof(ImportersAssemblyMarker).Assembly.GetName().Name.ShouldBe("Lukas.Importers");
    }
}
