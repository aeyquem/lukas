using Microsoft.EntityFrameworkCore;

namespace Lukas.Infrastructure.Persistence;

public sealed class LukasDbContext(DbContextOptions<LukasDbContext> options) : DbContext(options)
{
    protected override void OnModelCreating(ModelBuilder modelBuilder)
    {
        base.OnModelCreating(modelBuilder);
        modelBuilder.HasDefaultSchema(null);
    }
}
