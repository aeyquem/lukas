using System;
using Microsoft.EntityFrameworkCore.Migrations;

#nullable disable

namespace Lukas.Infrastructure.Persistence.Migrations;

public partial class InitialLedgerSlice : Migration
{
    protected override void Up(MigrationBuilder migrationBuilder)
    {
        migrationBuilder.CreateTable(
            name: "workspaces",
            columns: table => new
            {
                id = table.Column<Guid>(type: "TEXT", nullable: false),
                display_name = table.Column<string>(type: "TEXT", maxLength: 120, nullable: false),
                base_currency = table.Column<string>(type: "TEXT", maxLength: 3, nullable: false),
                time_zone = table.Column<string>(type: "TEXT", maxLength: 80, nullable: false),
                created_at = table.Column<DateTimeOffset>(type: "TEXT", nullable: false),
            },
            constraints: table =>
            {
                table.PrimaryKey("PK_workspaces", x => x.id);
            });

        migrationBuilder.CreateTable(
            name: "accounts",
            columns: table => new
            {
                id = table.Column<Guid>(type: "TEXT", nullable: false),
                workspace_id = table.Column<Guid>(type: "TEXT", nullable: false),
                display_name = table.Column<string>(type: "TEXT", maxLength: 120, nullable: false),
                type = table.Column<string>(type: "TEXT", maxLength: 40, nullable: false),
                currency = table.Column<string>(type: "TEXT", maxLength: 3, nullable: false),
                created_at = table.Column<DateTimeOffset>(type: "TEXT", nullable: false),
            },
            constraints: table =>
            {
                table.PrimaryKey("PK_accounts", x => x.id);
                table.ForeignKey(
                    name: "FK_accounts_workspaces_workspace_id",
                    column: x => x.workspace_id,
                    principalTable: "workspaces",
                    principalColumn: "id",
                    onDelete: ReferentialAction.Cascade);
            });

        migrationBuilder.CreateTable(
            name: "ledger_transactions",
            columns: table => new
            {
                id = table.Column<Guid>(type: "TEXT", nullable: false),
                workspace_id = table.Column<Guid>(type: "TEXT", nullable: false),
                kind = table.Column<string>(type: "TEXT", maxLength: 40, nullable: false),
                transaction_date = table.Column<DateOnly>(type: "TEXT", nullable: false),
                description = table.Column<string>(type: "TEXT", maxLength: 240, nullable: false),
                notes = table.Column<string>(type: "TEXT", maxLength: 1000, nullable: true),
                created_at = table.Column<DateTimeOffset>(type: "TEXT", nullable: false),
            },
            constraints: table =>
            {
                table.PrimaryKey("PK_ledger_transactions", x => x.id);
                table.ForeignKey(
                    name: "FK_ledger_transactions_workspaces_workspace_id",
                    column: x => x.workspace_id,
                    principalTable: "workspaces",
                    principalColumn: "id",
                    onDelete: ReferentialAction.Cascade);
            });

        migrationBuilder.CreateTable(
            name: "source_records",
            columns: table => new
            {
                id = table.Column<Guid>(type: "TEXT", nullable: false),
                workspace_id = table.Column<Guid>(type: "TEXT", nullable: false),
                source_type = table.Column<string>(type: "TEXT", maxLength: 40, nullable: false),
                raw_snapshot = table.Column<string>(type: "TEXT", nullable: false),
                normalized_snapshot = table.Column<string>(type: "TEXT", nullable: false),
                created_at = table.Column<DateTimeOffset>(type: "TEXT", nullable: false),
            },
            constraints: table =>
            {
                table.PrimaryKey("PK_source_records", x => x.id);
                table.ForeignKey(
                    name: "FK_source_records_workspaces_workspace_id",
                    column: x => x.workspace_id,
                    principalTable: "workspaces",
                    principalColumn: "id",
                    onDelete: ReferentialAction.Cascade);
            });

        migrationBuilder.CreateTable(
            name: "ledger_entries",
            columns: table => new
            {
                id = table.Column<Guid>(type: "TEXT", nullable: false),
                workspace_id = table.Column<Guid>(type: "TEXT", nullable: false),
                ledger_transaction_id = table.Column<Guid>(type: "TEXT", nullable: false),
                account_id = table.Column<Guid>(type: "TEXT", nullable: false),
                amount_value = table.Column<string>(type: "TEXT", maxLength: 80, nullable: false),
                currency = table.Column<string>(type: "TEXT", maxLength: 3, nullable: false),
                account_effect = table.Column<string>(type: "TEXT", maxLength: 20, nullable: false),
            },
            constraints: table =>
            {
                table.PrimaryKey("PK_ledger_entries", x => x.id);
                table.ForeignKey(
                    name: "FK_ledger_entries_accounts_account_id",
                    column: x => x.account_id,
                    principalTable: "accounts",
                    principalColumn: "id",
                    onDelete: ReferentialAction.Cascade);
                table.ForeignKey(
                    name: "FK_ledger_entries_ledger_transactions_ledger_transaction_id",
                    column: x => x.ledger_transaction_id,
                    principalTable: "ledger_transactions",
                    principalColumn: "id",
                    onDelete: ReferentialAction.Cascade);
            });

        migrationBuilder.CreateTable(
            name: "transaction_source_links",
            columns: table => new
            {
                ledger_transaction_id = table.Column<Guid>(type: "TEXT", nullable: false),
                source_record_id = table.Column<Guid>(type: "TEXT", nullable: false),
            },
            constraints: table =>
            {
                table.PrimaryKey("PK_transaction_source_links", x => new { x.ledger_transaction_id, x.source_record_id });
                table.ForeignKey(
                    name: "FK_transaction_source_links_ledger_transactions_ledger_transaction_id",
                    column: x => x.ledger_transaction_id,
                    principalTable: "ledger_transactions",
                    principalColumn: "id",
                    onDelete: ReferentialAction.Cascade);
                table.ForeignKey(
                    name: "FK_transaction_source_links_source_records_source_record_id",
                    column: x => x.source_record_id,
                    principalTable: "source_records",
                    principalColumn: "id",
                    onDelete: ReferentialAction.Cascade);
            });

        migrationBuilder.CreateIndex(name: "IX_accounts_workspace_id", table: "accounts", column: "workspace_id");
        migrationBuilder.CreateIndex(name: "IX_ledger_entries_account_id", table: "ledger_entries", column: "account_id");
        migrationBuilder.CreateIndex(name: "IX_ledger_entries_ledger_transaction_id", table: "ledger_entries", column: "ledger_transaction_id");
        migrationBuilder.CreateIndex(name: "IX_ledger_transactions_workspace_id", table: "ledger_transactions", column: "workspace_id");
        migrationBuilder.CreateIndex(name: "IX_source_records_workspace_id", table: "source_records", column: "workspace_id");
        migrationBuilder.CreateIndex(name: "IX_transaction_source_links_source_record_id", table: "transaction_source_links", column: "source_record_id");
    }

    protected override void Down(MigrationBuilder migrationBuilder)
    {
        migrationBuilder.DropTable(name: "ledger_entries");
        migrationBuilder.DropTable(name: "transaction_source_links");
        migrationBuilder.DropTable(name: "accounts");
        migrationBuilder.DropTable(name: "ledger_transactions");
        migrationBuilder.DropTable(name: "source_records");
        migrationBuilder.DropTable(name: "workspaces");
    }
}
