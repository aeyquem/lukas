using Microsoft.EntityFrameworkCore;
using Lukas.Infrastructure.Persistence.Models;

namespace Lukas.Infrastructure.Persistence;

public sealed class LukasDbContext(DbContextOptions<LukasDbContext> options) : DbContext(options)
{
    public DbSet<WorkspaceEntity> Workspaces => Set<WorkspaceEntity>();

    public DbSet<AccountEntity> Accounts => Set<AccountEntity>();

    public DbSet<LedgerTransactionEntity> LedgerTransactions => Set<LedgerTransactionEntity>();

    public DbSet<LedgerEntryEntity> LedgerEntries => Set<LedgerEntryEntity>();

    public DbSet<BudgetCategoryEntity> BudgetCategories => Set<BudgetCategoryEntity>();

    public DbSet<BudgetSubcategoryEntity> BudgetSubcategories => Set<BudgetSubcategoryEntity>();

    public DbSet<BudgetEnvelopeEntity> BudgetEnvelopes => Set<BudgetEnvelopeEntity>();

    public DbSet<SourceRecordEntity> SourceRecords => Set<SourceRecordEntity>();

    public DbSet<TransactionSourceLinkEntity> TransactionSourceLinks => Set<TransactionSourceLinkEntity>();

    protected override void OnModelCreating(ModelBuilder modelBuilder)
    {
        base.OnModelCreating(modelBuilder);
        modelBuilder.HasDefaultSchema(null);

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
            entity.Property(transaction => transaction.EnvelopeId).HasColumnName("envelope_id");
            entity.Property(transaction => transaction.CategoryName).HasColumnName("category_name").HasMaxLength(120);
            entity.Property(transaction => transaction.Notes).HasColumnName("notes").HasMaxLength(1000);
            entity.Property(transaction => transaction.CreatedAt).HasColumnName("created_at").IsRequired();
            entity.HasOne(transaction => transaction.Workspace).WithMany().HasForeignKey(transaction => transaction.WorkspaceId);
            entity.HasOne(transaction => transaction.Envelope).WithMany().HasForeignKey(transaction => transaction.EnvelopeId);
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

        modelBuilder.Entity<BudgetCategoryEntity>(entity =>
        {
            entity.ToTable("budget_categories");
            entity.HasKey(category => category.Id);
            entity.Property(category => category.Id).HasColumnName("id");
            entity.Property(category => category.WorkspaceId).HasColumnName("workspace_id");
            entity.Property(category => category.Name).HasColumnName("name").HasMaxLength(120).IsRequired();
            entity.Property(category => category.SortOrder).HasColumnName("sort_order").IsRequired();
            entity.HasOne(category => category.Workspace).WithMany().HasForeignKey(category => category.WorkspaceId);
            entity.HasIndex(category => new { category.WorkspaceId, category.SortOrder });
        });

        modelBuilder.Entity<BudgetSubcategoryEntity>(entity =>
        {
            entity.ToTable("budget_subcategories");
            entity.HasKey(subcategory => subcategory.Id);
            entity.Property(subcategory => subcategory.Id).HasColumnName("id");
            entity.Property(subcategory => subcategory.WorkspaceId).HasColumnName("workspace_id");
            entity.Property(subcategory => subcategory.CategoryId).HasColumnName("category_id");
            entity.Property(subcategory => subcategory.Name).HasColumnName("name").HasMaxLength(120).IsRequired();
            entity.Property(subcategory => subcategory.SortOrder).HasColumnName("sort_order").IsRequired();
            entity.HasOne(subcategory => subcategory.Workspace).WithMany().HasForeignKey(subcategory => subcategory.WorkspaceId);
            entity.HasOne(subcategory => subcategory.Category).WithMany(category => category.Subcategories).HasForeignKey(subcategory => subcategory.CategoryId);
            entity.HasIndex(subcategory => new { subcategory.WorkspaceId, subcategory.CategoryId, subcategory.SortOrder });
        });

        modelBuilder.Entity<BudgetEnvelopeEntity>(entity =>
        {
            entity.ToTable("budget_envelopes");
            entity.HasKey(envelope => envelope.Id);
            entity.Property(envelope => envelope.Id).HasColumnName("id");
            entity.Property(envelope => envelope.WorkspaceId).HasColumnName("workspace_id");
            entity.Property(envelope => envelope.SubcategoryId).HasColumnName("subcategory_id");
            entity.Property(envelope => envelope.Name).HasColumnName("name").HasMaxLength(120).IsRequired();
            entity.Property(envelope => envelope.AssignedAmount).HasColumnName("assigned_amount").HasMaxLength(80).IsRequired();
            entity.Property(envelope => envelope.SortOrder).HasColumnName("sort_order").IsRequired();
            entity.HasOne(envelope => envelope.Workspace).WithMany().HasForeignKey(envelope => envelope.WorkspaceId);
            entity.HasOne(envelope => envelope.Subcategory).WithMany(subcategory => subcategory.Envelopes).HasForeignKey(envelope => envelope.SubcategoryId);
            entity.HasIndex(envelope => new { envelope.WorkspaceId, envelope.SubcategoryId, envelope.SortOrder });
            entity.HasIndex(envelope => new { envelope.WorkspaceId, envelope.Name });
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
