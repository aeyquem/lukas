using Lukas.Infrastructure.Persistence;
using Microsoft.EntityFrameworkCore.Infrastructure;
using Microsoft.EntityFrameworkCore.Migrations;

#nullable disable

namespace Lukas.Infrastructure.Persistence.Migrations;

[DbContext(typeof(LukasDbContext))]
[Migration("20260511000000_AddTransactionTags")]
public partial class AddTransactionTags : Migration
{
    protected override void Up(MigrationBuilder migrationBuilder)
    {
        migrationBuilder.CreateTable(
            name: "tags",
            columns: table => new
            {
                id = table.Column<Guid>(type: "TEXT", nullable: false),
                workspace_id = table.Column<Guid>(type: "TEXT", nullable: false),
                name = table.Column<string>(type: "TEXT", maxLength: 60, nullable: false),
                normalized_name = table.Column<string>(type: "TEXT", maxLength: 60, nullable: false),
                created_at = table.Column<DateTimeOffset>(type: "TEXT", nullable: false),
            },
            constraints: table =>
            {
                table.PrimaryKey("PK_tags", tag => tag.id);
                table.ForeignKey(
                    name: "FK_tags_workspaces_workspace_id",
                    column: tag => tag.workspace_id,
                    principalTable: "workspaces",
                    principalColumn: "id",
                    onDelete: ReferentialAction.Cascade);
            });

        migrationBuilder.CreateTable(
            name: "transaction_tags",
            columns: table => new
            {
                ledger_transaction_id = table.Column<Guid>(type: "TEXT", nullable: false),
                tag_id = table.Column<Guid>(type: "TEXT", nullable: false),
            },
            constraints: table =>
            {
                table.PrimaryKey("PK_transaction_tags", transactionTag => new { transactionTag.ledger_transaction_id, transactionTag.tag_id });
                table.ForeignKey(
                    name: "FK_transaction_tags_ledger_transactions_ledger_transaction_id",
                    column: transactionTag => transactionTag.ledger_transaction_id,
                    principalTable: "ledger_transactions",
                    principalColumn: "id",
                    onDelete: ReferentialAction.Cascade);
                table.ForeignKey(
                    name: "FK_transaction_tags_tags_tag_id",
                    column: transactionTag => transactionTag.tag_id,
                    principalTable: "tags",
                    principalColumn: "id",
                    onDelete: ReferentialAction.Cascade);
            });

        migrationBuilder.CreateIndex(
            name: "IX_tags_workspace_id_normalized_name",
            table: "tags",
            columns: new[] { "workspace_id", "normalized_name" },
            unique: true);

        migrationBuilder.CreateIndex(
            name: "IX_transaction_tags_tag_id",
            table: "transaction_tags",
            column: "tag_id");
    }

    protected override void Down(MigrationBuilder migrationBuilder)
    {
        migrationBuilder.DropTable(name: "transaction_tags");
        migrationBuilder.DropTable(name: "tags");
    }
}
