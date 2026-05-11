using Lukas.Infrastructure.Persistence;
using Microsoft.EntityFrameworkCore.Infrastructure;
using Microsoft.EntityFrameworkCore.Migrations;

#nullable disable

namespace Lukas.Infrastructure.Persistence.Migrations;

[DbContext(typeof(LukasDbContext))]
[Migration("20260511010000_AddTagColor")]
public partial class AddTagColor : Migration
{
    protected override void Up(MigrationBuilder migrationBuilder)
    {
        migrationBuilder.AddColumn<string>(
            name: "color",
            table: "tags",
            type: "TEXT",
            maxLength: 7,
            nullable: false,
            defaultValue: "#64748B");
    }

    protected override void Down(MigrationBuilder migrationBuilder)
    {
        migrationBuilder.DropColumn(
            name: "color",
            table: "tags");
    }
}
