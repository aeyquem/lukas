using Lukas.Infrastructure.Persistence;
using Microsoft.EntityFrameworkCore.Infrastructure;
using Microsoft.EntityFrameworkCore.Migrations;

#nullable disable

namespace Lukas.Infrastructure.Persistence.Migrations;

[DbContext(typeof(LukasDbContext))]
[Migration("20260508010000_AddBudgetEnvelopes")]
public partial class AddBudgetEnvelopes : Migration
{
    protected override void Up(MigrationBuilder migrationBuilder)
    {
        migrationBuilder.AddColumn<Guid>(
            name: "envelope_id",
            table: "ledger_transactions",
            type: "TEXT",
            nullable: true);

        migrationBuilder.CreateTable(
            name: "budget_categories",
            columns: table => new
            {
                id = table.Column<Guid>(type: "TEXT", nullable: false),
                workspace_id = table.Column<Guid>(type: "TEXT", nullable: false),
                name = table.Column<string>(type: "TEXT", maxLength: 120, nullable: false),
                sort_order = table.Column<int>(type: "INTEGER", nullable: false),
            },
            constraints: table =>
            {
                table.PrimaryKey("PK_budget_categories", x => x.id);
                table.ForeignKey(
                    name: "FK_budget_categories_workspaces_workspace_id",
                    column: x => x.workspace_id,
                    principalTable: "workspaces",
                    principalColumn: "id",
                    onDelete: ReferentialAction.Cascade);
            });

        migrationBuilder.CreateTable(
            name: "budget_subcategories",
            columns: table => new
            {
                id = table.Column<Guid>(type: "TEXT", nullable: false),
                workspace_id = table.Column<Guid>(type: "TEXT", nullable: false),
                category_id = table.Column<Guid>(type: "TEXT", nullable: false),
                name = table.Column<string>(type: "TEXT", maxLength: 120, nullable: false),
                sort_order = table.Column<int>(type: "INTEGER", nullable: false),
            },
            constraints: table =>
            {
                table.PrimaryKey("PK_budget_subcategories", x => x.id);
                table.ForeignKey(
                    name: "FK_budget_subcategories_budget_categories_category_id",
                    column: x => x.category_id,
                    principalTable: "budget_categories",
                    principalColumn: "id",
                    onDelete: ReferentialAction.Cascade);
                table.ForeignKey(
                    name: "FK_budget_subcategories_workspaces_workspace_id",
                    column: x => x.workspace_id,
                    principalTable: "workspaces",
                    principalColumn: "id",
                    onDelete: ReferentialAction.Cascade);
            });

        migrationBuilder.CreateTable(
            name: "budget_envelopes",
            columns: table => new
            {
                id = table.Column<Guid>(type: "TEXT", nullable: false),
                workspace_id = table.Column<Guid>(type: "TEXT", nullable: false),
                subcategory_id = table.Column<Guid>(type: "TEXT", nullable: false),
                name = table.Column<string>(type: "TEXT", maxLength: 120, nullable: false),
                assigned_amount = table.Column<string>(type: "TEXT", maxLength: 80, nullable: false),
                sort_order = table.Column<int>(type: "INTEGER", nullable: false),
            },
            constraints: table =>
            {
                table.PrimaryKey("PK_budget_envelopes", x => x.id);
                table.ForeignKey(
                    name: "FK_budget_envelopes_budget_subcategories_subcategory_id",
                    column: x => x.subcategory_id,
                    principalTable: "budget_subcategories",
                    principalColumn: "id",
                    onDelete: ReferentialAction.Cascade);
                table.ForeignKey(
                    name: "FK_budget_envelopes_workspaces_workspace_id",
                    column: x => x.workspace_id,
                    principalTable: "workspaces",
                    principalColumn: "id",
                    onDelete: ReferentialAction.Cascade);
            });

        migrationBuilder.CreateIndex(name: "IX_ledger_transactions_envelope_id", table: "ledger_transactions", column: "envelope_id");
        migrationBuilder.CreateIndex(name: "IX_budget_categories_workspace_id_sort_order", table: "budget_categories", columns: new[] { "workspace_id", "sort_order" });
        migrationBuilder.CreateIndex(name: "IX_budget_envelopes_workspace_id_name", table: "budget_envelopes", columns: new[] { "workspace_id", "name" });
        migrationBuilder.CreateIndex(name: "IX_budget_envelopes_workspace_id_subcategory_id_sort_order", table: "budget_envelopes", columns: new[] { "workspace_id", "subcategory_id", "sort_order" });
        migrationBuilder.CreateIndex(name: "IX_budget_envelopes_subcategory_id", table: "budget_envelopes", column: "subcategory_id");
        migrationBuilder.CreateIndex(name: "IX_budget_subcategories_workspace_id_category_id_sort_order", table: "budget_subcategories", columns: new[] { "workspace_id", "category_id", "sort_order" });
        migrationBuilder.CreateIndex(name: "IX_budget_subcategories_category_id", table: "budget_subcategories", column: "category_id");

    }

    protected override void Down(MigrationBuilder migrationBuilder)
    {
        migrationBuilder.DropTable(name: "budget_envelopes");
        migrationBuilder.DropTable(name: "budget_subcategories");
        migrationBuilder.DropTable(name: "budget_categories");
        migrationBuilder.DropIndex(name: "IX_ledger_transactions_envelope_id", table: "ledger_transactions");
        migrationBuilder.DropColumn(name: "envelope_id", table: "ledger_transactions");
    }
}
