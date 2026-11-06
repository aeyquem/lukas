//! Explicit PostgreSQL row mappings and workspace-scoped read queries.

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde_json::Value;
use sqlx::{FromRow, PgPool};
use thiserror::Error;
use uuid::Uuid;

#[derive(Clone, Debug, FromRow, PartialEq, Eq)]
pub struct AccountRow {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub account_type: String,
    pub base_currency: String,
    pub budget_eligible: bool,
    pub revision: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, FromRow, PartialEq, Eq)]
pub struct WalletRow {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub account_id: Uuid,
    pub currency: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, FromRow, PartialEq, Eq)]
pub struct OpeningRow {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub wallet_id: Uuid,
    pub amount: Decimal,
    pub business_date: NaiveDate,
    pub revision: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, FromRow, PartialEq, Eq)]
pub struct LiabilityBasisRow {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub opening_id: Uuid,
    pub clp_amount: Decimal,
    pub provenance: String,
    pub revision: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, FromRow, PartialEq, Eq)]
pub struct CategoryRow {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub slug: String,
    pub name: String,
    pub sort_order: i16,
    pub archived: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, FromRow, PartialEq, Eq)]
pub struct TransactionRow {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub transaction_kind: String,
    pub business_date: NaiveDate,
    pub category_id: Option<Uuid>,
    pub source_expense_id: Option<Uuid>,
    pub payee: Option<String>,
    pub note: Option<String>,
    pub confirmed_clp_amount: Option<Decimal>,
    pub conversion_evidence: Option<Value>,
    pub revision: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, FromRow, PartialEq, Eq)]
pub struct WalletEffectRow {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub transaction_id: Uuid,
    pub wallet_id: Uuid,
    pub effect_role: String,
    pub amount: Decimal,
    pub opening_treatment: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, FromRow, PartialEq, Eq)]
pub struct IdempotencyReceiptRow {
    pub workspace_id: Uuid,
    pub idempotency_key: Uuid,
    pub http_method: String,
    pub route: String,
    pub request_hash: Vec<u8>,
    pub response_status: i16,
    pub resource_id: Option<Uuid>,
    pub resource_revision: Option<i64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, FromRow, PartialEq, Eq)]
pub struct AssignmentRow {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub period_start: NaiveDate,
    pub category_id: Uuid,
    pub amount: Decimal,
    pub revision: i64,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, FromRow, PartialEq, Eq)]
pub struct FxObservationRow {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub source: String,
    pub base_currency: String,
    pub quote_currency: String,
    pub effective_date: NaiveDate,
    pub rate: Decimal,
    pub observed_at: DateTime<Utc>,
}

#[derive(Clone, Debug, FromRow, PartialEq, Eq)]
pub struct CorrectionRow {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub record_kind: String,
    pub account_id: Option<Uuid>,
    pub opening_id: Option<Uuid>,
    pub liability_basis_id: Option<Uuid>,
    pub transaction_id: Option<Uuid>,
    pub assignment_id: Option<Uuid>,
    pub prior_revision: i64,
    pub new_revision: i64,
    pub reason: String,
    pub correction_period: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, FromRow, PartialEq, Eq)]
pub struct FundingEffectRow {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub period_start: NaiveDate,
    pub source_kind: String,
    pub opening_id: Option<Uuid>,
    pub transaction_id: Option<Uuid>,
    pub correction_id: Option<Uuid>,
    pub wallet_id: Option<Uuid>,
    pub amount: Decimal,
    pub currency: String,
    pub rate_observation_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, FromRow, PartialEq, Eq)]
pub struct CloseSnapshotRow {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub period_start: NaiveDate,
    pub revision: i64,
    pub funding: Decimal,
    pub ready_to_assign: Decimal,
    pub closed_at: DateTime<Utc>,
}

#[derive(Clone, Debug, FromRow, PartialEq, Eq)]
pub struct CloseCategorySnapshotRow {
    pub workspace_id: Uuid,
    pub snapshot_id: Uuid,
    pub category_id: Uuid,
    pub assigned: Decimal,
    pub activity: Decimal,
    pub remaining: Decimal,
    pub disposition: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransactionPage {
    pub items: Vec<TransactionRow>,
    pub has_more: bool,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PageError {
    #[error("page size must be between 1 and 100")]
    InvalidSize,
}

pub async fn fetch_accounts(
    pool: &PgPool,
    workspace_id: Uuid,
) -> Result<Vec<AccountRow>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, workspace_id, name, account_type, base_currency, budget_eligible, \
         revision, created_at, updated_at FROM financial_accounts \
         WHERE workspace_id = $1 ORDER BY name, id",
    )
    .bind(workspace_id)
    .fetch_all(pool)
    .await
}

