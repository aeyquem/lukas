using Lukas.Infrastructure.Persistence;
using Microsoft.EntityFrameworkCore.Infrastructure;
using Microsoft.EntityFrameworkCore.Migrations;

#nullable disable

namespace Lukas.Infrastructure.Persistence.Migrations;

[DbContext(typeof(LukasDbContext))]
[Migration("20260509010000_AddBudgetPeriodAllocations")]
public partial class AddBudgetPeriodAllocations : Migration
{
    protected override void Up(MigrationBuilder migrationBuilder)
    {
        migrationBuilder.CreateTable(
            name: "budget_period_allocations",
            columns: table => new
            {
                id = table.Column<Guid>(type: "TEXT", nullable: false),
                workspace_id = table.Column<Guid>(type: "TEXT", nullable: false),
                period_key = table.Column<string>(type: "TEXT", maxLength: 7, nullable: false),
                envelope_id = table.Column<Guid>(type: "TEXT", nullable: false),
                assigned_amount = table.Column<string>(type: "TEXT", maxLength: 80, nullable: false),
                created_at = table.Column<DateTimeOffset>(type: "TEXT", nullable: false),
                updated_at = table.Column<DateTimeOffset>(type: "TEXT", nullable: false),
            },
            constraints: table =>
            {
                table.PrimaryKey("PK_budget_period_allocations", x => x.id);
                table.ForeignKey(
                    name: "FK_budget_period_allocations_budget_envelopes_envelope_id",
                    column: x => x.envelope_id,
                    principalTable: "budget_envelopes",
                    principalColumn: "id",
                    onDelete: ReferentialAction.Cascade);
                table.ForeignKey(
                    name: "FK_budget_period_allocations_workspaces_workspace_id",
                    column: x => x.workspace_id,
                    principalTable: "workspaces",
                    principalColumn: "id",
                    onDelete: ReferentialAction.Cascade);
            });

        migrationBuilder.Sql(
            """
            INSERT INTO budget_period_allocations (id, workspace_id, period_key, envelope_id, assigned_amount, created_at, updated_at)
            SELECT
                lower(hex(randomblob(4))) || '-' || lower(hex(randomblob(2))) || '-7' || substr(lower(hex(randomblob(2))), 2) || '-' || substr('89ab', abs(random()) % 4 + 1, 1) || substr(lower(hex(randomblob(2))), 2) || '-' || lower(hex(randomblob(6))),
                workspace_id,
                strftime('%Y-%m', 'now'),
                id,
                assigned_amount,
                strftime('%Y-%m-%dT%H:%M:%f+00:00', 'now'),
                strftime('%Y-%m-%dT%H:%M:%f+00:00', 'now')
            FROM budget_envelopes
            WHERE assigned_amount != '0'
            """);

        migrationBuilder.CreateIndex(name: "IX_budget_period_allocations_envelope_id", table: "budget_period_allocations", column: "envelope_id");
        migrationBuilder.CreateIndex(name: "IX_budget_period_allocations_workspace_id_period_key", table: "budget_period_allocations", columns: new[] { "workspace_id", "period_key" });
        migrationBuilder.CreateIndex(name: "IX_budget_period_allocations_workspace_id_period_key_envelope_id", table: "budget_period_allocations", columns: new[] { "workspace_id", "period_key", "envelope_id" }, unique: true);
    }

    protected override void Down(MigrationBuilder migrationBuilder)
    {
        migrationBuilder.DropTable(name: "budget_period_allocations");
    }
}
