using System;
using Lukas.Infrastructure.Persistence.Models;
using Microsoft.EntityFrameworkCore;
using Microsoft.EntityFrameworkCore.Infrastructure;

#nullable disable

namespace Lukas.Infrastructure.Persistence.Migrations;

[DbContext(typeof(LukasDbContext))]
public sealed class LukasDbContextModelSnapshot : ModelSnapshot
{
    protected override void BuildModel(ModelBuilder modelBuilder)
    {
        ConfigureModel(modelBuilder);
    }

    internal static void ConfigureModel(ModelBuilder modelBuilder)
    {
        modelBuilder.HasAnnotation("ProductVersion", "10.0.7");

        modelBuilder.Entity<WorkspaceEntity>(entity =>
        {
            entity.ToTable("workspaces");
            entity.HasKey(workspace => workspace.Id);
            entity.Property(workspace => workspace.Id).HasColumnName("id");
            entity.Property(workspace => workspace.DisplayName).HasColumnName("display_name").HasMaxLength(120).IsRequired();
            entity.Property(workspace => workspace.BaseCurrency).HasColumnName("base_currency").HasMaxLength(3).IsRequired();
            entity.Property(workspace => workspace.TimeZone).HasColumnName("time_zone").HasMaxLength(80).IsRequired();
            entity.Property(workspace => workspace.CreatedAt).HasColumnName("created_at").IsRequired();
        });

        modelBuilder.Entity<AccountEntity>(entity =>
        {
            entity.ToTable("accounts");
            entity.HasKey(account => account.Id);
            entity.Property(account => account.Id).HasColumnName("id");
            entity.Property(account => account.WorkspaceId).HasColumnName("workspace_id");
            entity.Property(account => account.DisplayName).HasColumnName("display_name").HasMaxLength(120).IsRequired();
            entity.Property(account => account.Type).HasColumnName("type").HasMaxLength(40).IsRequired();
            entity.Property(account => account.Currency).HasColumnName("currency").HasMaxLength(3).IsRequired();
            entity.Property(account => account.CreatedAt).HasColumnName("created_at").IsRequired();
            entity.HasOne(account => account.Workspace).WithMany().HasForeignKey(account => account.WorkspaceId);
        });

        modelBuilder.Entity<LedgerTransactionEntity>(entity =>
        {
            entity.ToTable("ledger_transactions");
            entity.HasKey(transaction => transaction.Id);
            entity.Property(transaction => transaction.Id).HasColumnName("id");
            entity.Property(transaction => transaction.WorkspaceId).HasColumnName("workspace_id");
            entity.Property(transaction => transaction.Kind).HasColumnName("kind").HasMaxLength(40).IsRequired();
            entity.Property(transaction => transaction.TransactionDate).HasColumnName("transaction_date").IsRequired();
            entity.Property(transaction => transaction.Description).HasColumnName("description").HasMaxLength(240).IsRequired();
            entity.Property(transaction => transaction.Notes).HasColumnName("notes").HasMaxLength(1000);
            entity.Property(transaction => transaction.CreatedAt).HasColumnName("created_at").IsRequired();
            entity.HasOne(transaction => transaction.Workspace).WithMany().HasForeignKey(transaction => transaction.WorkspaceId);
        });

        modelBuilder.Entity<LedgerEntryEntity>(entity =>
        {
            entity.ToTable("ledger_entries");
            entity.HasKey(entry => entry.Id);
            entity.Property(entry => entry.Id).HasColumnName("id");
            entity.Property(entry => entry.WorkspaceId).HasColumnName("workspace_id");
            entity.Property(entry => entry.LedgerTransactionId).HasColumnName("ledger_transaction_id");
            entity.Property(entry => entry.AccountId).HasColumnName("account_id");
            entity.Property(entry => entry.AmountValue).HasColumnName("amount_value").HasMaxLength(80).IsRequired();
            entity.Property(entry => entry.Currency).HasColumnName("currency").HasMaxLength(3).IsRequired();
            entity.Property(entry => entry.AccountEffect).HasColumnName("account_effect").HasMaxLength(20).IsRequired();
            entity.HasOne(entry => entry.LedgerTransaction).WithMany(transaction => transaction.Entries).HasForeignKey(entry => entry.LedgerTransactionId);
            entity.HasOne(entry => entry.Account).WithMany().HasForeignKey(entry => entry.AccountId);
        });

        modelBuilder.Entity<SourceRecordEntity>(entity =>
        {
            entity.ToTable("source_records");
            entity.HasKey(sourceRecord => sourceRecord.Id);
            entity.Property(sourceRecord => sourceRecord.Id).HasColumnName("id");
            entity.Property(sourceRecord => sourceRecord.WorkspaceId).HasColumnName("workspace_id");
            entity.Property(sourceRecord => sourceRecord.SourceType).HasColumnName("source_type").HasMaxLength(40).IsRequired();
            entity.Property(sourceRecord => sourceRecord.RawSnapshot).HasColumnName("raw_snapshot").IsRequired();
            entity.Property(sourceRecord => sourceRecord.NormalizedSnapshot).HasColumnName("normalized_snapshot").IsRequired();
            entity.Property(sourceRecord => sourceRecord.CreatedAt).HasColumnName("created_at").IsRequired();
            entity.HasOne(sourceRecord => sourceRecord.Workspace).WithMany().HasForeignKey(sourceRecord => sourceRecord.WorkspaceId);
        });

        modelBuilder.Entity<TransactionSourceLinkEntity>(entity =>
        {
            entity.ToTable("transaction_source_links");
            entity.HasKey(link => new { link.LedgerTransactionId, link.SourceRecordId });
            entity.Property(link => link.LedgerTransactionId).HasColumnName("ledger_transaction_id");
            entity.Property(link => link.SourceRecordId).HasColumnName("source_record_id");
            entity.HasOne(link => link.LedgerTransaction).WithMany().HasForeignKey(link => link.LedgerTransactionId);
            entity.HasOne(link => link.SourceRecord).WithMany().HasForeignKey(link => link.SourceRecordId);
        });
    }
}
