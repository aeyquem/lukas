using Lukas.Infrastructure.Persistence;
using Shouldly;

namespace Lukas.Infrastructure.Tests;

public sealed class InfrastructureAssemblyMarkerTests
{
    [Fact]
    public void DbContextTypeIsInInfrastructureAssembly()
    {
        typeof(LukasDbContext).Assembly.GetName().Name.ShouldBe("Lukas.Infrastructure");
    }
}