pub async fn fetch_wallets(
    pool: &PgPool,
    workspace_id: Uuid,
    account_id: Uuid,
) -> Result<Vec<WalletRow>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, workspace_id, account_id, currency, created_at FROM financial_wallets \
         WHERE workspace_id = $1 AND account_id = $2 ORDER BY currency, id",
    )
    .bind(workspace_id)
    .bind(account_id)
    .fetch_all(pool)
    .await
}

pub async fn fetch_openings(
    pool: &PgPool,
    workspace_id: Uuid,
    wallet_id: Uuid,
) -> Result<Vec<OpeningRow>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, workspace_id, wallet_id, amount, business_date, revision, created_at \
         FROM financial_openings WHERE workspace_id = $1 AND wallet_id = $2 \
         ORDER BY business_date, id",
    )
    .bind(workspace_id)
    .bind(wallet_id)
    .fetch_all(pool)
    .await
}

pub async fn fetch_liability_basis(
    pool: &PgPool,
    workspace_id: Uuid,
    opening_id: Uuid,
) -> Result<Option<LiabilityBasisRow>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, workspace_id, opening_id, clp_amount, provenance, revision, created_at \
         FROM foreign_liability_bases WHERE workspace_id = $1 AND opening_id = $2",
    )
    .bind(workspace_id)
    .bind(opening_id)
    .fetch_optional(pool)
    .await
}

pub async fn fetch_categories(
    pool: &PgPool,
    workspace_id: Uuid,
) -> Result<Vec<CategoryRow>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, workspace_id, slug, name, sort_order, archived, created_at, updated_at \
         FROM expense_categories WHERE workspace_id = $1 ORDER BY sort_order, id",
    )
    .bind(workspace_id)
    .fetch_all(pool)
    .await
}

pub async fn fetch_transactions_page(
    pool: &PgPool,
    workspace_id: Uuid,
    page_size: i64,
    before: Option<(NaiveDate, Uuid)>,
) -> Result<TransactionPage, PageOrSqlxError> {
    if !(1..=100).contains(&page_size) {
        return Err(PageError::InvalidSize.into());
    }

    let rows: Vec<TransactionRow> = sqlx::query_as(
        "SELECT id, workspace_id, transaction_kind, business_date, category_id, \
         source_expense_id, payee, note, confirmed_clp_amount, conversion_evidence, \
         revision, created_at, updated_at FROM financial_transactions \
         WHERE workspace_id = $1 AND ($2::DATE IS NULL OR (business_date, id) < ($2, $3)) \
         ORDER BY business_date DESC, id DESC LIMIT $4",
    )
    .bind(workspace_id)
    .bind(before.map(|cursor| cursor.0))
    .bind(before.map(|cursor| cursor.1))
    .bind(page_size + 1)
    .fetch_all(pool)
    .await?;

    let has_more = rows.len() > page_size as usize;
    let items = rows.into_iter().take(page_size as usize).collect();
    Ok(TransactionPage { items, has_more })
}

pub async fn fetch_wallet_effects(
    pool: &PgPool,
    workspace_id: Uuid,
    transaction_id: Uuid,
) -> Result<Vec<WalletEffectRow>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, workspace_id, transaction_id, wallet_id, effect_role, amount, \
         opening_treatment, created_at FROM financial_wallet_effects \
         WHERE workspace_id = $1 AND transaction_id = $2 ORDER BY effect_role, id",
    )
    .bind(workspace_id)
    .bind(transaction_id)
    .fetch_all(pool)
    .await
}

pub async fn fetch_assignments(
    pool: &PgPool,
    workspace_id: Uuid,
    period_start: NaiveDate,
) -> Result<Vec<AssignmentRow>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, workspace_id, period_start, category_id, amount, revision, updated_at \
         FROM budget_assignments WHERE workspace_id = $1 AND period_start = $2 \
         ORDER BY category_id, id",
    )
    .bind(workspace_id)
    .bind(period_start)
    .fetch_all(pool)
    .await
}

pub async fn fetch_funding_effects(
    pool: &PgPool,
    workspace_id: Uuid,
    period_start: NaiveDate,
) -> Result<Vec<FundingEffectRow>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, workspace_id, period_start, source_kind, opening_id, transaction_id, \
         correction_id, wallet_id, amount, currency, rate_observation_id, created_at \
         FROM budget_funding_effects WHERE workspace_id = $1 AND period_start = $2 \
         ORDER BY created_at, id",
    )
    .bind(workspace_id)
    .bind(period_start)
    .fetch_all(pool)
    .await
}

