using System;
using Lukas.Infrastructure.Persistence;
using Microsoft.EntityFrameworkCore;
using Microsoft.EntityFrameworkCore.Infrastructure;
using Microsoft.EntityFrameworkCore.Migrations;

#nullable disable

namespace Lukas.Infrastructure.Persistence.Migrations;

[DbContext(typeof(LukasDbContext))]
[Migration("20260505000000_InitialLedgerSlice")]
partial class InitialLedgerSlice
{
    protected override void BuildTargetModel(ModelBuilder modelBuilder)
    {
        modelBuilder
            .HasAnnotation("ProductVersion", "10.0.7");

        LukasDbContextModelSnapshot.ConfigureModel(modelBuilder);
    }
}
