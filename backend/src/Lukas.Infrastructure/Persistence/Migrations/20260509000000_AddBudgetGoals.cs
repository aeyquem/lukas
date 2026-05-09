using Lukas.Infrastructure.Persistence;
using Microsoft.EntityFrameworkCore.Infrastructure;
using Microsoft.EntityFrameworkCore.Migrations;

#nullable disable

namespace Lukas.Infrastructure.Persistence.Migrations;

[DbContext(typeof(LukasDbContext))]
[Migration("20260509000000_AddBudgetGoals")]
public partial class AddBudgetGoals : Migration
{
    protected override void Up(MigrationBuilder migrationBuilder)
    {
        migrationBuilder.CreateTable(
            name: "budget_goals",
            columns: table => new
            {
                id = table.Column<Guid>(type: "TEXT", nullable: false),
                workspace_id = table.Column<Guid>(type: "TEXT", nullable: false),
                envelope_id = table.Column<Guid>(type: "TEXT", nullable: false),
                target_amount = table.Column<string>(type: "TEXT", maxLength: 80, nullable: false),
                deadline_date = table.Column<string>(type: "TEXT", maxLength: 10, nullable: false),
                status = table.Column<string>(type: "TEXT", maxLength: 20, nullable: false),
            },
            constraints: table =>
            {
                table.PrimaryKey("PK_budget_goals", x => x.id);
                table.ForeignKey(
                    name: "FK_budget_goals_budget_envelopes_envelope_id",
                    column: x => x.envelope_id,
                    principalTable: "budget_envelopes",
                    principalColumn: "id",
                    onDelete: ReferentialAction.Cascade);
                table.ForeignKey(
                    name: "FK_budget_goals_workspaces_workspace_id",
                    column: x => x.workspace_id,
                    principalTable: "workspaces",
                    principalColumn: "id",
                    onDelete: ReferentialAction.Cascade);
            });

        migrationBuilder.CreateIndex(name: "IX_budget_goals_envelope_id", table: "budget_goals", column: "envelope_id");
        migrationBuilder.CreateIndex(name: "IX_budget_goals_workspace_id_envelope_id", table: "budget_goals", columns: new[] { "workspace_id", "envelope_id" }, unique: true);
        migrationBuilder.CreateIndex(name: "IX_budget_goals_workspace_id_status", table: "budget_goals", columns: new[] { "workspace_id", "status" });
    }

    protected override void Down(MigrationBuilder migrationBuilder)
    {
        migrationBuilder.DropTable(name: "budget_goals");
    }
}