pub async fn fetch_fx_observations(
    pool: &PgPool,
    workspace_id: Uuid,
    base_currency: &str,
    quote_currency: &str,
) -> Result<Vec<FxObservationRow>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, workspace_id, source, base_currency, quote_currency, effective_date, \
         rate, observed_at FROM daily_fx_observations \
         WHERE workspace_id = $1 AND base_currency = $2 AND quote_currency = $3 \
         ORDER BY effective_date DESC, id DESC",
    )
    .bind(workspace_id)
    .bind(base_currency)
    .bind(quote_currency)
    .fetch_all(pool)
    .await
}

pub async fn find_idempotency_receipt(
    pool: &PgPool,
    workspace_id: Uuid,
    idempotency_key: Uuid,
) -> Result<Option<IdempotencyReceiptRow>, sqlx::Error> {
    sqlx::query_as(
        "SELECT workspace_id, idempotency_key, http_method, route, request_hash, \
         response_status, resource_id, resource_revision, created_at \
         FROM financial_idempotency_receipts WHERE workspace_id = $1 AND idempotency_key = $2",
    )
    .bind(workspace_id)
    .bind(idempotency_key)
    .fetch_optional(pool)
    .await
}

pub async fn fetch_corrections(
    pool: &PgPool,
    workspace_id: Uuid,
    record_kind: &str,
    record_id: Uuid,
) -> Result<Vec<CorrectionRow>, sqlx::Error> {
    let query = match record_kind {
        "account" => {
            "SELECT id, workspace_id, record_kind, account_id, opening_id, liability_basis_id, \
             transaction_id, assignment_id, prior_revision, new_revision, reason, correction_period, created_at \
             FROM financial_corrections WHERE workspace_id = $1 AND record_kind = $2 AND account_id = $3 \
             ORDER BY created_at, id"
        }
        "opening" => {
            "SELECT id, workspace_id, record_kind, account_id, opening_id, liability_basis_id, \
             transaction_id, assignment_id, prior_revision, new_revision, reason, correction_period, created_at \
             FROM financial_corrections WHERE workspace_id = $1 AND record_kind = $2 AND opening_id = $3 \
             ORDER BY created_at, id"
        }
        "liability_basis" => {
            "SELECT id, workspace_id, record_kind, account_id, opening_id, liability_basis_id, \
             transaction_id, assignment_id, prior_revision, new_revision, reason, correction_period, created_at \
             FROM financial_corrections WHERE workspace_id = $1 AND record_kind = $2 AND liability_basis_id = $3 \
             ORDER BY created_at, id"
        }
        "transaction" => {
            "SELECT id, workspace_id, record_kind, account_id, opening_id, liability_basis_id, \
             transaction_id, assignment_id, prior_revision, new_revision, reason, correction_period, created_at \
             FROM financial_corrections WHERE workspace_id = $1 AND record_kind = $2 AND transaction_id = $3 \
             ORDER BY created_at, id"
        }
        "assignment" => {
            "SELECT id, workspace_id, record_kind, account_id, opening_id, liability_basis_id, \
             transaction_id, assignment_id, prior_revision, new_revision, reason, correction_period, created_at \
             FROM financial_corrections WHERE workspace_id = $1 AND record_kind = $2 AND assignment_id = $3 \
             ORDER BY created_at, id"
        }
        _ => return Ok(Vec::new()),
    };

    sqlx::query_as(query)
        .bind(workspace_id)
        .bind(record_kind)
        .bind(record_id)
        .fetch_all(pool)
        .await
}

pub async fn fetch_close_snapshot(
    pool: &PgPool,
    workspace_id: Uuid,
    period_start: NaiveDate,
) -> Result<Option<CloseSnapshotRow>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, workspace_id, period_start, revision, funding, ready_to_assign, closed_at \
         FROM budget_close_snapshots WHERE workspace_id = $1 AND period_start = $2",
    )
    .bind(workspace_id)
    .bind(period_start)
    .fetch_optional(pool)
    .await
}

pub async fn fetch_close_category_snapshots(
    pool: &PgPool,
    workspace_id: Uuid,
    snapshot_id: Uuid,
) -> Result<Vec<CloseCategorySnapshotRow>, sqlx::Error> {
    sqlx::query_as(
        "SELECT workspace_id, snapshot_id, category_id, assigned, activity, remaining, disposition \
         FROM budget_close_category_snapshots WHERE workspace_id = $1 AND snapshot_id = $2 \
         ORDER BY category_id",
    )
    .bind(workspace_id)
    .bind(snapshot_id)
    .fetch_all(pool)
    .await
}

#[derive(Debug, Error)]
pub enum PageOrSqlxError {
    #[error(transparent)]
    InvalidSize(#[from] PageError),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}
