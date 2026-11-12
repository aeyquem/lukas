//! Workspace-scoped category and monthly budget HTTP use cases.

use std::collections::{BTreeMap, HashMap, HashSet};

use axum::{
    Json, Router,
    body::Bytes,
    extract::{Path, State, rejection::PathRejection},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::get,
};
use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use sqlx::{FromRow, PgPool};
use tokio::task::JoinSet;
use uuid::Uuid;

use crate::{
    budget_calculations::{
        ActivityKind, BudgetPeriod, CalculatedValue, FundingEvent, FundingKind, MissingValue,
        PeriodInput, PeriodResult, TransactionActivity, UnresolvedActivity, ValueState,
        attribute_transactions, calculate_period, foreign_asset_opening_value,
        foreign_liability_opening_value, transfer_funding_effect,
    },
    financial_values::MoneyAmount,
    fx::{
        FRANKFURTER_SOURCE, FrankfurterProvider, FxError, FxObservation, get_observation,
        value_foreign_asset,
    },
    health::AppState,
    http_error::{ApiError, retry_transaction},
    write_guard::{installation_generation, lock_workspace},
};

macro_rules! category_references { () => { "EXISTS(SELECT 1 FROM financial_transactions t WHERE t.workspace_id=c.workspace_id AND t.category_id=c.id) OR EXISTS(SELECT 1 FROM budget_assignments a WHERE a.workspace_id=c.workspace_id AND a.category_id=c.id) OR EXISTS(SELECT 1 FROM budget_close_category_snapshots s WHERE s.workspace_id=c.workspace_id AND s.category_id=c.id) OR EXISTS(SELECT 1 FROM financial_corrections f WHERE f.workspace_id=c.workspace_id AND f.transaction_target_category_id=c.id) OR EXISTS(SELECT 1 FROM budget_activity_effects e WHERE e.workspace_id=c.workspace_id AND e.category_id=c.id) OR EXISTS(SELECT 1 FROM budget_close_transaction_snapshots t WHERE t.workspace_id=c.workspace_id AND t.category_id=c.id)" }; }

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/categories",
            get(list_categories).post(create_category),
        )
        .route(
            "/api/categories/{category_id}",
            axum::routing::put(edit_category).delete(delete_category),
        )
        .route("/api/budget-groups", get(list_groups).post(create_group))
        .route(
            "/api/budget-groups/{group_id}",
            axum::routing::put(edit_group),
        )
        .route("/api/budgets/{period}", get(read_budget))
        .route(
            "/api/budgets/{period}/close-preview",
            get(read_close_preview),
        )
        .route(
            "/api/budgets/{period}/close",
            axum::routing::post(close_budget),
        )
        .route(
            "/api/budgets/{period}/assignments/{category_id}",
            axum::routing::put(save_assignment),
        )
}

#[derive(Clone, Debug, FromRow, Serialize)]
pub(crate) struct CategoryView {
    id: Uuid,
    slug: String,
    name: String,
    sort_order: i16,
    archived: bool,
    group_id: Uuid,
    revision: i64,
    #[sqlx(default)]
    can_delete: bool,
}

#[derive(Clone, Debug, FromRow, Serialize)]
struct BudgetGroup {
    id: Uuid,
    name: String,
    sort_order: i16,
    archived: bool,
    revision: i64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GroupRequest {
    name: String,
    sort_order: i16,
    #[serde(default)]
    archived: bool,
    expected_revision: Option<i64>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DeleteCategoryRequest {
    expected_revision: Option<i64>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateCategoryRequest {
    name: String,
    sort_order: i16,
    group_id: Option<Uuid>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EditCategoryRequest {
    name: String,
    sort_order: i16,
    archived: bool,
    group_id: Option<Uuid>,
    expected_revision: Option<i64>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AssignmentRequest {
    amount: MoneyAmount,
    expected_period_revision: String,
    expected_revision: Option<i64>,
    #[serde(default)]
    reason: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CloseRequest {
    expected_period_revision: String,
    categories: Vec<CloseCategoryRequest>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CloseCategoryRequest {
    category_id: Uuid,
    disposition: String,
}

#[derive(Serialize)]
struct MutationReceipt {
    resource_id: Uuid,
    revision: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    period_revision: Option<String>,
}

#[derive(Serialize)]
struct BudgetResponse {
    period: String,
    revision: String,
    closed: bool,
    funding: BudgetValue,
    ready_to_assign: BudgetValue,
    activity: BudgetValue,
    uncategorized_activity: BudgetValue,
    complete: bool,
    categories: Vec<BudgetCategory>,
    archived_categories: Vec<BudgetCategory>,
    groups: Vec<BudgetGroup>,
    available_to_budget_breakdown: AvailableToBudgetBreakdown,
}

#[derive(Serialize)]
struct ClosePreviewResponse {
    #[serde(flatten)]
    budget: BudgetResponse,
    close: ClosePreview,
}

#[derive(Serialize)]
struct ClosePreview {
    eligible: bool,
    reason: Option<String>,
    expected_period_revision: String,
    categories: Vec<ClosePreviewCategory>,
}

#[derive(Serialize)]
struct ClosePreviewCategory {
    category_id: Uuid,
    proposed_disposition: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
enum BudgetValue {
    Complete {
        amount: MoneyAmount,
    },
    Unavailable {
        reasons: Vec<String>,
    },
    Incomplete {
        known_subtotal: MoneyAmount,
        unresolved: Vec<UnresolvedValue>,
    },
}

#[derive(Clone, Serialize, Deserialize)]
struct UnresolvedValue {
    kind: String,
    category_id: Option<String>,
    currency: Option<String>,
    amount: Option<MoneyAmount>,
}

#[derive(Clone, Serialize, Deserialize)]
struct AvailableToBudgetBreakdown {
    prior_unassigned: BudgetValue,
    funding_total: BudgetValue,
    income: BudgetValue,
    openings: BudgetValue,
    transfers: BudgetValue,
    valuation: BudgetValue,
    corrections: BudgetValue,
    category_releases: BudgetValue,
    category_rollover: BudgetValue,
    assignments: BudgetValue,
}

#[derive(Serialize)]
struct BudgetCategory {
    id: Uuid,
    group_id: Uuid,
    archived: bool,
    name: String,
    sort_order: i16,
    assigned: MoneyAmount,
    #[serde(skip_serializing_if = "Option::is_none")]
    assignment_intent: Option<MoneyAmount>,
    assignment_revision: Option<i64>,
    activity: BudgetValue,
    remaining: BudgetValue,
}

#[derive(FromRow)]
struct AssignmentRow {
    id: Uuid,
    period_start: NaiveDate,
    category_id: Uuid,
    amount: Decimal,
    revision: i64,
}

#[derive(FromRow)]
struct OpeningRow {
    business_date: NaiveDate,
    amount: Decimal,
    currency: String,
    budget_eligible: bool,
    liability_clp_basis: Option<Decimal>,
}

#[derive(FromRow)]
struct TransactionEffectRow {
    id: Uuid,
    wallet_id: Uuid,
    transaction_kind: String,
    business_date: NaiveDate,
    category_id: Option<Uuid>,
    confirmed_clp_amount: Option<Decimal>,
    conversion_evidence: Option<Value>,
    currency: String,
    effect_role: String,
    effect_amount: Decimal,
    opening_treatment: Option<String>,
    budget_eligible: bool,
}

#[derive(FromRow)]
struct FundingEffectRow {
    period_start: NaiveDate,
    source_kind: String,
    amount: Option<Decimal>,
    missing_reason: Option<String>,
}

#[derive(FromRow)]
#[allow(dead_code)]
struct ActivityEffectRow {
    period_start: NaiveDate,
    transaction_id: Uuid,
    category_id: Option<Uuid>,
    baseline_amount: Option<Decimal>,
    target_amount: Option<Decimal>,
    baseline_kind: String,
    baseline_currency: String,
    baseline_original_amount: Decimal,
    baseline_rate_observation_id: Option<Uuid>,
    baseline_business_date: NaiveDate,
    target_kind: String,
    target_currency: String,
    target_original_amount: Decimal,
    target_rate_observation_id: Option<Uuid>,
    target_business_date: NaiveDate,
}

#[derive(FromRow)]
struct FundingCorrectionStateRow {
    baseline_amount: Option<Decimal>,
    target_amount: Option<Decimal>,
    baseline_rate_observation_id: Option<Uuid>,
    baseline_additional_rate_observation_id: Option<Uuid>,
    target_rate_observation_id: Option<Uuid>,
    target_additional_rate_observation_id: Option<Uuid>,
}

#[derive(FromRow)]
struct OpeningCloseBaselineRow {
    funding_amount: Decimal,
    rate_observation_id: Option<Uuid>,
}

#[derive(FromRow)]
struct OpeningCloseSourceRow {
    opening_id: Uuid,
    business_date: NaiveDate,
    amount: Decimal,
    currency: String,
    budget_eligible: bool,
    liability_clp_basis: Option<Decimal>,
}

#[derive(FromRow)]
struct TransactionCloseBaselineRow {
    funding_amount: Option<Decimal>,
    funding_rate_observation_id: Option<Uuid>,
    funding_additional_rate_observation_id: Option<Uuid>,
    activity_amount: Option<Decimal>,
    activity_kind: Option<String>,
    category_id: Option<Uuid>,
    currency: String,
    original_amount: Decimal,
    activity_rate_observation_id: Option<Uuid>,
}

#[derive(FromRow)]
struct CarryingFlowRow {
    business_date: NaiveDate,
    amount: Decimal,
    record_kind: String,
    transaction_kind: Option<String>,
    effect_role: Option<String>,
    confirmed_clp_amount: Option<Decimal>,
    conversion_evidence: Option<Value>,
    liability_clp_basis: Option<Decimal>,
}

#[derive(Clone, FromRow)]
struct CloseSnapshotRow {
    id: Uuid,
    period_start: NaiveDate,
    funding: Decimal,
    ready_to_assign: Decimal,
    activity: Decimal,
    uncategorized_activity: Decimal,
    closed_at: chrono::DateTime<chrono::Utc>,
    available_to_budget_breakdown: Option<Value>,
}

#[derive(Clone, FromRow)]
struct CloseCategorySnapshotRow {
    snapshot_id: Uuid,
    category_id: Uuid,
    category_name: Option<String>,
    category_sort_order: Option<i16>,
    assigned: Decimal,
    activity: Decimal,
    remaining: Decimal,
    disposition: String,
}

#[derive(FromRow)]
struct AssignmentTargetRow {
    assignment_id: Uuid,
    assignment_target_amount: Decimal,
}

#[derive(FromRow)]
struct StoredRateRow {
    id: Uuid,
    workspace_id: Uuid,
    source: String,
    base_currency: String,
    quote_currency: String,
    effective_date: NaiveDate,
    rate: Decimal,
}

#[derive(FromRow)]
struct RevaluationWalletRow {
    wallet_id: Uuid,
    currency: String,
    budget_eligible: bool,
    current_balance: Decimal,
}

#[derive(FromRow)]
struct RevaluationFlowRow {
    business_date: NaiveDate,
    amount: Decimal,
    record_kind: String,
    transaction_kind: Option<String>,
    effect_role: Option<String>,
    confirmed_clp_amount: Option<Decimal>,
    conversion_evidence: Option<Value>,
    liability_clp_basis: Option<Decimal>,
}

#[derive(Clone, FromRow)]
struct RateRequestRow {
    currency: String,
    rate_date: NaiveDate,
}

#[derive(FromRow)]
struct FundingCorrectionRefreshRow {
    correction_id: Uuid,
    opening_id: Option<Uuid>,
    transaction_id: Option<Uuid>,
    baseline_amount: Option<Decimal>,
}

#[derive(FromRow)]
struct ActivityCorrectionRefreshRow {
    category_id: Option<Uuid>,
}

#[derive(Clone)]
pub(crate) struct BudgetTransactionSide {
    pub wallet_id: Uuid,
    pub currency: String,
    pub role: String,
    pub amount: Decimal,
    pub budget_eligible: bool,
    pub opening_treatment: Option<String>,
}

#[derive(Clone)]
pub(crate) struct BudgetTransactionState {
    pub transaction_id: Uuid,
    pub kind: String,
    pub business_date: NaiveDate,
    pub category_id: Option<Uuid>,
    pub confirmed_clp_amount: Option<Decimal>,
    pub conversion_evidence: Option<Value>,
    pub effects: Vec<BudgetTransactionSide>,
}

#[derive(FromRow)]
struct CorrectionTransactionRow {
    kind: String,
    business_date: NaiveDate,
    category_id: Option<Uuid>,
    confirmed_clp_amount: Option<Decimal>,
    conversion_evidence: Option<Value>,
}

#[derive(FromRow)]
struct CorrectionTransactionSideRow {
    wallet_id: Uuid,
    currency: String,
    role: String,
    amount: Decimal,
    budget_eligible: bool,
    opening_treatment: Option<String>,
}

#[derive(FromRow)]
struct StoredReceipt {
    http_method: String,
    route: String,
    request_hash: Vec<u8>,
    response_status: i16,
    resource_id: Option<Uuid>,
    resource_revision: Option<i64>,
}

pub(crate) async fn list_categories(
    State(state): State<AppState>,
) -> Result<Json<Vec<CategoryView>>, ApiError> {
    let sql = concat!(
        "SELECT c.id,c.slug,c.name,c.sort_order,c.archived,c.group_id,c.revision, NOT (",
        category_references!(),
        ") AS can_delete FROM expense_categories c WHERE c.workspace_id=$1 ORDER BY c.sort_order,c.id"
    );
    let rows = sqlx::query_as::<_, CategoryView>(sql)
        .bind(state.workspace_id)
        .fetch_all(&state.pool)
        .await
        .map_err(ApiError::database)?;
    Ok(Json(rows))
}

async fn create_category(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<impl IntoResponse, ApiError> {
    let key = idempotency_key(&headers)?;
    let generation = installation_generation(&headers)?;
    retry_transaction(|| create_category_attempt(state.clone(), generation, key, body.clone()))
        .await
}

async fn create_category_attempt(
    state: AppState,
    generation: Uuid,
    key: Uuid,
    body: Bytes,
) -> Result<impl IntoResponse, ApiError> {
    let route = "/api/categories";
    let hash = Sha256::digest(&body).to_vec();
    let mut tx = state.pool.begin().await.map_err(ApiError::database)?;
    lock_workspace(&mut tx, state.workspace_id, generation).await?;
    if let Some(receipt) =
        find_receipt(&mut tx, state.workspace_id, key, "POST", route, &hash).await?
    {
        tx.commit().await.map_err(ApiError::database)?;
        return Ok((
            StatusCode::from_u16(receipt.response_status as u16).unwrap_or(StatusCode::CREATED),
            Json(MutationReceipt {
                resource_id: receipt.resource_id.ok_or_else(ApiError::not_found)?,
                revision: receipt.resource_revision.unwrap_or(1),
                period_revision: None,
            }),
        ));
    }
    let request: CreateCategoryRequest = parse_body(&body)?;
    validate_category(&request.name, request.sort_order)?;
    let group_id = active_group_in(&mut tx, state.workspace_id, request.group_id).await?;
    let id = Uuid::new_v4();
    let slug = format!("custom-{}", id.simple());
    let row = sqlx::query_as::<_, CategoryView>(
        "INSERT INTO expense_categories (id, workspace_id, slug, name, sort_order, group_id) \
         VALUES ($1, $2, $3, $4, $5, $6) \
         RETURNING id, slug, name, sort_order, archived, group_id, revision",
    )
    .bind(id)
    .bind(state.workspace_id)
    .bind(slug)
    .bind(request.name.trim())
    .bind(request.sort_order)
    .bind(group_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(ApiError::database)?;
    save_receipt(
        &mut tx,
        state.workspace_id,
        key,
        "POST",
        route,
        &hash,
        StatusCode::CREATED,
        row.id,
        1,
    )
    .await?;
    tx.commit().await.map_err(ApiError::database)?;
    Ok((
        StatusCode::CREATED,
        Json(MutationReceipt {
            resource_id: row.id,
            revision: 1,
            period_revision: None,
        }),
    ))
}

async fn edit_category(
    State(state): State<AppState>,
    path: Result<Path<Uuid>, PathRejection>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<impl IntoResponse, ApiError> {
    let Path(category_id) = path.map_err(|_| ApiError::invalid_request_parameter())?;
    let key = idempotency_key(&headers)?;
    let generation = installation_generation(&headers)?;
    retry_transaction(|| {
        edit_category_attempt(state.clone(), generation, category_id, key, body.clone())
    })
    .await
}

async fn edit_category_attempt(
    state: AppState,
    generation: Uuid,
    category_id: Uuid,
    key: Uuid,
    body: Bytes,
) -> Result<impl IntoResponse, ApiError> {
    let route = format!("/api/categories/{category_id}");
    let hash = Sha256::digest(&body).to_vec();
    let mut tx = state.pool.begin().await.map_err(ApiError::database)?;
    lock_workspace(&mut tx, state.workspace_id, generation).await?;
    if let Some(receipt) =
        find_receipt(&mut tx, state.workspace_id, key, "PUT", &route, &hash).await?
    {
        tx.commit().await.map_err(ApiError::database)?;
        return Ok((
            StatusCode::from_u16(receipt.response_status as u16).unwrap_or(StatusCode::OK),
            Json(MutationReceipt {
                resource_id: receipt.resource_id.ok_or_else(ApiError::not_found)?,
                revision: receipt.resource_revision.unwrap_or(1),
                period_revision: None,
            }),
        ));
    }
    let request: EditCategoryRequest = parse_body(&body)?;
    validate_category(&request.name, request.sort_order)?;
    let current = sqlx::query_as::<_, CategoryView>("SELECT id,slug,name,sort_order,archived,group_id,revision FROM expense_categories WHERE workspace_id=$1 AND id=$2 FOR UPDATE")
        .bind(state.workspace_id).bind(category_id).fetch_optional(&mut *tx).await.map_err(ApiError::database)?.ok_or_else(ApiError::not_found)?;
    check_organization_revision(request.expected_revision, current.revision)?;
    let group_id = request.group_id.unwrap_or(current.group_id);
    if !request.archived {
        active_group_in(&mut tx, state.workspace_id, Some(group_id)).await?;
    } else {
        // Archived categories may retain membership in an archived group.
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM budget_category_groups WHERE workspace_id=$1 AND id=$2)",
        )
        .bind(state.workspace_id)
        .bind(group_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(ApiError::database)?;
        if !exists {
            return Err(ApiError::invalid("group_id", "not_found"));
        }
    }
    let revision = current.revision + 1;
    sqlx::query("UPDATE expense_categories SET name=$3,sort_order=$4,archived=$5,group_id=$6,revision=$7,updated_at=now() WHERE workspace_id=$1 AND id=$2")
        .bind(state.workspace_id).bind(category_id).bind(request.name.trim()).bind(request.sort_order).bind(request.archived).bind(group_id).bind(revision)
        .execute(&mut *tx).await.map_err(ApiError::database)?;
    save_receipt(
        &mut tx,
        state.workspace_id,
        key,
        "PUT",
        &route,
        &hash,
        StatusCode::OK,
        category_id,
        revision,
    )
    .await?;
    tx.commit().await.map_err(ApiError::database)?;
    Ok((
        StatusCode::OK,
        Json(MutationReceipt {
            resource_id: category_id,
            revision,
            period_revision: None,
        }),
    ))
}

async fn read_budget(
    State(state): State<AppState>,
    path: Result<Path<String>, PathRejection>,
) -> Result<Json<BudgetResponse>, ApiError> {
    let Path(period) = path.map_err(|_| ApiError::invalid_request_parameter())?;
    let target = parse_period(&period)?;
    for _ in 0..3 {
        let (prefetched_rates, revaluation_revision) =
            apply_open_period_revaluations(&state.pool, state.workspace_id, target).await?;
        let data = load_budget_data(&state.pool, state.workspace_id, target).await?;
        if revaluation_revision
            .as_ref()
            .is_some_and(|revision| revision != &data.revision)
        {
            continue;
        }
        if let Some(snapshot) = data
            .close_snapshots
            .iter()
            .find(|snapshot| snapshot.period_start == target)
        {
            let mut revision_tx = state.pool.begin().await.map_err(ApiError::database)?;
            let current_revision =
                period_revision_in(&mut revision_tx, state.workspace_id, next_month(target)?)
                    .await?;
            revision_tx.commit().await.map_err(ApiError::database)?;
            if current_revision != data.revision {
                continue;
            }
            return Ok(Json(closed_snapshot_response(
                target, period, snapshot, &data,
            )?));
        }
        let result = calculate_open_period(
            &state.pool,
            state.workspace_id,
            target,
            &data,
            prefetched_rates.clone(),
        )
        .await?;
        let mut revision_tx = state.pool.begin().await.map_err(ApiError::database)?;
        let current_revision =
            period_revision_in(&mut revision_tx, state.workspace_id, next_month(target)?).await?;
        revision_tx.commit().await.map_err(ApiError::database)?;
        if current_revision == data.revision {
            return Ok(Json(to_budget_response(target, period, &result, &data)?));
        }
    }
    Err(ApiError::new(
        StatusCode::SERVICE_UNAVAILABLE,
        "budget_changed_during_read",
        "Budget inputs changed while the read was calculated. Retry the request.",
    ))
}

fn closed_snapshot_response(
    target: NaiveDate,
    period_text: String,
    snapshot: &CloseSnapshotRow,
    data: &BudgetData,
) -> Result<BudgetResponse, ApiError> {
    let categories = data
        .close_category_snapshots
        .iter()
        .filter(|row| {
            data.close_snapshots
                .iter()
                .find(|item| item.period_start == target)
                .is_some_and(|item| item.id == row.snapshot_id)
        })
        .map(|row| {
            let assignment = data.assignments.iter().find(|assignment| {
                assignment.period_start == target && assignment.category_id == row.category_id
            });
            let category = data
                .categories
                .iter()
                .find(|category| category.id == row.category_id);
            Ok(BudgetCategory {
                id: row.category_id,
                group_id: category
                    .map(|c| c.group_id)
                    .ok_or_else(ApiError::not_found)?,
                archived: category.is_some_and(|c| c.archived),
                name: row.category_name.clone().unwrap_or_else(|| {
                    data.categories
                        .iter()
                        .find(|category| category.id == row.category_id)
                        .map(|category| category.name.clone())
                        .unwrap_or_else(|| "Archived category".to_owned())
                }),
                sort_order: row.category_sort_order.unwrap_or(i16::MAX),
                assigned: MoneyAmount::new(row.assigned).map_err(stored_money_error)?,
                assignment_intent: assignment
                    .and_then(|assignment| data.assignment_targets.get(&assignment.id))
                    .map(|amount| MoneyAmount::new(*amount).map_err(stored_money_error))
                    .transpose()?,
                assignment_revision: assignment.map(|assignment| assignment.revision),
                activity: BudgetValue::Complete {
                    amount: MoneyAmount::new(row.activity).map_err(stored_money_error)?,
                },
                remaining: BudgetValue::Complete {
                    amount: MoneyAmount::new(row.remaining).map_err(stored_money_error)?,
                },
            })
        })
        .collect::<Result<Vec<_>, ApiError>>()?;
    let _closed_at = snapshot.closed_at;
    Ok(BudgetResponse {
        period: period_text,
        revision: data.revision.clone(),
        closed: true,
        funding: BudgetValue::Complete {
            amount: MoneyAmount::new(snapshot.funding).map_err(stored_money_error)?,
        },
        ready_to_assign: BudgetValue::Complete {
            amount: MoneyAmount::new(snapshot.ready_to_assign).map_err(stored_money_error)?,
        },
        activity: BudgetValue::Complete {
            amount: MoneyAmount::new(snapshot.activity).map_err(stored_money_error)?,
        },
        uncategorized_activity: BudgetValue::Complete {
            amount: MoneyAmount::new(snapshot.uncategorized_activity)
                .map_err(stored_money_error)?,
        },
        complete: true,
        categories,
        archived_categories: Vec::new(),
        groups: data.groups.clone(),
        available_to_budget_breakdown: closed_breakdown(target, snapshot, data)?,
    })
}

async fn read_close_preview(
    State(state): State<AppState>,
    path: Result<Path<String>, PathRejection>,
) -> Result<Json<ClosePreviewResponse>, ApiError> {
    let Path(period) = path.map_err(|_| ApiError::invalid_request_parameter())?;
    let target = parse_period(&period)?;
    let budget = read_budget(State(state.clone()), Ok(Path(period.clone())))
        .await?
        .0;
    let (eligible, reason) =
        close_eligibility(&state.pool, state.workspace_id, target, &budget).await?;
    let categories = budget
        .categories
        .iter()
        .chain(budget.archived_categories.iter())
        .map(|category| ClosePreviewCategory {
            category_id: category.id,
            proposed_disposition: "carry".to_owned(),
        })
        .collect();
    Ok(Json(ClosePreviewResponse {
        close: ClosePreview {
            eligible,
            reason: reason.map(str::to_owned),
            expected_period_revision: budget.revision.clone(),
            categories,
        },
        budget,
    }))
}

async fn close_eligibility(
    pool: &PgPool,
    workspace_id: Uuid,
    target: NaiveDate,
    budget: &BudgetResponse,
) -> Result<(bool, Option<&'static str>), ApiError> {
    let already_closed: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM budget_close_snapshots WHERE workspace_id=$1 AND period_start=$2)",
    )
    .bind(workspace_id)
    .bind(target)
    .fetch_one(pool)
    .await
    .map_err(ApiError::database)?;
    if already_closed {
        return Ok((false, Some("period_already_closed")));
    }

    let latest_closed: Option<NaiveDate> = sqlx::query_scalar(
        "SELECT MAX(period_start) FROM budget_close_snapshots WHERE workspace_id=$1",
    )
    .bind(workspace_id)
    .fetch_one(pool)
    .await
    .map_err(ApiError::database)?;
    let first_fact: Option<NaiveDate> = sqlx::query_scalar(
        "SELECT MIN(event_date) FROM ( \
            SELECT business_date AS event_date FROM financial_openings WHERE workspace_id=$1 \
            UNION ALL SELECT business_date FROM financial_transactions WHERE workspace_id=$1 \
            UNION ALL SELECT period_start FROM budget_funding_effects WHERE workspace_id=$1 \
            UNION ALL SELECT period_start FROM budget_assignments WHERE workspace_id=$1 \
        ) events",
    )
    .bind(workspace_id)
    .fetch_one(pool)
    .await
    .map_err(ApiError::database)?;
    let Some(first_fact) = first_fact else {
        return Ok((false, Some("no_budget_period")));
    };
    let next_expected = latest_closed
        .map(next_month)
        .transpose()?
        .unwrap_or_else(|| first_of_month(first_fact));
    if target != next_expected {
        return Ok((false, Some("out_of_order")));
    }
    let current_period: NaiveDate = sqlx::query_scalar(
        "SELECT date_trunc('month', now() AT TIME ZONE 'America/Santiago')::date",
    )
    .fetch_one(pool)
    .await
    .map_err(ApiError::database)?;
    if target >= current_period {
        return Ok((false, Some("period_not_ended")));
    }
    if !budget.complete {
        return Ok((false, Some("incomplete_values")));
    }
    Ok((true, None))
}

async fn close_budget(
    State(state): State<AppState>,
    path: Result<Path<String>, PathRejection>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<impl IntoResponse, ApiError> {
    let Path(period) = path.map_err(|_| ApiError::invalid_request_parameter())?;
    let target = parse_period(&period)?;
    let key = idempotency_key(&headers)?;
    let generation = installation_generation(&headers)?;
    retry_transaction(|| {
        close_budget_attempt(
            state.clone(),
            generation,
            target,
            period.clone(),
            key,
            body.clone(),
        )
    })
    .await
}

async fn close_budget_attempt(
    state: AppState,
    generation: Uuid,
    target: NaiveDate,
    period_text: String,
    key: Uuid,
    body: Bytes,
) -> Result<impl IntoResponse, ApiError> {
    let route = format!("/api/budgets/{period_text}/close");
    let hash = Sha256::digest(&body).to_vec();
    let mut receipt_tx = state.pool.begin().await.map_err(ApiError::database)?;
    lock_workspace(&mut receipt_tx, state.workspace_id, generation).await?;
    if let Some(receipt) = find_receipt(
        &mut receipt_tx,
        state.workspace_id,
        key,
        "POST",
        &route,
        &hash,
    )
    .await?
    {
        receipt_tx.commit().await.map_err(ApiError::database)?;
        return Ok((
            StatusCode::from_u16(receipt.response_status as u16).unwrap_or(StatusCode::OK),
            Json(MutationReceipt {
                resource_id: receipt.resource_id.ok_or_else(ApiError::not_found)?,
                revision: receipt.resource_revision.unwrap_or(1),
                period_revision: None,
            }),
        ));
    }
    receipt_tx.commit().await.map_err(ApiError::database)?;

    let request: CloseRequest = parse_body(&body)?;
    let mut requested = BTreeMap::new();
    for choice in &request.categories {
        if !matches!(choice.disposition.as_str(), "carry" | "release")
            || requested
                .insert(choice.category_id, choice.disposition.as_str())
                .is_some()
        {
            return Err(ApiError::invalid(
                "categories",
                "invalid_or_duplicate_disposition",
            ));
        }
    }
    let Path(period_for_read) = Path(period_text.clone());
    let budget = read_budget(State(state.clone()), Ok(Path(period_for_read)))
        .await?
        .0;
    let preclosed: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM budget_close_snapshots WHERE workspace_id=$1 AND period_start=$2)",
    )
    .bind(state.workspace_id)
    .bind(target)
    .fetch_one(&state.pool)
    .await
    .map_err(ApiError::database)?;
    if preclosed {
        return Err(close_conflict("period_already_closed"));
    }
    if budget.revision != request.expected_period_revision {
        return Err(ApiError::stale_revision());
    }
    let (eligible, reason) =
        close_eligibility(&state.pool, state.workspace_id, target, &budget).await?;
    if !eligible {
        if reason == Some("incomplete_values") {
            return Err(ApiError::invalid("period", "incomplete_values"));
        }
        return Err(close_conflict(reason.unwrap_or("period_not_closable")));
    }
    let expected_categories = budget
        .categories
        .iter()
        .chain(budget.archived_categories.iter())
        .map(|category| category.id)
        .collect::<HashSet<_>>();
    if requested.len() != expected_categories.len()
        || requested
            .keys()
            .any(|category_id| !expected_categories.contains(category_id))
    {
        return Err(ApiError::invalid(
            "categories",
            "one_disposition_required_per_category",
        ));
    }

    let mut tx = state.pool.begin().await.map_err(ApiError::database)?;
    lock_workspace(&mut tx, state.workspace_id, generation).await?;
    if let Some(receipt) =
        find_receipt(&mut tx, state.workspace_id, key, "POST", &route, &hash).await?
    {
        tx.commit().await.map_err(ApiError::database)?;
        return Ok((
            StatusCode::from_u16(receipt.response_status as u16).unwrap_or(StatusCode::OK),
            Json(MutationReceipt {
                resource_id: receipt.resource_id.ok_or_else(ApiError::not_found)?,
                revision: receipt.resource_revision.unwrap_or(1),
                period_revision: None,
            }),
        ));
    }
    let already_closed: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM budget_close_snapshots WHERE workspace_id=$1 AND period_start=$2)",
    )
    .bind(state.workspace_id)
    .bind(target)
    .fetch_one(&mut *tx)
    .await
    .map_err(ApiError::database)?;
    if already_closed {
        return Err(close_conflict("period_already_closed"));
    }
    let locked_revision =
        period_revision_in(&mut tx, state.workspace_id, next_month(target)?).await?;
    if locked_revision != request.expected_period_revision {
        return Err(ApiError::stale_revision());
    }
    let funding =
        complete_amount(&budget.funding).ok_or_else(|| close_conflict("incomplete_values"))?;
    let ready = complete_amount(&budget.ready_to_assign)
        .ok_or_else(|| close_conflict("incomplete_values"))?;
    let activity =
        complete_amount(&budget.activity).ok_or_else(|| close_conflict("incomplete_values"))?;
    let uncategorized = complete_amount(&budget.uncategorized_activity)
        .ok_or_else(|| close_conflict("incomplete_values"))?;
    let snapshot_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO budget_close_snapshots \
         (id, workspace_id, period_start, revision, funding, ready_to_assign, activity, uncategorized_activity, available_to_budget_breakdown) \
         VALUES ($1,$2,$3,1,$4,$5,$6,$7,$8)",
    )
    .bind(snapshot_id)
    .bind(state.workspace_id)
    .bind(target)
    .bind(funding)
    .bind(ready)
    .bind(activity)
    .bind(uncategorized)
    .bind(serde_json::to_value(&budget.available_to_budget_breakdown).map_err(|_| stored_money_error("breakdown serialization"))?)
    .execute(&mut *tx)
    .await
    .map_err(ApiError::database)?;
    for category in budget
        .categories
        .iter()
        .chain(budget.archived_categories.iter())
    {
        let assigned = category.assigned.decimal();
        let category_activity = complete_amount(&category.activity)
            .ok_or_else(|| close_conflict("incomplete_values"))?;
        let remaining = complete_amount(&category.remaining)
            .ok_or_else(|| close_conflict("incomplete_values"))?;
        sqlx::query(
            "INSERT INTO budget_close_category_snapshots \
             (workspace_id,snapshot_id,category_id,assigned,activity,remaining,disposition,category_name,category_sort_order) \
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)",
        )
        .bind(state.workspace_id)
        .bind(snapshot_id)
        .bind(category.id)
        .bind(assigned)
        .bind(category_activity)
        .bind(remaining)
        .bind(requested[&category.id])
        .bind(&category.name)
        .bind(category.sort_order)
        .execute(&mut *tx)
        .await
        .map_err(ApiError::database)?;
    }
    save_close_fact_snapshots(&mut tx, state.workspace_id, target, snapshot_id).await?;
    save_receipt(
        &mut tx,
        state.workspace_id,
        key,
        "POST",
        &route,
        &hash,
        StatusCode::CREATED,
        snapshot_id,
        1,
    )
    .await?;
    tx.commit().await.map_err(ApiError::database)?;
    Ok((
        StatusCode::CREATED,
        Json(MutationReceipt {
            resource_id: snapshot_id,
            revision: 1,
            period_revision: None,
        }),
    ))
}

fn complete_amount(value: &BudgetValue) -> Option<Decimal> {
    match value {
        BudgetValue::Complete { amount } => Some(amount.decimal()),
        BudgetValue::Unavailable { .. } | BudgetValue::Incomplete { .. } => None,
    }
}

async fn save_close_fact_snapshots(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    workspace_id: Uuid,
    period: NaiveDate,
    snapshot_id: Uuid,
) -> Result<(), ApiError> {
    let opening_rows = sqlx::query_as::<_, OpeningCloseSourceRow>(
        "SELECT o.id AS opening_id, o.business_date, o.amount, w.currency::text AS currency, \
                a.budget_eligible, b.clp_amount AS liability_clp_basis \
         FROM financial_openings o \
         JOIN financial_wallets w ON w.workspace_id=o.workspace_id AND w.id=o.wallet_id \
         JOIN financial_accounts a ON a.workspace_id=w.workspace_id AND a.id=w.account_id \
         LEFT JOIN foreign_liability_bases b ON b.workspace_id=o.workspace_id AND b.opening_id=o.id \
         WHERE o.workspace_id=$1 AND o.business_date >= $2 \
           AND o.business_date < ($2 + interval '1 month')::date \
         ORDER BY o.business_date,o.id",
    )
    .bind(workspace_id)
    .bind(period)
    .fetch_all(&mut **tx)
    .await
    .map_err(ApiError::database)?;
    for opening in opening_rows.into_iter().filter(|row| row.budget_eligible) {
        let (value, rate_id) = opening_clp_value(
            tx,
            workspace_id,
            opening.currency.trim(),
            opening.business_date,
            opening.amount,
            opening.liability_clp_basis,
        )
        .await?;
        let Some(value) = value else {
            return Err(ApiError::invalid("period", "incomplete_values"));
        };
        sqlx::query(
            "INSERT INTO budget_close_opening_snapshots \
             (workspace_id,snapshot_id,opening_id,business_date,amount,currency,funding_amount,liability_clp_basis,rate_observation_id) \
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)",
        )
        .bind(workspace_id)
        .bind(snapshot_id)
        .bind(opening.opening_id)
        .bind(opening.business_date)
        .bind(opening.amount)
        .bind(opening.currency.trim())
        .bind(value)
        .bind(opening.liability_clp_basis)
        .bind(rate_id)
        .execute(&mut **tx)
        .await
        .map_err(ApiError::database)?;
    }

    let transactions: Vec<Uuid> = sqlx::query_scalar(
        "SELECT id FROM financial_transactions WHERE workspace_id=$1 AND business_date >= $2 \
           AND business_date < ($2 + interval '1 month')::date ORDER BY business_date,id",
    )
    .bind(workspace_id)
    .bind(period)
    .fetch_all(&mut **tx)
    .await
    .map_err(ApiError::database)?;
    for transaction_id in transactions {
        let state = capture_transaction_budget_state(tx, workspace_id, transaction_id).await?;
        let value = transaction_correction_value(tx, workspace_id, &state).await?;
        let side = state.effects.first();
        let activity = value.activity.get(&state.category_id);
        let activity_amount = activity.and_then(|activity| activity.amount);
        let activity_kind = activity.map(|activity| activity.kind.as_str());
        let activity_rate_id = activity.and_then(|activity| activity.rate_id);
        let original_amount = side
            .map(|effect| effect.amount.abs())
            .unwrap_or(Decimal::ZERO);
        let currency = side.map(|effect| effect.currency.as_str()).unwrap_or("CLP");
        let funding_missing_reason = value.funding.is_none().then_some("transaction_clp_value");
        let funding_amount = value.funding;
        if funding_amount.is_none() {
            return Err(ApiError::invalid("period", "incomplete_values"));
        }
        sqlx::query(
            "INSERT INTO budget_close_transaction_snapshots \
             (workspace_id,snapshot_id,transaction_id,business_date,transaction_kind,category_id,currency,original_amount,confirmed_clp_amount,conversion_evidence,activity_amount,activity_kind,activity_rate_observation_id,funding_amount,funding_missing_reason,funding_rate_observation_id,funding_additional_rate_observation_id) \
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17)",
        )
        .bind(workspace_id)
        .bind(snapshot_id)
        .bind(transaction_id)
        .bind(state.business_date)
        .bind(&state.kind)
        .bind(state.category_id)
        .bind(currency)
        .bind(original_amount)
        .bind(state.confirmed_clp_amount)
        .bind(state.conversion_evidence.clone())
        .bind(activity_amount)
        .bind(activity_kind)
        .bind(activity_rate_id)
        .bind(funding_amount)
        .bind(funding_missing_reason)
    .bind(value.funding_rate_observation_id)
    .bind(value.funding_additional_rate_observation_id)
        .execute(&mut **tx)
        .await
        .map_err(ApiError::database)?;
        let mut rate_ids = value.funding_rate_observation_ids.clone();
        rate_ids.extend(
            [
                value.funding_rate_observation_id,
                value.funding_additional_rate_observation_id,
            ]
            .into_iter()
            .flatten(),
        );
        rate_ids.sort_unstable();
        rate_ids.dedup();
        for rate_id in rate_ids {
            sqlx::query(
                "INSERT INTO budget_close_transaction_rate_observations \
                 (workspace_id,snapshot_id,transaction_id,rate_observation_id) VALUES($1,$2,$3,$4)",
            )
            .bind(workspace_id)
            .bind(snapshot_id)
            .bind(transaction_id)
            .bind(rate_id)
            .execute(&mut **tx)
            .await
            .map_err(ApiError::database)?;
        }
    }
    Ok(())
}

fn close_conflict(reason: &'static str) -> ApiError {
    ApiError::new(
        StatusCode::CONFLICT,
        reason,
        "The budget period cannot be closed in its current state.",
    )
}

pub(crate) async fn capture_transaction_budget_state(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    workspace_id: Uuid,
    transaction_id: Uuid,
) -> Result<BudgetTransactionState, ApiError> {
    let row = sqlx::query_as::<_, CorrectionTransactionRow>(
        "SELECT transaction_kind AS kind, business_date, category_id, \
                confirmed_clp_amount, conversion_evidence \
         FROM financial_transactions WHERE workspace_id=$1 AND id=$2",
    )
    .bind(workspace_id)
    .bind(transaction_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(ApiError::database)?
    .ok_or_else(ApiError::not_found)?;
    let effects = sqlx::query_as::<_, CorrectionTransactionSideRow>(
        "SELECT e.wallet_id, w.currency::text AS currency, e.effect_role AS role, e.amount, \
                a.budget_eligible, e.opening_treatment \
         FROM financial_wallet_effects e \
         JOIN financial_wallets w ON w.workspace_id=e.workspace_id AND w.id=e.wallet_id \
         JOIN financial_accounts a ON a.workspace_id=w.workspace_id AND a.id=w.account_id \
         WHERE e.workspace_id=$1 AND e.transaction_id=$2 ORDER BY e.effect_role,e.wallet_id",
    )
    .bind(workspace_id)
    .bind(transaction_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(ApiError::database)?
    .into_iter()
    .map(|effect| BudgetTransactionSide {
        wallet_id: effect.wallet_id,
        currency: effect.currency.trim().to_owned(),
        role: effect.role,
        amount: effect.amount,
        budget_eligible: effect.budget_eligible,
        opening_treatment: effect.opening_treatment,
    })
    .collect();
    Ok(BudgetTransactionState {
        transaction_id,
        kind: row.kind,
        business_date: row.business_date,
        category_id: row.category_id,
        confirmed_clp_amount: row.confirmed_clp_amount,
        conversion_evidence: row.conversion_evidence,
        effects,
    })
}

struct ActivityCorrectionValue {
    amount: Option<Decimal>,
    kind: String,
    currency: String,
    original_amount: Decimal,
    rate_id: Option<Uuid>,
}

struct TransactionCorrectionValue {
    funding: Option<Decimal>,
    funding_rate_observation_id: Option<Uuid>,
    funding_additional_rate_observation_id: Option<Uuid>,
    funding_rate_observation_ids: Vec<Uuid>,
    activity: BTreeMap<Option<Uuid>, ActivityCorrectionValue>,
    currency: String,
}

async fn correction_value_for_side(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    workspace_id: Uuid,
    currency: &str,
    date: NaiveDate,
    signed_amount: Decimal,
    confirmed_clp: Option<Decimal>,
) -> Result<(Option<Decimal>, Option<Uuid>), ApiError> {
    if let Some(confirmed) = confirmed_clp {
        return Ok((Some(confirmed.abs()), None));
    }
    if currency == "CLP" {
        return Ok((Some(signed_amount.abs()), None));
    }
    let Some(rate) = latest_stored_rate_in(tx, workspace_id, currency, date).await? else {
        return Ok((None, None));
    };
    let value = value_foreign_asset(
        MoneyAmount::new(signed_amount.abs())
            .map_err(|_| ApiError::invalid("amount", "out_of_range"))?,
        &rate,
    )
    .map_err(|_| ApiError::invalid("amount", "out_of_range"))?
    .decimal();
    Ok((Some(value), Some(rate.id)))
}

async fn transaction_correction_value(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    workspace_id: Uuid,
    state: &BudgetTransactionState,
) -> Result<TransactionCorrectionValue, ApiError> {
    let mut activity = BTreeMap::new();
    let mut funding_rate_observation_id = None;
    let mut funding_additional_rate_observation_id = None;
    let mut funding_rate_observation_ids = Vec::new();
    let currency = state
        .effects
        .first()
        .map(|effect| effect.currency.clone())
        .unwrap_or_else(|| "CLP".to_owned());
    let funding = match state.kind.as_str() {
        "income" => {
            let Some(effect) = state.effects.first() else {
                return Ok(TransactionCorrectionValue {
                    funding: None,
                    funding_rate_observation_id,
                    funding_additional_rate_observation_id,
                    funding_rate_observation_ids,
                    activity,
                    currency,
                });
            };
            if !effect.budget_eligible
                || effect.opening_treatment.as_deref() == Some("preserve_balance")
            {
                Some(Decimal::ZERO)
            } else {
                let (amount, rate_id) = correction_value_for_side(
                    tx,
                    workspace_id,
                    &effect.currency,
                    state.business_date,
                    effect.amount,
                    state.confirmed_clp_amount,
                )
                .await?;
                funding_rate_observation_id = rate_id;
                funding_rate_observation_ids.extend(rate_id);
                amount
            }
        }
        "expense" | "refund" => {
            let Some(effect) = state.effects.first() else {
                return Ok(TransactionCorrectionValue {
                    funding: Some(Decimal::ZERO),
                    funding_rate_observation_id,
                    funding_additional_rate_observation_id,
                    funding_rate_observation_ids,
                    activity,
                    currency,
                });
            };
            if effect.budget_eligible
                && effect.opening_treatment.as_deref() != Some("preserve_balance")
            {
                let (amount, rate_id) = correction_value_for_side(
                    tx,
                    workspace_id,
                    &effect.currency,
                    state.business_date,
                    effect.amount,
                    state.confirmed_clp_amount,
                )
                .await?;
                let signed = amount.map(|amount| {
                    if state.kind == "refund" {
                        -amount
                    } else {
                        amount
                    }
                });
                activity.insert(
                    state.category_id,
                    ActivityCorrectionValue {
                        amount: signed,
                        kind: state.kind.clone(),
                        currency: effect.currency.clone(),
                        original_amount: effect.amount.abs(),
                        rate_id,
                    },
                );
            }
            Some(Decimal::ZERO)
        }
        "transfer" => {
            let source = state.effects.iter().find(|effect| effect.role == "source");
            let destination = state
                .effects
                .iter()
                .find(|effect| effect.role == "destination");
            let (Some(source), Some(destination)) = (source, destination) else {
                return Ok(TransactionCorrectionValue {
                    funding: None,
                    funding_rate_observation_id,
                    funding_additional_rate_observation_id,
                    funding_rate_observation_ids,
                    activity,
                    currency,
                });
            };
            let source_value = if !source.budget_eligible {
                ValueState::Known(Decimal::ZERO)
            } else {
                if source.currency == "CLP" {
                    ValueState::Known(source.amount.abs())
                } else {
                    let (amount, observation_ids) = carrying_source_value_in(
                        tx,
                        workspace_id,
                        source.wallet_id,
                        &source.currency,
                        state.business_date,
                        state.transaction_id,
                        source.amount,
                    )
                    .await?;
                    funding_rate_observation_id = observation_ids.last().copied();
                    funding_rate_observation_ids.extend(observation_ids);
                    amount
                        .map(ValueState::Known)
                        .unwrap_or(ValueState::Missing(MissingValue::TransferSourceValuation))
                }
            };
            let received_clp = evidence_clp_amount(&state.conversion_evidence)?;
            let destination_value = if !destination.budget_eligible {
                ValueState::Known(Decimal::ZERO)
            } else {
                let (amount, rate_id) = correction_value_for_side(
                    tx,
                    workspace_id,
                    &destination.currency,
                    state.business_date,
                    destination.amount,
                    received_clp,
                )
                .await?;
                funding_additional_rate_observation_id = rate_id;
                funding_rate_observation_ids.extend(rate_id);
                amount.map(ValueState::Known).unwrap_or(ValueState::Missing(
                    MissingValue::TransferDestinationValuation,
                ))
            };
            match transfer_funding_effect(
                source.budget_eligible,
                source_value,
                destination.budget_eligible,
                destination_value,
            )
            .map_err(calculation_error)?
            {
                ValueState::Known(amount) => Some(amount),
                ValueState::Missing(_) => None,
            }
        }
        _ => None,
    };
    Ok(TransactionCorrectionValue {
        funding,
        funding_rate_observation_id,
        funding_additional_rate_observation_id,
        funding_rate_observation_ids,
        activity,
        currency,
    })
}

pub(crate) async fn record_closed_transaction_correction(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    workspace_id: Uuid,
    correction_period: NaiveDate,
    audit_correction_id: Uuid,
    old_state: &BudgetTransactionState,
    new_state: &BudgetTransactionState,
) -> Result<(), ApiError> {
    let old_closed = period_is_closed(tx, workspace_id, old_state.business_date).await?;
    let new_closed = period_is_closed(tx, workspace_id, new_state.business_date).await?;
    let new_value = transaction_correction_value(tx, workspace_id, new_state).await?;
    let stable_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM financial_corrections WHERE workspace_id=$1 AND record_kind='transaction' \
           AND transaction_id=$2 AND correction_period=$3 \
         ORDER BY prior_revision, created_at, id LIMIT 1",
    )
    .bind(workspace_id)
    .bind(old_state.transaction_id)
    .bind(correction_period)
    .fetch_optional(&mut **tx)
    .await
    .map_err(ApiError::database)?
    .unwrap_or(audit_correction_id);

    let target_value = if new_closed {
        new_value.funding
    } else {
        Some(Decimal::ZERO)
    };
    let current_funding_state = sqlx::query_as::<_, FundingCorrectionStateRow>(
        "SELECT period_start, correction_id, source_kind, opening_id, transaction_id, \
                baseline_amount, target_amount, missing_reason, baseline_rate_observation_id, \
                baseline_additional_rate_observation_id, target_rate_observation_id, \
                target_additional_rate_observation_id, baseline_business_date, target_business_date, currency \
         FROM budget_funding_correction_states WHERE workspace_id=$1 AND period_start=$2 \
           AND source_kind='transaction' AND transaction_id=$3",
    )
    .bind(workspace_id)
    .bind(correction_period)
    .bind(old_state.transaction_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(ApiError::database)?;
    let prior_funding_state = if current_funding_state.is_none() {
        sqlx::query_as::<_, FundingCorrectionStateRow>(
            "SELECT period_start, correction_id, source_kind, opening_id, transaction_id, \
                    baseline_amount, target_amount, missing_reason, baseline_rate_observation_id, \
                    baseline_additional_rate_observation_id, target_rate_observation_id, \
                    target_additional_rate_observation_id, baseline_business_date, target_business_date, currency \
             FROM budget_funding_correction_states WHERE workspace_id=$1 AND source_kind='transaction' \
               AND transaction_id=$2 AND period_start<$3 ORDER BY period_start DESC LIMIT 1",
        )
        .bind(workspace_id)
        .bind(old_state.transaction_id)
        .bind(correction_period)
        .fetch_optional(&mut **tx)
        .await
        .map_err(ApiError::database)?
    } else {
        None
    };
    let frozen_transaction =
        if old_closed && current_funding_state.is_none() && prior_funding_state.is_none() {
            load_close_transaction_baseline(tx, workspace_id, old_state).await?
        } else {
            None
        };
    let baseline_candidate = if let Some(row) = prior_funding_state.as_ref() {
        row.target_amount
    } else if let Some(row) = frozen_transaction.as_ref() {
        row.funding_amount
    } else if old_closed {
        None
    } else {
        Some(Decimal::ZERO)
    };
    let baseline = current_funding_state
        .as_ref()
        .and_then(|row| row.baseline_amount)
        .or(baseline_candidate);
    let baseline_rate_id = current_funding_state
        .as_ref()
        .and_then(|row| row.baseline_rate_observation_id)
        .or_else(|| {
            prior_funding_state
                .as_ref()
                .and_then(|row| row.target_rate_observation_id)
        })
        .or_else(|| {
            frozen_transaction
                .as_ref()
                .and_then(|row| row.funding_rate_observation_id)
        });
    let baseline_additional_rate_id = current_funding_state
        .as_ref()
        .and_then(|row| row.baseline_additional_rate_observation_id)
        .or_else(|| {
            prior_funding_state
                .as_ref()
                .and_then(|row| row.target_additional_rate_observation_id)
        })
        .or_else(|| {
            frozen_transaction
                .as_ref()
                .and_then(|row| row.funding_additional_rate_observation_id)
        });
    let net = match (baseline, target_value) {
        (Some(baseline), Some(target)) => Some(target.checked_sub(baseline).ok_or_else(|| {
            calculation_api_error("Funding correction exceeded the supported range.")
        })?),
        _ => None,
    };
    let missing_reason = if target_value.is_none() || baseline.is_none() {
        Some("transaction_clp_value")
    } else {
        None
    };
    sqlx::query(
        "INSERT INTO budget_funding_correction_states \
         (id,workspace_id,period_start,correction_id,source_kind,transaction_id,baseline_amount,target_amount,missing_reason,baseline_rate_observation_id,baseline_additional_rate_observation_id,target_rate_observation_id,target_additional_rate_observation_id,baseline_business_date,target_business_date,currency) \
         VALUES ($1,$2,$3,$4,'transaction',$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15) \
         ON CONFLICT (workspace_id,period_start,transaction_id) WHERE transaction_id IS NOT NULL \
         DO UPDATE SET target_amount=EXCLUDED.target_amount, missing_reason=EXCLUDED.missing_reason, \
           target_rate_observation_id=EXCLUDED.target_rate_observation_id, \
           target_additional_rate_observation_id=EXCLUDED.target_additional_rate_observation_id, \
           target_business_date=EXCLUDED.target_business_date, currency=EXCLUDED.currency",
    )
    .bind(Uuid::new_v4())
    .bind(workspace_id)
    .bind(correction_period)
    .bind(stable_id)
    .bind(old_state.transaction_id)
    .bind(baseline)
    .bind(target_value)
    .bind(missing_reason)
    .bind(baseline_rate_id)
    .bind(baseline_additional_rate_id)
    .bind(if new_closed { new_value.funding_rate_observation_id } else { None })
    .bind(if new_closed { new_value.funding_additional_rate_observation_id } else { None })
    .bind(old_state.business_date)
    .bind(new_state.business_date)
    .bind(new_value.currency.as_str())
    .execute(&mut **tx)
    .await
    .map_err(ApiError::database)?;
    sqlx::query(
        "DELETE FROM budget_funding_correction_rate_observations \
         WHERE workspace_id=$1 AND correction_id=$2",
    )
    .bind(workspace_id)
    .bind(stable_id)
    .execute(&mut **tx)
    .await
    .map_err(ApiError::database)?;
    let mut target_rate_ids = new_value.funding_rate_observation_ids.clone();
    target_rate_ids.extend(
        [
            new_value.funding_rate_observation_id,
            new_value.funding_additional_rate_observation_id,
        ]
        .into_iter()
        .flatten(),
    );
    target_rate_ids.sort_unstable();
    target_rate_ids.dedup();
    for rate_id in target_rate_ids {
        sqlx::query(
            "INSERT INTO budget_funding_correction_rate_observations \
             (workspace_id,correction_id,rate_observation_id) VALUES($1,$2,$3)",
        )
        .bind(workspace_id)
        .bind(stable_id)
        .bind(rate_id)
        .execute(&mut **tx)
        .await
        .map_err(ApiError::database)?;
    }
    sqlx::query(
        "INSERT INTO budget_funding_effects \
         (id,workspace_id,period_start,source_kind,correction_id,amount,currency,missing_reason) \
         VALUES ($1,$2,$3,'correction',$4,$5,'CLP',$6) \
         ON CONFLICT (workspace_id,period_start,correction_id) WHERE correction_id IS NOT NULL \
         DO UPDATE SET amount=EXCLUDED.amount, missing_reason=EXCLUDED.missing_reason",
    )
    .bind(Uuid::new_v4())
    .bind(workspace_id)
    .bind(correction_period)
    .bind(stable_id)
    .bind(net)
    .bind(missing_reason)
    .execute(&mut **tx)
    .await
    .map_err(ApiError::database)?;

    let old_activity = if old_closed {
        let mut baseline = prior_activity_correction_values(
            tx,
            workspace_id,
            old_state.transaction_id,
            correction_period,
        )
        .await?;
        if baseline.is_empty() {
            if let Some(row) = frozen_transaction.as_ref() {
                if let Some(kind) = row.activity_kind.as_deref() {
                    baseline.insert(
                        row.category_id,
                        ActivityCorrectionValue {
                            amount: row.activity_amount,
                            kind: kind.to_owned(),
                            currency: row.currency.trim().to_owned(),
                            original_amount: row.original_amount,
                            rate_id: row.activity_rate_observation_id,
                        },
                    );
                }
            }
        }
        let current_period_rows = sqlx::query_as::<_, ActivityEffectRow>(
            "SELECT period_start,transaction_id,category_id,baseline_amount,target_amount, \
                    baseline_kind,baseline_currency::text AS baseline_currency, \
                    baseline_original_amount,baseline_rate_observation_id,baseline_business_date, \
                    target_kind,target_currency::text AS target_currency,target_original_amount, \
                    target_rate_observation_id,target_business_date \
             FROM budget_activity_effects WHERE workspace_id=$1 AND period_start=$2 \
               AND transaction_id=$3",
        )
        .bind(workspace_id)
        .bind(correction_period)
        .bind(old_state.transaction_id)
        .fetch_all(&mut **tx)
        .await
        .map_err(ApiError::database)?;
        for row in current_period_rows {
            baseline.insert(
                row.category_id,
                ActivityCorrectionValue {
                    amount: row.baseline_amount,
                    kind: row.baseline_kind,
                    currency: row.baseline_currency.trim().to_owned(),
                    original_amount: row.baseline_original_amount,
                    rate_id: row.baseline_rate_observation_id,
                },
            );
        }
        baseline
    } else {
        BTreeMap::new()
    };
    let new_activity = if new_closed {
        new_value.activity
    } else {
        BTreeMap::new()
    };
    let categories = old_activity
        .keys()
        .chain(new_activity.keys())
        .copied()
        .collect::<HashSet<_>>();
    for category_id in categories {
        let baseline = old_activity.get(&category_id);
        let target = new_activity.get(&category_id);
        let baseline_fallback = old_state.effects.first();
        let target_fallback = new_state.effects.first();
        let baseline_kind = baseline
            .map(|value| value.kind.as_str())
            .unwrap_or("expense");
        let target_kind = target.map(|value| value.kind.as_str()).unwrap_or("expense");
        let baseline_currency = baseline
            .map(|value| value.currency.as_str())
            .or_else(|| baseline_fallback.map(|effect| effect.currency.as_str()))
            .unwrap_or("CLP");
        let target_currency = target
            .map(|value| value.currency.as_str())
            .or_else(|| target_fallback.map(|effect| effect.currency.as_str()))
            .unwrap_or("CLP");
        let baseline_original = baseline
            .map(|value| value.original_amount)
            .or_else(|| baseline_fallback.map(|effect| effect.amount.abs()))
            .unwrap_or(Decimal::ZERO);
        let target_original = target
            .map(|value| value.original_amount)
            .or_else(|| target_fallback.map(|effect| effect.amount.abs()))
            .unwrap_or(Decimal::ZERO);
        let baseline_amount = baseline
            .map(|value| value.amount)
            .unwrap_or(Some(Decimal::ZERO));
        let target_amount = target
            .map(|value| value.amount)
            .unwrap_or(Some(Decimal::ZERO));
        let old_rate_id = baseline.and_then(|value| value.rate_id);
        let new_rate_id = target.and_then(|value| value.rate_id);
        let baseline_date = old_state.business_date;
        let target_date = new_state.business_date;
        let values = (
            workspace_id,
            correction_period,
            old_state.transaction_id,
            category_id,
        );
        if category_id.is_some() {
            sqlx::query(
                "INSERT INTO budget_activity_effects \
                 (id,workspace_id,period_start,correction_id,transaction_id,category_id,baseline_amount,target_amount,baseline_kind,baseline_currency,baseline_original_amount,baseline_rate_observation_id,baseline_business_date,target_kind,target_currency,target_original_amount,target_rate_observation_id,target_business_date) \
                 VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18) \
                 ON CONFLICT (workspace_id,period_start,transaction_id,category_id) WHERE category_id IS NOT NULL \
                 DO UPDATE SET target_amount=EXCLUDED.target_amount,target_kind=EXCLUDED.target_kind, \
                   target_currency=EXCLUDED.target_currency,target_original_amount=EXCLUDED.target_original_amount, \
                   target_rate_observation_id=EXCLUDED.target_rate_observation_id,target_business_date=EXCLUDED.target_business_date",
            )
            .bind(Uuid::new_v4())
            .bind(values.0)
            .bind(values.1)
            .bind(stable_id)
            .bind(values.2)
            .bind(category_id)
            .bind(baseline_amount)
            .bind(target_amount)
            .bind(baseline_kind)
            .bind(baseline_currency)
            .bind(baseline_original)
            .bind(old_rate_id)
            .bind(baseline_date)
            .bind(target_kind)
            .bind(target_currency)
            .bind(target_original)
            .bind(new_rate_id)
            .bind(target_date)
            .execute(&mut **tx)
            .await
            .map_err(ApiError::database)?;
        } else {
            sqlx::query(
                "INSERT INTO budget_activity_effects \
                 (id,workspace_id,period_start,correction_id,transaction_id,category_id,baseline_amount,target_amount,baseline_kind,baseline_currency,baseline_original_amount,baseline_rate_observation_id,baseline_business_date,target_kind,target_currency,target_original_amount,target_rate_observation_id,target_business_date) \
                 VALUES ($1,$2,$3,$4,$5,NULL,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17) \
                 ON CONFLICT (workspace_id,period_start,transaction_id) WHERE category_id IS NULL \
                 DO UPDATE SET target_amount=EXCLUDED.target_amount,target_kind=EXCLUDED.target_kind, \
                   target_currency=EXCLUDED.target_currency,target_original_amount=EXCLUDED.target_original_amount, \
                   target_rate_observation_id=EXCLUDED.target_rate_observation_id,target_business_date=EXCLUDED.target_business_date",
            )
            .bind(Uuid::new_v4())
            .bind(values.0)
            .bind(values.1)
            .bind(stable_id)
            .bind(values.2)
            .bind(baseline_amount)
            .bind(target_amount)
            .bind(baseline_kind)
            .bind(baseline_currency)
            .bind(baseline_original)
            .bind(old_rate_id)
            .bind(baseline_date)
            .bind(target_kind)
            .bind(target_currency)
            .bind(target_original)
            .bind(new_rate_id)
            .bind(target_date)
            .execute(&mut **tx)
            .await
            .map_err(ApiError::database)?;
        }
    }
    Ok(())
}

async fn period_is_closed(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    workspace_id: Uuid,
    date: NaiveDate,
) -> Result<bool, ApiError> {
    let period = first_of_month(date);
    sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM budget_close_snapshots WHERE workspace_id=$1 AND period_start=$2)",
    )
    .bind(workspace_id)
    .bind(period)
    .fetch_one(&mut **tx)
    .await
    .map_err(ApiError::database)
}

async fn load_close_transaction_baseline(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    workspace_id: Uuid,
    state: &BudgetTransactionState,
) -> Result<Option<TransactionCloseBaselineRow>, ApiError> {
    sqlx::query_as::<_, TransactionCloseBaselineRow>(
        "SELECT c.funding_amount, c.funding_missing_reason, c.funding_rate_observation_id, \
                c.funding_additional_rate_observation_id, \
                c.activity_amount, c.activity_kind, c.category_id, c.currency::text AS currency, \
                c.original_amount, c.activity_rate_observation_id \
         FROM budget_close_transaction_snapshots c \
         JOIN budget_close_snapshots s ON s.workspace_id=c.workspace_id AND s.id=c.snapshot_id \
         WHERE c.workspace_id=$1 AND c.transaction_id=$2 \
           AND s.period_start=date_trunc('month',$3::date)::date \
         ORDER BY s.closed_at DESC LIMIT 1",
    )
    .bind(workspace_id)
    .bind(state.transaction_id)
    .bind(state.business_date)
    .fetch_optional(&mut **tx)
    .await
    .map_err(ApiError::database)
}

async fn prior_activity_correction_values(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    workspace_id: Uuid,
    transaction_id: Uuid,
    correction_period: NaiveDate,
) -> Result<BTreeMap<Option<Uuid>, ActivityCorrectionValue>, ApiError> {
    let rows = sqlx::query_as::<_, ActivityEffectRow>(
        "SELECT DISTINCT ON (category_id) period_start,transaction_id,category_id,baseline_amount,target_amount, \
                baseline_kind,baseline_currency::text AS baseline_currency,baseline_original_amount, \
                baseline_rate_observation_id,baseline_business_date,target_kind,target_currency::text AS target_currency, \
                target_original_amount,target_rate_observation_id,target_business_date \
         FROM budget_activity_effects WHERE workspace_id=$1 AND transaction_id=$2 AND period_start<$3 \
         ORDER BY category_id,period_start DESC",
    )
    .bind(workspace_id)
    .bind(transaction_id)
    .bind(correction_period)
    .fetch_all(&mut **tx)
    .await
    .map_err(ApiError::database)?;
    Ok(rows
        .into_iter()
        .map(|row| {
            (
                row.category_id,
                ActivityCorrectionValue {
                    amount: row.target_amount,
                    kind: row.target_kind,
                    currency: row.target_currency.trim().to_owned(),
                    original_amount: row.target_original_amount,
                    rate_id: row.target_rate_observation_id,
                },
            )
        })
        .collect())
}

pub(crate) async fn correction_period_for_source(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    workspace_id: Uuid,
    old_date: NaiveDate,
    new_date: NaiveDate,
) -> Result<Option<NaiveDate>, ApiError> {
    sqlx::query_scalar(
        "WITH affected AS ( \
           SELECT EXISTS(SELECT 1 FROM budget_close_snapshots WHERE workspace_id=$1 \
             AND period_start IN (date_trunc('month',$2::date)::date,date_trunc('month',$3::date)::date)) AS is_closed \
         ), current_month AS ( \
           SELECT date_trunc('month',now() AT TIME ZONE 'America/Santiago')::date AS period_start \
         ) \
         SELECT CASE WHEN NOT affected.is_closed THEN NULL \
           WHEN EXISTS(SELECT 1 FROM budget_close_snapshots c,current_month m \
             WHERE c.workspace_id=$1 AND c.period_start=m.period_start) \
           THEN (SELECT (max(period_start)+interval '1 month')::date FROM budget_close_snapshots WHERE workspace_id=$1) \
           ELSE (SELECT period_start FROM current_month) END FROM affected",
    )
    .bind(workspace_id)
    .bind(old_date)
    .bind(new_date)
    .fetch_one(&mut **tx)
    .await
    .map_err(ApiError::database)
}

pub(crate) async fn record_closed_opening_correction(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    workspace_id: Uuid,
    correction_period: NaiveDate,
    audit_correction_id: Uuid,
    opening_id: Uuid,
    business_date: NaiveDate,
    currency: &str,
    budget_eligible: bool,
    prior_amount: Decimal,
    prior_liability_basis: Option<Decimal>,
    target_amount: Decimal,
    target_liability_basis: Option<Decimal>,
) -> Result<(), ApiError> {
    let stable_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM financial_corrections WHERE workspace_id=$1 AND record_kind='opening' \
           AND opening_id=$2 AND correction_period=$3 ORDER BY prior_revision,created_at,id LIMIT 1",
    )
    .bind(workspace_id)
    .bind(opening_id)
    .bind(correction_period)
    .fetch_optional(&mut **tx)
    .await
    .map_err(ApiError::database)?
    .unwrap_or(audit_correction_id);

    let current_state = sqlx::query_as::<_, FundingCorrectionStateRow>(
        "SELECT period_start, correction_id, source_kind, opening_id, transaction_id, \
                baseline_amount, target_amount, missing_reason, baseline_rate_observation_id, \
                baseline_additional_rate_observation_id, target_rate_observation_id, \
                target_additional_rate_observation_id, baseline_business_date, target_business_date, currency \
         FROM budget_funding_correction_states WHERE workspace_id=$1 AND period_start=$2 \
           AND source_kind='opening' AND opening_id=$3",
    )
    .bind(workspace_id)
    .bind(correction_period)
    .bind(opening_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(ApiError::database)?;

    let prior_state = if current_state.is_none() {
        sqlx::query_as::<_, FundingCorrectionStateRow>(
            "SELECT period_start, correction_id, source_kind, opening_id, transaction_id, \
                    baseline_amount, target_amount, missing_reason, baseline_rate_observation_id, \
                    baseline_additional_rate_observation_id, target_rate_observation_id, \
                    target_additional_rate_observation_id, baseline_business_date, target_business_date, currency \
             FROM budget_funding_correction_states WHERE workspace_id=$1 AND source_kind='opening' \
               AND opening_id=$2 AND period_start<$3 ORDER BY period_start DESC LIMIT 1",
        )
        .bind(workspace_id)
        .bind(opening_id)
        .bind(correction_period)
        .fetch_optional(&mut **tx)
        .await
        .map_err(ApiError::database)?
    } else {
        None
    };
    let frozen_baseline = if current_state.is_none() && prior_state.is_none() {
        sqlx::query_as::<_, OpeningCloseBaselineRow>(
            "SELECT c.funding_amount, c.rate_observation_id \
             FROM budget_close_opening_snapshots c \
             JOIN budget_close_snapshots s ON s.workspace_id=c.workspace_id AND s.id=c.snapshot_id \
             WHERE c.workspace_id=$1 AND c.opening_id=$2 AND s.period_start= date_trunc('month',$3::date)::date",
        )
        .bind(workspace_id)
        .bind(opening_id)
        .bind(business_date)
        .fetch_optional(&mut **tx)
        .await
        .map_err(ApiError::database)?
    } else {
        None
    };
    let baseline_candidate = if !budget_eligible {
        Some(Decimal::ZERO)
    } else if let Some(row) = frozen_baseline.as_ref() {
        Some(row.funding_amount)
    } else if let Some(row) = prior_state.as_ref() {
        row.target_amount
    } else {
        opening_clp_value(
            tx,
            workspace_id,
            currency,
            business_date,
            prior_amount,
            prior_liability_basis,
        )
        .await?
        .0
    };
    let baseline_rate_id = current_state
        .as_ref()
        .and_then(|row| row.baseline_rate_observation_id)
        .or_else(|| {
            prior_state
                .as_ref()
                .and_then(|row| row.target_rate_observation_id)
        })
        .or_else(|| {
            frozen_baseline
                .as_ref()
                .and_then(|row| row.rate_observation_id)
        });
    let baseline = current_state
        .as_ref()
        .map(|row| row.baseline_amount)
        .unwrap_or(baseline_candidate);

    let (target_value, target_rate_id) = if !budget_eligible {
        (Some(Decimal::ZERO), None)
    } else {
        opening_clp_value(
            tx,
            workspace_id,
            currency,
            business_date,
            target_amount,
            target_liability_basis,
        )
        .await?
    };
    let missing_reason = if target_value.is_none() || baseline.is_none() {
        Some(if currency == "CLP" {
            "transaction_clp_value"
        } else if target_amount.is_sign_negative() {
            "foreign_liability_basis"
        } else {
            "foreign_asset_rate"
        })
    } else {
        None
    };
    let net = match (baseline, target_value) {
        (Some(baseline), Some(target)) => Some(target.checked_sub(baseline).ok_or_else(|| {
            calculation_api_error("Opening correction exceeded the supported range.")
        })?),
        _ => None,
    };
    sqlx::query(
        "INSERT INTO budget_funding_correction_states \
         (id,workspace_id,period_start,correction_id,source_kind,opening_id,baseline_amount,target_amount,missing_reason,baseline_rate_observation_id,target_rate_observation_id,baseline_business_date,target_business_date,currency) \
         VALUES ($1,$2,$3,$4,'opening',$5,$6,$7,$8,$9,$10,$11,$12,$13) \
         ON CONFLICT (workspace_id,period_start,opening_id) WHERE opening_id IS NOT NULL \
         DO UPDATE SET target_amount=EXCLUDED.target_amount,missing_reason=EXCLUDED.missing_reason, \
           target_rate_observation_id=EXCLUDED.target_rate_observation_id,target_business_date=EXCLUDED.target_business_date,currency=EXCLUDED.currency",
    )
    .bind(Uuid::new_v4())
    .bind(workspace_id)
    .bind(correction_period)
    .bind(stable_id)
    .bind(opening_id)
    .bind(baseline)
    .bind(target_value)
    .bind(missing_reason)
    .bind(baseline_rate_id)
    .bind(target_rate_id)
    .bind(business_date)
    .bind(business_date)
    .bind(currency)
    .execute(&mut **tx)
    .await
    .map_err(ApiError::database)?;
    sqlx::query(
        "INSERT INTO budget_funding_effects \
         (id,workspace_id,period_start,source_kind,correction_id,amount,currency,missing_reason) \
         VALUES ($1,$2,$3,'correction',$4,$5,'CLP',$6) \
         ON CONFLICT (workspace_id,period_start,correction_id) WHERE correction_id IS NOT NULL \
         DO UPDATE SET amount=EXCLUDED.amount,missing_reason=EXCLUDED.missing_reason",
    )
    .bind(Uuid::new_v4())
    .bind(workspace_id)
    .bind(correction_period)
    .bind(stable_id)
    .bind(net)
    .bind(missing_reason)
    .execute(&mut **tx)
    .await
    .map_err(ApiError::database)?;
    sqlx::query("UPDATE financial_corrections SET opening_target_rate_observation_id=$3 WHERE workspace_id=$1 AND id=$2")
        .bind(workspace_id)
        .bind(audit_correction_id)
        .bind(target_rate_id)
        .execute(&mut **tx)
        .await
        .map_err(ApiError::database)?;
    Ok(())
}

async fn opening_clp_value(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    workspace_id: Uuid,
    currency: &str,
    date: NaiveDate,
    signed_amount: Decimal,
    liability_basis: Option<Decimal>,
) -> Result<(Option<Decimal>, Option<Uuid>), ApiError> {
    if currency == "CLP" {
        return Ok((Some(signed_amount), None));
    }
    if signed_amount.is_sign_negative() {
        return Ok((liability_basis, None));
    }
    if signed_amount.is_zero() {
        return Ok((Some(Decimal::ZERO), None));
    }
    let Some(rate) = latest_stored_rate_in(tx, workspace_id, currency, date).await? else {
        return Ok((None, None));
    };
    let value = value_foreign_asset(
        MoneyAmount::new(signed_amount).map_err(|_| ApiError::invalid("amount", "out_of_range"))?,
        &rate,
    )
    .map_err(|_| ApiError::invalid("amount", "out_of_range"))?
    .decimal();
    Ok((Some(value), Some(rate.id)))
}

async fn refresh_open_correction_targets(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    workspace_id: Uuid,
    correction_period: NaiveDate,
) -> Result<(), ApiError> {
    let closed: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM budget_close_snapshots \
         WHERE workspace_id=$1 AND period_start=$2)",
    )
    .bind(workspace_id)
    .bind(correction_period)
    .fetch_one(&mut **tx)
    .await
    .map_err(ApiError::database)?;
    if closed {
        return Ok(());
    }
    let funding_states = sqlx::query_as::<_, FundingCorrectionRefreshRow>(
        "SELECT correction_id, opening_id, transaction_id, baseline_amount \
         FROM budget_funding_correction_states WHERE workspace_id=$1 AND period_start=$2 \
         ORDER BY correction_id",
    )
    .bind(workspace_id)
    .bind(correction_period)
    .fetch_all(&mut **tx)
    .await
    .map_err(ApiError::database)?;
    for state in funding_states {
        let (target_date, currency, target, rate_id, additional_rate_id, missing_reason, rate_ids) =
            if let Some(opening_id) = state.opening_id {
                let row = sqlx::query_as::<_, (NaiveDate, Decimal, String, Option<Decimal>)>(
                    "SELECT o.business_date,o.amount,w.currency::text,b.clp_amount \
                     FROM financial_openings o JOIN financial_wallets w \
                       ON w.workspace_id=o.workspace_id AND w.id=o.wallet_id \
                     LEFT JOIN foreign_liability_bases b \
                       ON b.workspace_id=o.workspace_id AND b.opening_id=o.id \
                     WHERE o.workspace_id=$1 AND o.id=$2",
                )
                .bind(workspace_id)
                .bind(opening_id)
                .fetch_one(&mut **tx)
                .await
                .map_err(ApiError::database)?;
                let (value, rate_id) =
                    opening_clp_value(tx, workspace_id, row.2.trim(), row.0, row.1, row.3).await?;
                let target = if period_is_closed(tx, workspace_id, row.0).await? {
                    value
                } else {
                    Some(Decimal::ZERO)
                };
                (
                    row.0,
                    row.2.trim().to_owned(),
                    target,
                    rate_id,
                    None,
                    value.is_none().then_some("opening_clp_value"),
                    Vec::new(),
                )
            } else if let Some(transaction_id) = state.transaction_id {
                let current =
                    capture_transaction_budget_state(tx, workspace_id, transaction_id).await?;
                let value = transaction_correction_value(tx, workspace_id, &current).await?;
                let closed = period_is_closed(tx, workspace_id, current.business_date).await?;
                let target = if closed {
                    value.funding
                } else {
                    Some(Decimal::ZERO)
                };
                let missing = target.is_none();
                let mut rate_ids = if closed {
                    value.funding_rate_observation_ids.clone()
                } else {
                    Vec::new()
                };
                if closed {
                    rate_ids.extend(
                        [
                            value.funding_rate_observation_id,
                            value.funding_additional_rate_observation_id,
                        ]
                        .into_iter()
                        .flatten(),
                    );
                    rate_ids.sort_unstable();
                    rate_ids.dedup();
                }
                (
                    current.business_date,
                    value.currency,
                    target,
                    if closed {
                        value.funding_rate_observation_id
                    } else {
                        None
                    },
                    if closed {
                        value.funding_additional_rate_observation_id
                    } else {
                        None
                    },
                    missing.then_some("transaction_clp_value"),
                    rate_ids,
                )
            } else {
                continue;
            };
        let delta = match (state.baseline_amount, target) {
            (Some(baseline), Some(target)) => {
                Some(target.checked_sub(baseline).ok_or_else(|| {
                    calculation_api_error("Funding correction exceeded the supported range.")
                })?)
            }
            _ => None,
        };
        sqlx::query(
            "UPDATE budget_funding_correction_states \
             SET target_amount=$3, missing_reason=$4, target_rate_observation_id=$5, \
                 target_additional_rate_observation_id=$6,target_business_date=$7,currency=$8 \
             WHERE workspace_id=$1 AND period_start=$2 AND correction_id=$9",
        )
        .bind(workspace_id)
        .bind(correction_period)
        .bind(target)
        .bind(missing_reason)
        .bind(rate_id)
        .bind(additional_rate_id)
        .bind(target_date)
        .bind(currency.trim())
        .bind(state.correction_id)
        .execute(&mut **tx)
        .await
        .map_err(ApiError::database)?;
        sqlx::query(
            "DELETE FROM budget_funding_correction_rate_observations \
             WHERE workspace_id=$1 AND correction_id=$2",
        )
        .bind(workspace_id)
        .bind(state.correction_id)
        .execute(&mut **tx)
        .await
        .map_err(ApiError::database)?;
        for rate_id in rate_ids {
            sqlx::query(
                "INSERT INTO budget_funding_correction_rate_observations \
                 (workspace_id,correction_id,rate_observation_id) VALUES($1,$2,$3)",
            )
            .bind(workspace_id)
            .bind(state.correction_id)
            .bind(rate_id)
            .execute(&mut **tx)
            .await
            .map_err(ApiError::database)?;
        }
        sqlx::query(
            "UPDATE budget_funding_effects SET amount=$3,missing_reason=$4 \
             WHERE workspace_id=$1 AND period_start=$2 AND correction_id=$5 AND source_kind='correction'",
        )
        .bind(workspace_id)
        .bind(correction_period)
        .bind(delta)
        .bind(missing_reason)
        .bind(state.correction_id)
        .execute(&mut **tx)
        .await
        .map_err(ApiError::database)?;

        if let Some(transaction_id) = state.transaction_id {
            let current =
                capture_transaction_budget_state(tx, workspace_id, transaction_id).await?;
            let value = transaction_correction_value(tx, workspace_id, &current).await?;
            let closed = period_is_closed(tx, workspace_id, current.business_date).await?;
            let activity_rows = sqlx::query_as::<_, ActivityCorrectionRefreshRow>(
                "SELECT category_id FROM budget_activity_effects \
                 WHERE workspace_id=$1 AND period_start=$2 AND transaction_id=$3 \
                 ORDER BY category_id NULLS FIRST",
            )
            .bind(workspace_id)
            .bind(correction_period)
            .bind(transaction_id)
            .fetch_all(&mut **tx)
            .await
            .map_err(ApiError::database)?;
            for row in activity_rows {
                let target = if closed {
                    match value.activity.get(&row.category_id) {
                        Some(activity) => activity.amount,
                        None => Some(Decimal::ZERO),
                    }
                } else {
                    Some(Decimal::ZERO)
                };
                sqlx::query(
                    "UPDATE budget_activity_effects SET target_amount=$4, \
                     target_business_date=$5, target_kind=$6, target_currency=$7, \
                     target_original_amount=$8, target_rate_observation_id=$9 \
                     WHERE workspace_id=$1 AND period_start=$2 AND transaction_id=$3 \
                       AND category_id IS NOT DISTINCT FROM $10",
                )
                .bind(workspace_id)
                .bind(correction_period)
                .bind(transaction_id)
                .bind(target)
                .bind(current.business_date)
                .bind(if current.kind == "refund" {
                    "refund"
                } else {
                    "expense"
                })
                .bind(value.currency.trim())
                .bind(
                    current
                        .effects
                        .first()
                        .map(|effect| effect.amount.abs())
                        .unwrap_or(Decimal::ZERO),
                )
                .bind(
                    closed
                        .then(|| {
                            value
                                .activity
                                .get(&row.category_id)
                                .and_then(|activity| activity.rate_id)
                        })
                        .flatten(),
                )
                .bind(row.category_id)
                .execute(&mut **tx)
                .await
                .map_err(ApiError::database)?;
            }
        }
    }
    Ok(())
}

async fn apply_open_period_revaluations(
    pool: &PgPool,
    workspace_id: Uuid,
    target: NaiveDate,
) -> Result<
    (
        HashMap<(String, NaiveDate), Option<FxObservation>>,
        Option<String>,
    ),
    ApiError,
> {
    for _ in 0..3 {
        if let Some(result) =
            apply_open_period_revaluations_attempt(pool, workspace_id, target).await?
        {
            return Ok(result);
        }
    }
    Err(ApiError::new(
        StatusCode::SERVICE_UNAVAILABLE,
        "fx_inputs_changed_during_revaluation",
        "Foreign-exchange inputs changed while the budget was valued. Retry the request.",
    ))
}

async fn prefetch_open_correction_rates(
    pool: &PgPool,
    workspace_id: Uuid,
    correction_period: NaiveDate,
) -> Result<(), ApiError> {
    let requests: Vec<(String, NaiveDate, bool)> = sqlx::query_as(
        "WITH target_transactions AS ( \
             SELECT t.id,t.business_date FROM budget_funding_correction_states s \
             JOIN financial_transactions t ON t.workspace_id=s.workspace_id AND t.id=s.transaction_id \
             WHERE s.workspace_id=$1 AND s.period_start=$2 \
         ), transfer_source_wallets AS ( \
             SELECT e.wallet_id,t.business_date AS target_date FROM target_transactions t \
             JOIN financial_wallet_effects e ON e.workspace_id=$1 AND e.transaction_id=t.id \
             WHERE e.effect_role='source' AND EXISTS ( \
                 SELECT 1 FROM financial_transactions x WHERE x.workspace_id=$1 AND x.id=t.id AND x.transaction_kind='transfer' \
             ) \
         ), requests AS ( \
             SELECT w.currency::text AS currency,o.business_date AS rate_date, \
                    (s.target_rate_observation_id IS NULL) AS force_fetch \
             FROM budget_funding_correction_states s JOIN financial_openings o \
               ON o.workspace_id=s.workspace_id AND o.id=s.opening_id \
             JOIN financial_wallets w ON w.workspace_id=o.workspace_id AND w.id=o.wallet_id \
             WHERE s.workspace_id=$1 AND s.period_start=$2 AND w.currency<>'CLP' \
             UNION SELECT w.currency::text,t.business_date, \
                    (s.target_rate_observation_id IS NULL OR s.target_additional_rate_observation_id IS NULL) \
             FROM target_transactions t JOIN budget_funding_correction_states s \
               ON s.workspace_id=$1 AND s.transaction_id=t.id AND s.period_start=$2 \
             JOIN financial_wallet_effects e ON e.workspace_id=$1 AND e.transaction_id=t.id \
             JOIN financial_wallets w ON w.workspace_id=e.workspace_id AND w.id=e.wallet_id WHERE w.currency<>'CLP' \
             UNION SELECT w.currency::text,o.business_date,false \
             FROM transfer_source_wallets s JOIN financial_openings o \
               ON o.workspace_id=$1 AND o.wallet_id=s.wallet_id AND o.business_date<=s.target_date \
             JOIN financial_wallets w ON w.workspace_id=o.workspace_id AND w.id=o.wallet_id WHERE w.currency<>'CLP' \
             UNION SELECT w.currency::text,t.business_date,false \
             FROM transfer_source_wallets s JOIN financial_wallet_effects e \
               ON e.workspace_id=$1 AND e.wallet_id=s.wallet_id \
             JOIN financial_transactions t ON t.workspace_id=e.workspace_id AND t.id=e.transaction_id \
             JOIN financial_wallets w ON w.workspace_id=e.workspace_id AND w.id=e.wallet_id \
             WHERE t.business_date<=s.target_date AND w.currency<>'CLP' \
         ) SELECT DISTINCT currency,rate_date,bool_or(force_fetch) FROM requests \
           GROUP BY currency,rate_date ORDER BY currency,rate_date",
    )
    .bind(workspace_id)
    .bind(correction_period)
    .fetch_all(pool)
    .await
    .map_err(ApiError::database)?;
    let Ok(provider) = FrankfurterProvider::new() else {
        return Ok(());
    };
    for (currency, rate_date, _force_fetch) in requests {
        if exact_stored_rate(pool, workspace_id, currency.trim(), rate_date)
            .await?
            .is_some()
        {
            continue;
        }
        match get_observation(
            pool,
            workspace_id,
            currency.trim(),
            "CLP",
            rate_date,
            &provider,
        )
        .await
        {
            Ok(_)
            | Err(
                FxError::RateUnavailable
                | FxError::ProviderUnavailable
                | FxError::InvalidProviderResponse,
            ) => {}
            Err(FxError::Database(error)) => return Err(ApiError::database(error)),
            Err(
                FxError::InvalidCurrency | FxError::AmountOutOfRange | FxError::ArithmeticOverflow,
            ) => {
                return Err(ApiError::invalid("currency", "invalid_or_out_of_range"));
            }
        }
    }
    Ok(())
}

async fn apply_open_period_revaluations_attempt(
    pool: &PgPool,
    workspace_id: Uuid,
    target: NaiveDate,
) -> Result<
    Option<(
        HashMap<(String, NaiveDate), Option<FxObservation>>,
        Option<String>,
    )>,
    ApiError,
> {
    let current_period: NaiveDate = sqlx::query_scalar(
        "SELECT date_trunc('month', now() AT TIME ZONE 'America/Santiago')::date",
    )
    .fetch_one(pool)
    .await
    .map_err(ApiError::database)?;
    if target != current_period {
        prefetch_open_correction_rates(pool, workspace_id, target).await?;
        let mut tx = pool.begin().await.map_err(ApiError::database)?;
        sqlx::query_scalar::<_, Uuid>("SELECT id FROM workspaces WHERE id=$1 FOR UPDATE")
            .bind(workspace_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(ApiError::database)?
            .ok_or_else(ApiError::not_found)?;
        refresh_open_correction_targets(&mut tx, workspace_id, target).await?;
        let revision = period_revision_in(&mut tx, workspace_id, next_month(target)?).await?;
        tx.commit().await.map_err(ApiError::database)?;
        return Ok(Some((HashMap::new(), Some(revision))));
    }
    let as_of: NaiveDate =
        sqlx::query_scalar("SELECT (now() AT TIME ZONE 'America/Santiago')::date")
            .fetch_one(pool)
            .await
            .map_err(ApiError::database)?;

    let requests = sqlx::query_as::<_, RateRequestRow>(
        "SELECT DISTINCT currency, rate_date FROM ( \
            SELECT w.currency::text AS currency, o.business_date AS rate_date \
            FROM financial_openings o \
            JOIN financial_wallets w ON w.workspace_id=o.workspace_id AND w.id=o.wallet_id \
            WHERE o.workspace_id=$1 AND w.currency <> 'CLP' AND o.business_date <= $2 \
            UNION \
            SELECT w.currency::text, t.business_date \
            FROM financial_wallet_effects e \
            JOIN financial_transactions t ON t.workspace_id=e.workspace_id AND t.id=e.transaction_id \
            JOIN financial_wallets w ON w.workspace_id=e.workspace_id AND w.id=e.wallet_id \
            WHERE e.workspace_id=$1 AND w.currency <> 'CLP' AND t.business_date <= $2 \
            UNION \
            SELECT w.currency::text, $2::date FROM financial_wallets w \
            WHERE w.workspace_id=$1 AND w.currency <> 'CLP' \
         ) rates ORDER BY currency, rate_date",
    )
    .bind(workspace_id)
    .bind(as_of)
    .fetch_all(pool)
    .await
    .map_err(ApiError::database)?;
    let provider = FrankfurterProvider::new().ok();
    let mut rate_cache = HashMap::<(String, NaiveDate), Option<FxObservation>>::new();
    let mut pending = Vec::new();
    for request in requests {
        let currency = request.currency.trim().to_owned();
        if request.rate_date != as_of {
            if let Some(stored) =
                exact_stored_rate(pool, workspace_id, &currency, request.rate_date).await?
            {
                rate_cache.insert((currency, request.rate_date), Some(stored));
                continue;
            }
        }
        pending.push(RateRequestRow {
            currency,
            rate_date: request.rate_date,
        });
    }
    if let Some(provider) = provider {
        pending.sort_by_key(|request| {
            (
                request.rate_date != as_of,
                request.currency.clone(),
                request.rate_date,
            )
        });
        let mut unavailable_currencies = HashSet::new();
        for batch in pending.chunks(8) {
            let mut tasks = JoinSet::new();
            for request in batch
                .iter()
                .filter(|request| !unavailable_currencies.contains(&request.currency))
                .cloned()
            {
                let provider = provider.clone();
                let pool = pool.clone();
                let workspace_id = workspace_id;
                tasks.spawn(async move {
                    let result = get_observation(
                        &pool,
                        workspace_id,
                        request.currency.trim(),
                        "CLP",
                        request.rate_date,
                        &provider,
                    )
                    .await;
                    (request, result)
                });
            }
            while let Some(joined) = tasks.join_next().await {
                let (request, result) = joined.map_err(|_| {
                    ApiError::new(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "internal_error",
                        "Exchange-rate retrieval could not be completed.",
                    )
                })?;
                let provider_unavailable = match &result {
                    Ok(observation) => observation.provider_unavailable,
                    Err(
                        FxError::ProviderUnavailable
                        | FxError::RateUnavailable
                        | FxError::InvalidProviderResponse,
                    ) => true,
                    Err(_) => false,
                };
                if provider_unavailable {
                    unavailable_currencies.insert(request.currency.clone());
                }
                rate_cache.insert(
                    (request.currency.clone(), request.rate_date),
                    provider_result_to_rate(result)?,
                );
            }
            for request in batch {
                let key = (request.currency.clone(), request.rate_date);
                if !rate_cache.contains_key(&key)
                    && unavailable_currencies.contains(&request.currency)
                {
                    let fallback = latest_stored_rate(
                        pool,
                        workspace_id,
                        &request.currency,
                        request.rate_date,
                    )
                    .await
                    .map_err(ApiError::database)?;
                    rate_cache.insert(key, fallback);
                }
            }
        }
    } else {
        for request in pending {
            let rate = latest_stored_rate(pool, workspace_id, &request.currency, request.rate_date)
                .await
                .map_err(ApiError::database)?;
            rate_cache.insert((request.currency, request.rate_date), rate);
        }
    }

    let mut tx = pool.begin().await.map_err(ApiError::database)?;
    sqlx::query_scalar::<_, Uuid>("SELECT id FROM workspaces WHERE id=$1 FOR UPDATE")
        .bind(workspace_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(ApiError::database)?
        .ok_or_else(ApiError::not_found)?;

    let requests = rate_cache.keys().cloned().collect::<Vec<_>>();
    for (currency, rate_date) in requests {
        let rate = latest_stored_rate_in(&mut tx, workspace_id, &currency, rate_date).await?;
        rate_cache.insert((currency, rate_date), rate);
    }

    // Provider requests completed before the workspace lock. Re-read facts and
    // rates under the lock before applying any correction.
    let wallets = sqlx::query_as::<_, RevaluationWalletRow>(
        "SELECT w.id AS wallet_id, w.currency::text AS currency, a.budget_eligible, \
                COALESCE((SELECT sum(o.amount) FROM financial_openings o \
                    WHERE o.workspace_id=w.workspace_id AND o.wallet_id=w.id AND o.business_date <= $2), 0) \
                + COALESCE((SELECT sum(e.amount) FROM financial_wallet_effects e \
                    JOIN financial_transactions t ON t.workspace_id=e.workspace_id AND t.id=e.transaction_id \
                    WHERE e.workspace_id=w.workspace_id AND e.wallet_id=w.id AND t.business_date <= $2 \
                      AND e.opening_treatment IS DISTINCT FROM 'preserve_balance'), 0) AS current_balance \
         FROM financial_wallets w \
         JOIN financial_accounts a ON a.workspace_id=w.workspace_id AND a.id=w.account_id \
         WHERE w.workspace_id=$1 AND w.currency <> 'CLP' ORDER BY w.id",
    )
    .bind(workspace_id)
    .bind(as_of)
    .fetch_all(&mut *tx)
    .await
    .map_err(ApiError::database)?;

    for wallet in wallets {
        if !wallet.budget_eligible || wallet.current_balance <= Decimal::ZERO {
            sqlx::query(
                "UPDATE budget_funding_effects SET amount=0 \
                 WHERE workspace_id=$1 AND period_start=$2 AND source_kind='valuation' AND wallet_id=$3",
            )
            .bind(workspace_id)
            .bind(target)
            .bind(wallet.wallet_id)
            .execute(&mut *tx)
            .await
            .map_err(ApiError::database)?;
            continue;
        }
        let currency = wallet.currency.trim();
        let current_key = (currency.to_owned(), as_of);
        if !rate_cache.contains_key(&current_key) {
            tx.rollback().await.map_err(ApiError::database)?;
            return Ok(None);
        }
        let Some(current) = rate_cache.get(&current_key).cloned().flatten() else {
            continue;
        };
        let flows = sqlx::query_as::<_, RevaluationFlowRow>(
            "SELECT business_date, amount, record_kind, transaction_kind, effect_role, \
                    confirmed_clp_amount, conversion_evidence, liability_clp_basis FROM ( \
                SELECT o.business_date, o.amount, 'opening'::text AS record_kind, \
                       NULL::text AS transaction_kind, NULL::text AS effect_role, \
                       NULL::numeric AS confirmed_clp_amount, NULL::jsonb AS conversion_evidence, \
                       b.clp_amount AS liability_clp_basis \
                FROM financial_openings o \
                LEFT JOIN foreign_liability_bases b ON b.workspace_id=o.workspace_id AND b.opening_id=o.id \
                WHERE o.workspace_id=$1 AND o.wallet_id=$2 AND o.business_date <= $3 \
                UNION ALL \
                SELECT t.business_date, e.amount, 'transaction'::text, t.transaction_kind, \
                       e.effect_role, t.confirmed_clp_amount, t.conversion_evidence, NULL::numeric \
                FROM financial_wallet_effects e \
                JOIN financial_transactions t ON t.workspace_id=e.workspace_id AND t.id=e.transaction_id \
                WHERE e.workspace_id=$1 AND e.wallet_id=$2 AND t.business_date <= $3 \
                  AND e.opening_treatment IS DISTINCT FROM 'preserve_balance' \
            ) flows ORDER BY business_date, record_kind, transaction_kind, effect_role",
        )
        .bind(workspace_id)
        .bind(wallet.wallet_id)
        .bind(as_of)
        .fetch_all(&mut *tx)
        .await
        .map_err(ApiError::database)?;
        let mut carrying_basis = Decimal::ZERO;
        let mut basis_complete = true;
        for flow in flows {
            if flow.amount.is_zero() {
                continue;
            }
            let clp_amount = if flow.record_kind == "opening" && flow.amount.is_sign_negative() {
                let Some(basis) = flow.liability_clp_basis else {
                    basis_complete = false;
                    break;
                };
                basis
            } else if let Some(confirmed) = flow.confirmed_clp_amount {
                if flow.record_kind == "transaction" {
                    signed_value(flow.amount, confirmed.abs())?
                } else {
                    Decimal::ZERO
                }
            } else if flow.transaction_kind.as_deref() == Some("transfer")
                && flow.effect_role.as_deref() == Some("destination")
            {
                if let Some(clp) = evidence_clp_amount(&flow.conversion_evidence)? {
                    signed_value(flow.amount, clp)?
                } else {
                    let flow_key = (currency.to_owned(), flow.business_date);
                    if !rate_cache.contains_key(&flow_key) {
                        tx.rollback().await.map_err(ApiError::database)?;
                        return Ok(None);
                    }
                    match rate_cache.get(&flow_key).cloned().flatten() {
                        Some(rate) => valued_signed_amount(flow.amount, &rate)?,
                        None => {
                            basis_complete = false;
                            break;
                        }
                    }
                }
            } else {
                let flow_key = (currency.to_owned(), flow.business_date);
                if !rate_cache.contains_key(&flow_key) {
                    tx.rollback().await.map_err(ApiError::database)?;
                    return Ok(None);
                }
                match rate_cache.get(&flow_key).cloned().flatten() {
                    Some(rate) => valued_signed_amount(flow.amount, &rate)?,
                    None => {
                        basis_complete = false;
                        break;
                    }
                }
            };
            carrying_basis = carrying_basis.checked_add(clp_amount).ok_or_else(|| {
                calculation_api_error("Carrying value exceeded the supported range.")
            })?;
        }
        if !basis_complete {
            continue;
        }
        let prior_valuations: Decimal = sqlx::query_scalar(
            "SELECT COALESCE(sum(amount), 0) FROM budget_funding_effects \
             WHERE workspace_id=$1 AND wallet_id=$2 AND source_kind='valuation' \
               AND period_start <= $3 \
               AND NOT (period_start=$3 AND rate_observation_id=$4)",
        )
        .bind(workspace_id)
        .bind(wallet.wallet_id)
        .bind(target)
        .bind(current.id)
        .fetch_one(&mut *tx)
        .await
        .map_err(ApiError::database)?;
        carrying_basis = carrying_basis
            .checked_add(prior_valuations)
            .ok_or_else(|| calculation_api_error("Carrying value exceeded the supported range."))?;
        let amount = MoneyAmount::new(wallet.current_balance)
            .map_err(|_| ApiError::invalid("amount", "out_of_range"))?;
        let current_value = value_foreign_asset(amount, &current)
            .map_err(|_| ApiError::invalid("amount", "out_of_range"))?
            .decimal();
        let delta = current_value
            .checked_sub(carrying_basis)
            .ok_or_else(|| calculation_api_error("Carrying value exceeded the supported range."))?;
        if delta.is_zero() {
            sqlx::query(
                "UPDATE budget_funding_effects SET amount=0 \
                 WHERE workspace_id=$1 AND period_start=$2 AND wallet_id=$3 \
                   AND source_kind='valuation' AND rate_observation_id=$4",
            )
            .bind(workspace_id)
            .bind(target)
            .bind(wallet.wallet_id)
            .bind(current.id)
            .execute(&mut *tx)
            .await
            .map_err(ApiError::database)?;
            continue;
        }
        sqlx::query(
            "INSERT INTO budget_funding_effects \
             (id, workspace_id, period_start, source_kind, wallet_id, rate_observation_id, amount, currency) \
             VALUES ($1, $2, $3, 'valuation', $4, $5, $6, $7) \
             ON CONFLICT (workspace_id, period_start, wallet_id, rate_observation_id) \
             WHERE wallet_id IS NOT NULL DO UPDATE SET amount=EXCLUDED.amount",
        )
        .bind(Uuid::new_v4())
        .bind(workspace_id)
        .bind(target)
        .bind(wallet.wallet_id)
        .bind(current.id)
        .bind(delta)
        .bind(currency)
        .execute(&mut *tx)
        .await
        .map_err(ApiError::database)?;
    }
    refresh_open_correction_targets(&mut tx, workspace_id, target).await?;
    let revision = period_revision_in(&mut tx, workspace_id, next_month(target)?).await?;
    tx.commit().await.map_err(ApiError::database)?;
    Ok(Some((rate_cache, Some(revision))))
}

async fn save_assignment(
    State(state): State<AppState>,
    path: Result<Path<(String, Uuid)>, PathRejection>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<impl IntoResponse, ApiError> {
    let Path((period, category_id)) = path.map_err(|_| ApiError::invalid_request_parameter())?;
    let period = parse_period(&period)?;
    let key = idempotency_key(&headers)?;
    let generation = installation_generation(&headers)?;
    retry_transaction(|| {
        save_assignment_attempt(
            state.clone(),
            generation,
            period,
            category_id,
            key,
            body.clone(),
        )
    })
    .await
}

async fn save_assignment_attempt(
    state: AppState,
    generation: Uuid,
    period: NaiveDate,
    category_id: Uuid,
    key: Uuid,
    body: Bytes,
) -> Result<impl IntoResponse, ApiError> {
    let route = format!(
        "/api/budgets/{}/assignments/{category_id}",
        period.format("%Y-%m")
    );
    let hash = Sha256::digest(&body).to_vec();
    let mut tx = state.pool.begin().await.map_err(ApiError::database)?;
    lock_workspace(&mut tx, state.workspace_id, generation).await?;
    if let Some(receipt) =
        find_receipt(&mut tx, state.workspace_id, key, "PUT", &route, &hash).await?
    {
        let period_revision =
            period_revision_in(&mut tx, state.workspace_id, next_month(period)?).await?;
        tx.commit().await.map_err(ApiError::database)?;
        return Ok((
            StatusCode::from_u16(receipt.response_status as u16).unwrap_or(StatusCode::OK),
            Json(MutationReceipt {
                resource_id: receipt.resource_id.ok_or_else(ApiError::not_found)?,
                revision: receipt.resource_revision.unwrap_or(1),
                period_revision: Some(period_revision),
            }),
        ));
    }
    let request: AssignmentRequest = parse_body(&body)?;
    if request.amount.decimal() < Decimal::ZERO {
        return Err(ApiError::invalid("amount", "must_be_nonnegative"));
    }
    let current_period: NaiveDate = sqlx::query_scalar(
        "SELECT date_trunc('month', now() AT TIME ZONE 'America/Santiago')::date",
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(ApiError::database)?;
    if period > current_period {
        return Err(ApiError::invalid("period", "future_period_not_open"));
    }
    let current_revision =
        period_revision_in(&mut tx, state.workspace_id, next_month(period)?).await?;
    if request.expected_period_revision != current_revision {
        return Err(ApiError::stale_revision());
    }
    let closed: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM budget_close_snapshots WHERE workspace_id=$1 AND period_start=$2)",
    )
    .bind(state.workspace_id)
    .bind(period)
    .fetch_one(&mut *tx)
    .await
    .map_err(ApiError::database)?;
    let active_category: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM expense_categories WHERE workspace_id=$1 AND id=$2 AND archived=FALSE)",
    )
    .bind(state.workspace_id)
    .bind(category_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(ApiError::database)?;
    if !active_category {
        return Err(ApiError::invalid("category_id", "unknown_or_archived"));
    }
    let existing = sqlx::query_as::<_, AssignmentRow>(
        "SELECT id, period_start, category_id, amount, revision FROM budget_assignments \
         WHERE workspace_id=$1 AND period_start=$2 AND category_id=$3 FOR UPDATE",
    )
    .bind(state.workspace_id)
    .bind(period)
    .bind(category_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(ApiError::database)?;
    if closed {
        let reason = request.reason.as_deref().unwrap_or("").trim();
        if reason.is_empty() {
            return Err(ApiError::invalid("reason", "required"));
        }
        match &existing {
            Some(current) if request.expected_revision != Some(current.revision) => {
                return Err(ApiError::stale_revision());
            }
            None if request.expected_revision.is_some() => {
                return Err(ApiError::stale_revision());
            }
            _ => {}
        }
        let assignment_id = save_closed_assignment_correction(
            &mut tx,
            state.workspace_id,
            period,
            category_id,
            existing,
            request.amount.decimal(),
            reason,
        )
        .await?;
        let new_period_revision =
            period_revision_in(&mut tx, state.workspace_id, next_month(period)?).await?;
        save_receipt(
            &mut tx,
            state.workspace_id,
            key,
            "PUT",
            &route,
            &hash,
            StatusCode::OK,
            assignment_id.0,
            assignment_id.1,
        )
        .await?;
        tx.commit().await.map_err(ApiError::database)?;
        return Ok((
            StatusCode::OK,
            Json(MutationReceipt {
                resource_id: assignment_id.0,
                revision: assignment_id.1,
                period_revision: Some(new_period_revision),
            }),
        ));
    }
    let (id, revision) = match existing {
        Some(current) => {
            if request.expected_revision != Some(current.revision) {
                return Err(ApiError::stale_revision());
            }
            let next = current.revision + 1;
            sqlx::query(
                "UPDATE budget_assignments SET amount=$4, revision=$5, updated_at=now() \
                 WHERE workspace_id=$1 AND period_start=$2 AND category_id=$3 AND revision=$6",
            )
            .bind(state.workspace_id)
            .bind(period)
            .bind(category_id)
            .bind(request.amount.decimal())
            .bind(next)
            .bind(current.revision)
            .execute(&mut *tx)
            .await
            .map_err(ApiError::database)?;
            (current.id, next)
        }
        None => {
            if request.expected_revision.is_some() {
                return Err(ApiError::stale_revision());
            }
            let id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO budget_assignments (id, workspace_id, period_start, category_id, amount) \
                 VALUES ($1, $2, $3, $4, $5)",
            )
            .bind(id)
            .bind(state.workspace_id)
            .bind(period)
            .bind(category_id)
            .bind(request.amount.decimal())
            .execute(&mut *tx)
            .await
            .map_err(ApiError::database)?;
            (id, 1)
        }
    };
    let new_period_revision =
        period_revision_in(&mut tx, state.workspace_id, next_month(period)?).await?;
    save_receipt(
        &mut tx,
        state.workspace_id,
        key,
        "PUT",
        &route,
        &hash,
        StatusCode::OK,
        id,
        revision,
    )
    .await?;
    tx.commit().await.map_err(ApiError::database)?;
    Ok((
        StatusCode::OK,
        Json(MutationReceipt {
            resource_id: id,
            revision,
            period_revision: Some(new_period_revision),
        }),
    ))
}

async fn save_closed_assignment_correction(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    workspace_id: Uuid,
    closed_period: NaiveDate,
    category_id: Uuid,
    existing: Option<AssignmentRow>,
    new_target: Decimal,
    reason: &str,
) -> Result<(Uuid, i64), ApiError> {
    let snapshot_baseline: Decimal = sqlx::query_scalar(
        "SELECT assigned FROM budget_close_category_snapshots c \
         JOIN budget_close_snapshots s ON s.workspace_id=c.workspace_id AND s.id=c.snapshot_id \
         WHERE c.workspace_id=$1 AND s.period_start=$2 AND c.category_id=$3",
    )
    .bind(workspace_id)
    .bind(closed_period)
    .bind(category_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(ApiError::database)?
    .ok_or_else(ApiError::not_found)?;

    let (assignment_id, previous_revision, snapshot_amount) = match existing {
        Some(row) => (row.id, row.revision, row.amount),
        None => {
            let id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO budget_assignments (id,workspace_id,period_start,category_id,amount) \
                 VALUES ($1,$2,$3,$4,$5)",
            )
            .bind(id)
            .bind(workspace_id)
            .bind(closed_period)
            .bind(category_id)
            .bind(snapshot_baseline)
            .execute(&mut **tx)
            .await
            .map_err(ApiError::database)?;
            (id, 1, snapshot_baseline)
        }
    };
    let _unchanged_snapshot_assignment = snapshot_amount;
    let new_revision = previous_revision.checked_add(1).ok_or_else(|| {
        calculation_api_error("Assignment revision exceeded the supported range.")
    })?;

    let current_period: NaiveDate = sqlx::query_scalar(
        "SELECT date_trunc('month', now() AT TIME ZONE 'America/Santiago')::date",
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(ApiError::database)?;
    let latest_closed: Option<NaiveDate> = sqlx::query_scalar(
        "SELECT MAX(period_start) FROM budget_close_snapshots WHERE workspace_id=$1",
    )
    .bind(workspace_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(ApiError::database)?;
    let correction_period = if latest_closed.is_some_and(|closed| closed >= current_period) {
        next_month(latest_closed.unwrap_or(current_period))?
    } else {
        current_period
    };
    let baseline = sqlx::query_scalar::<_, Decimal>(
        "SELECT assignment_target_amount FROM financial_corrections \
         WHERE workspace_id=$1 AND record_kind='assignment' AND assignment_id=$2 \
           AND assignment_target_amount IS NOT NULL AND correction_period < $3 \
         ORDER BY correction_period DESC, new_revision DESC, created_at DESC, id DESC LIMIT 1",
    )
    .bind(workspace_id)
    .bind(assignment_id)
    .bind(correction_period)
    .fetch_optional(&mut **tx)
    .await
    .map_err(ApiError::database)?
    .unwrap_or(snapshot_baseline);

    let audit_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO financial_corrections \
         (id,workspace_id,record_kind,assignment_id,prior_revision,new_revision,reason,correction_period,assignment_target_amount) \
         VALUES ($1,$2,'assignment',$3,$4,$5,$6,$7,$8)",
    )
    .bind(audit_id)
    .bind(workspace_id)
    .bind(assignment_id)
    .bind(previous_revision)
    .bind(new_revision)
    .bind(reason)
    .bind(correction_period)
    .bind(new_target)
    .execute(&mut **tx)
    .await
    .map_err(ApiError::database)?;
    sqlx::query(
        "UPDATE budget_assignments SET revision=$4, updated_at=now() \
         WHERE workspace_id=$1 AND period_start=$2 AND category_id=$3 AND revision=$5",
    )
    .bind(workspace_id)
    .bind(closed_period)
    .bind(category_id)
    .bind(new_revision)
    .bind(previous_revision)
    .execute(&mut **tx)
    .await
    .map_err(ApiError::database)?;

    let effect_owner: Uuid = sqlx::query_scalar(
        "SELECT id FROM financial_corrections WHERE workspace_id=$1 AND record_kind='assignment' \
           AND assignment_id=$2 AND correction_period=$3 ORDER BY prior_revision, created_at, id LIMIT 1",
    )
    .bind(workspace_id)
    .bind(assignment_id)
    .bind(correction_period)
    .fetch_optional(&mut **tx)
    .await
    .map_err(ApiError::database)?
    .unwrap_or(audit_id);
    let delta = baseline.checked_sub(new_target).ok_or_else(|| {
        calculation_api_error("Assignment correction exceeded the supported range.")
    })?;
    sqlx::query(
        "INSERT INTO budget_funding_effects \
         (id,workspace_id,period_start,source_kind,correction_id,amount,currency) \
         VALUES ($1,$2,$3,'correction',$4,$5,'CLP') \
         ON CONFLICT (workspace_id,period_start,correction_id) WHERE correction_id IS NOT NULL \
         DO UPDATE SET amount=EXCLUDED.amount",
    )
    .bind(Uuid::new_v4())
    .bind(workspace_id)
    .bind(correction_period)
    .bind(effect_owner)
    .bind(delta)
    .execute(&mut **tx)
    .await
    .map_err(ApiError::database)?;
    Ok((assignment_id, new_revision))
}

struct BudgetData {
    openings: Vec<OpeningRow>,
    transaction_effects: Vec<TransactionEffectRow>,
    funding_effects: Vec<FundingEffectRow>,
    activity_effects: Vec<ActivityEffectRow>,
    assignments: Vec<AssignmentRow>,
    categories: Vec<CategoryView>,
    groups: Vec<BudgetGroup>,
    close_snapshots: Vec<CloseSnapshotRow>,
    close_category_snapshots: Vec<CloseCategorySnapshotRow>,
    assignment_targets: HashMap<Uuid, Decimal>,
    earliest_date: Option<NaiveDate>,
    revision: String,
}

async fn period_revision_in(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    workspace_id: Uuid,
    cutoff: NaiveDate,
) -> Result<String, ApiError> {
    let source: String = sqlx::query_scalar(
        "SELECT concat_ws('|', \
         COALESCE((SELECT jsonb_agg(to_jsonb(o) ORDER BY o.id)::text FROM financial_openings o WHERE o.workspace_id=$1 AND o.business_date < $2), '[]'), \
         COALESCE((SELECT jsonb_agg(to_jsonb(t) ORDER BY t.id)::text FROM financial_transactions t WHERE t.workspace_id=$1 AND t.business_date < $2), '[]'), \
         COALESCE((SELECT jsonb_agg(to_jsonb(e) ORDER BY e.id)::text FROM financial_wallet_effects e JOIN financial_transactions t ON t.workspace_id=e.workspace_id AND t.id=e.transaction_id WHERE e.workspace_id=$1 AND t.business_date < $2), '[]'), \
         COALESCE((SELECT jsonb_agg(to_jsonb(a) ORDER BY a.id)::text FROM financial_accounts a WHERE a.workspace_id=$1), '[]'), \
         COALESCE((SELECT jsonb_agg(to_jsonb(w) ORDER BY w.id)::text FROM financial_wallets w WHERE w.workspace_id=$1), '[]'), \
         COALESCE((SELECT jsonb_agg(to_jsonb(b) ORDER BY b.id)::text FROM foreign_liability_bases b WHERE b.workspace_id=$1), '[]'), \
         COALESCE((SELECT jsonb_agg(to_jsonb(c) ORDER BY c.id)::text FROM financial_corrections c WHERE c.workspace_id=$1 AND (c.correction_period IS NULL OR c.correction_period < $2)), '[]'), \
         COALESCE((SELECT jsonb_agg(to_jsonb(f) ORDER BY f.id)::text FROM budget_funding_effects f WHERE f.workspace_id=$1 AND f.period_start < $2), '[]'), \
         COALESCE((SELECT jsonb_agg(to_jsonb(e) ORDER BY e.period_start,e.transaction_id,e.category_id NULLS FIRST)::text FROM budget_activity_effects e WHERE e.workspace_id=$1 AND e.period_start < $2), '[]'), \
         COALESCE((SELECT jsonb_agg(jsonb_build_object('id', r.id, 'source', r.source, 'base_currency', r.base_currency, 'quote_currency', r.quote_currency, 'effective_date', r.effective_date, 'rate', r.rate, 'observed_at', r.observed_at) ORDER BY r.id)::text FROM daily_fx_observations r WHERE r.workspace_id=$1 AND r.effective_date < $2), '[]'), \
         COALESCE((SELECT jsonb_agg(to_jsonb(c) ORDER BY c.id)::text FROM expense_categories c WHERE c.workspace_id=$1), '[]'), \
         COALESCE((SELECT jsonb_agg(to_jsonb(g) ORDER BY g.id)::text FROM budget_category_groups g WHERE g.workspace_id=$1), '[]'), \
         COALESCE((SELECT jsonb_agg(to_jsonb(a) ORDER BY a.id)::text FROM budget_assignments a WHERE a.workspace_id=$1 AND a.period_start < $2), '[]'), \
         COALESCE((SELECT jsonb_agg(to_jsonb(s) ORDER BY s.period_start)::text FROM budget_close_snapshots s WHERE s.workspace_id=$1 AND s.period_start < $2), '[]'), \
         COALESCE((SELECT jsonb_agg(to_jsonb(s) ORDER BY s.snapshot_id, s.category_id)::text FROM budget_close_category_snapshots s JOIN budget_close_snapshots p ON p.workspace_id=s.workspace_id AND p.id=s.snapshot_id WHERE s.workspace_id=$1 AND p.period_start < $2), '[]'))",
    )
    .bind(workspace_id)
    .bind(cutoff)
    .fetch_one(&mut **tx)
    .await
    .map_err(ApiError::database)?;
    Ok(format!("{:x}", Sha256::digest(source.as_bytes())))
}

struct CurrentBudget {
    result: PeriodResult,
    funding: CalculatedValue,
    breakdown: AvailableToBudgetBreakdown,
}

async fn load_budget_data(
    pool: &PgPool,
    workspace_id: Uuid,
    target: NaiveDate,
) -> Result<BudgetData, ApiError> {
    let mut tx = pool.begin().await.map_err(ApiError::database)?;
    sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY")
        .execute(&mut *tx)
        .await
        .map_err(ApiError::database)?;

    let openings = sqlx::query_as::<_, OpeningRow>(
        "SELECT o.business_date, o.amount, w.currency::text AS currency, \
                a.budget_eligible, b.clp_amount AS liability_clp_basis \
         FROM financial_openings o \
         JOIN financial_wallets w ON w.workspace_id=o.workspace_id AND w.id=o.wallet_id \
         JOIN financial_accounts a ON a.workspace_id=w.workspace_id AND a.id=w.account_id \
         LEFT JOIN foreign_liability_bases b ON b.workspace_id=o.workspace_id AND b.opening_id=o.id \
         WHERE o.workspace_id=$1 AND o.business_date <= $2 \
         ORDER BY o.business_date, o.id",
    )
    .bind(workspace_id)
    .bind(last_day_of_month(target)?)
    .fetch_all(&mut *tx)
    .await
    .map_err(ApiError::database)?;

    let transaction_effects = sqlx::query_as::<_, TransactionEffectRow>(
        "SELECT t.id, e.wallet_id, t.transaction_kind, t.business_date, t.category_id, \
                t.confirmed_clp_amount, t.conversion_evidence, \
                w.currency::text AS currency, e.effect_role, e.amount AS effect_amount, \
                e.opening_treatment, a.budget_eligible \
         FROM financial_transactions t \
         JOIN financial_wallet_effects e ON e.workspace_id=t.workspace_id AND e.transaction_id=t.id \
         JOIN financial_wallets w ON w.workspace_id=e.workspace_id AND w.id=e.wallet_id \
         JOIN financial_accounts a ON a.workspace_id=w.workspace_id AND a.id=w.account_id \
         WHERE t.workspace_id=$1 AND t.business_date <= $2 \
         ORDER BY t.business_date, t.id, e.effect_role, e.wallet_id",
    )
    .bind(workspace_id)
    .bind(last_day_of_month(target)?)
    .fetch_all(&mut *tx)
    .await
    .map_err(ApiError::database)?;

    let funding_effects = sqlx::query_as::<_, FundingEffectRow>(
        "SELECT period_start, source_kind, amount, missing_reason \
         FROM budget_funding_effects WHERE workspace_id=$1 AND period_start <= $2 \
           AND source_kind IN ('correction', 'valuation') \
         ORDER BY period_start, id",
    )
    .bind(workspace_id)
    .bind(target)
    .fetch_all(&mut *tx)
    .await
    .map_err(ApiError::database)?;

    let activity_effects = sqlx::query_as::<_, ActivityEffectRow>(
        "SELECT period_start, transaction_id, category_id, baseline_amount, target_amount, \
                baseline_kind, baseline_currency::text AS baseline_currency, \
                baseline_original_amount, baseline_rate_observation_id, baseline_business_date, \
                target_kind, target_currency::text AS target_currency, target_original_amount, \
                target_rate_observation_id, target_business_date \
         FROM budget_activity_effects WHERE workspace_id=$1 AND period_start <= $2 \
         ORDER BY period_start, transaction_id, category_id NULLS FIRST",
    )
    .bind(workspace_id)
    .bind(target)
    .fetch_all(&mut *tx)
    .await
    .map_err(ApiError::database)?;

    let assignments = sqlx::query_as::<_, AssignmentRow>(
        "SELECT id, period_start, category_id, amount, revision FROM budget_assignments \
         WHERE workspace_id=$1 AND period_start <= $2 ORDER BY period_start, category_id",
    )
    .bind(workspace_id)
    .bind(target)
    .fetch_all(&mut *tx)
    .await
    .map_err(ApiError::database)?;

    let categories = sqlx::query_as::<_, CategoryView>(
        "SELECT id, slug, name, sort_order, archived, group_id, revision FROM expense_categories \
         WHERE workspace_id=$1 ORDER BY sort_order, id",
    )
    .bind(workspace_id)
    .fetch_all(&mut *tx)
    .await
    .map_err(ApiError::database)?;

    let groups = sqlx::query_as::<_, BudgetGroup>("SELECT id,name,sort_order,archived,revision FROM budget_category_groups WHERE workspace_id=$1 ORDER BY sort_order,id")
        .bind(workspace_id).fetch_all(&mut *tx).await.map_err(ApiError::database)?;
    let close_snapshots = sqlx::query_as::<_, CloseSnapshotRow>(
        "SELECT id, period_start, funding, ready_to_assign, activity, \
                uncategorized_activity, closed_at, available_to_budget_breakdown \
         FROM budget_close_snapshots WHERE workspace_id=$1 AND period_start <= $2 \
         ORDER BY period_start",
    )
    .bind(workspace_id)
    .bind(target)
    .fetch_all(&mut *tx)
    .await
    .map_err(ApiError::database)?;

    let close_category_snapshots = sqlx::query_as::<_, CloseCategorySnapshotRow>(
        "SELECT c.snapshot_id, c.category_id, c.category_name, c.category_sort_order, \
                c.assigned, c.activity, c.remaining, c.disposition \
         FROM budget_close_category_snapshots c \
         JOIN budget_close_snapshots s ON s.workspace_id=c.workspace_id AND s.id=c.snapshot_id \
         WHERE c.workspace_id=$1 AND s.period_start <= $2 \
         ORDER BY s.period_start, c.category_sort_order NULLS LAST, c.category_id",
    )
    .bind(workspace_id)
    .bind(target)
    .fetch_all(&mut *tx)
    .await
    .map_err(ApiError::database)?;

    let assignment_targets = sqlx::query_as::<_, AssignmentTargetRow>(
        "SELECT DISTINCT ON (assignment_id) assignment_id, assignment_target_amount \
         FROM financial_corrections WHERE workspace_id=$1 AND record_kind='assignment' \
           AND assignment_target_amount IS NOT NULL \
         ORDER BY assignment_id, new_revision DESC, created_at DESC, id DESC",
    )
    .bind(workspace_id)
    .fetch_all(&mut *tx)
    .await
    .map_err(ApiError::database)?
    .into_iter()
    .map(|row| (row.assignment_id, row.assignment_target_amount))
    .collect();

    let revision = period_revision_in(&mut tx, workspace_id, next_month(target)?).await?;

    tx.commit().await.map_err(ApiError::database)?;

    let earliest_date = openings
        .iter()
        .map(|opening| opening.business_date)
        .chain(
            transaction_effects
                .iter()
                .map(|effect| effect.business_date),
        )
        .chain(funding_effects.iter().map(|effect| effect.period_start))
        .chain(assignments.iter().map(|assignment| assignment.period_start))
        .min();
    Ok(BudgetData {
        openings,
        transaction_effects,
        funding_effects,
        activity_effects,
        assignments,
        categories,
        groups,
        close_snapshots,
        close_category_snapshots,
        assignment_targets,
        earliest_date,
        revision,
    })
}

async fn calculate_open_period(
    pool: &PgPool,
    workspace_id: Uuid,
    target: NaiveDate,
    data: &BudgetData,
    mut rates: HashMap<(String, NaiveDate), Option<FxObservation>>,
) -> Result<CurrentBudget, ApiError> {
    let latest_close = data.close_snapshots.last();
    let first = latest_close
        .map(|snapshot| next_month(snapshot.period_start))
        .transpose()?
        .unwrap_or_else(|| first_of_month(data.earliest_date.unwrap_or(target)));
    let mut month = first;
    let mut prior_unassigned = latest_close
        .map(|snapshot| snapshot.ready_to_assign)
        .unwrap_or(Decimal::ZERO);
    let mut prior_category_remaining = BTreeMap::new();
    let mut released_category_remainders = BTreeMap::new();
    if let Some(snapshot) = latest_close {
        for row in data
            .close_category_snapshots
            .iter()
            .filter(|row| row.snapshot_id == snapshot.id)
        {
            if row.disposition == "carry" {
                prior_category_remaining.insert(row.category_id.to_string(), row.remaining);
            } else {
                released_category_remainders.insert(row.category_id.to_string(), row.remaining);
            }
        }
    }
    let mut current_result = None;
    let mut inherited_missing_funding = Vec::new();
    let mut inherited_category_unresolved =
        BTreeMap::<String, Vec<crate::budget_calculations::UnresolvedActivity>>::new();

    while month <= target {
        let period = BudgetPeriod::new(month.year(), month.month())
            .map_err(|_| ApiError::invalid("period", "invalid_period"))?;
        let next_month = next_month(month)?;
        let mut funding_events = Vec::new();
        for opening in &data.openings {
            if !opening.budget_eligible || !period.contains(opening.business_date) {
                continue;
            }
            let value = if opening.currency == "CLP" {
                ValueState::known(opening.amount).map_err(calculation_error)?
            } else if opening.amount < Decimal::ZERO {
                foreign_liability_opening_value(opening.liability_clp_basis)
                    .map_err(calculation_error)?
            } else if opening.amount.is_zero() {
                ValueState::known(Decimal::ZERO).map_err(calculation_error)?
            } else {
                match lookup_rate(
                    pool,
                    workspace_id,
                    &opening.currency,
                    opening.business_date,
                    &mut rates,
                )
                .await?
                {
                    Some(rate) => foreign_asset_opening_value(opening.amount, Some(rate.rate))
                        .map_err(calculation_error)?,
                    None => foreign_asset_opening_value(opening.amount, None)
                        .map_err(calculation_error)?,
                }
            };
            funding_events.push(FundingEvent {
                date: opening.business_date,
                kind: FundingKind::Opening,
                eligible: true,
                value_clp: value,
            });
        }

        for stored in &data.funding_effects {
            if first_of_month(stored.period_start) != month {
                continue;
            }
            let kind = if stored.source_kind == "valuation" {
                FundingKind::ForeignAssetRevaluation
            } else {
                FundingKind::OtherCorrection
            };
            funding_events.push(FundingEvent {
                date: stored.period_start,
                kind,
                eligible: true,
                value_clp: match stored.amount {
                    Some(amount) => ValueState::known(amount).map_err(calculation_error)?,
                    None => ValueState::Missing(match stored.missing_reason.as_deref() {
                        Some("transfer_source_valuation") => MissingValue::TransferSourceValuation,
                        Some("transfer_destination_valuation") => {
                            MissingValue::TransferDestinationValuation
                        }
                        Some("foreign_asset_rate") => MissingValue::ForeignAssetRate,
                        _ => MissingValue::TransactionClpValue,
                    }),
                },
            });
        }

        let mut grouped: BTreeMap<Uuid, Vec<&TransactionEffectRow>> = BTreeMap::new();
        for effect in &data.transaction_effects {
            if period.contains(effect.business_date) {
                grouped.entry(effect.id).or_default().push(effect);
            }
        }
        let mut activities = Vec::new();
        for rows in grouped.values() {
            let transaction = rows[0];
            match transaction.transaction_kind.as_str() {
                "transfer" => {
                    let event = transfer_event(rows, pool, workspace_id, &mut rates).await?;
                    if let Some(event) = event {
                        funding_events.push(event);
                    }
                }
                "income" | "expense" | "refund" => {
                    if let Some(row) = rows.first() {
                        let amount_clp = if let Some(confirmed) = row.confirmed_clp_amount {
                            Some(confirmed)
                        } else if row.currency == "CLP" {
                            Some(row.effect_amount.abs())
                        } else if row.transaction_kind == "expense" {
                            None
                        } else {
                            match lookup_rate(
                                pool,
                                workspace_id,
                                &row.currency,
                                row.business_date,
                                &mut rates,
                            )
                            .await?
                            {
                                Some(rate) => Some(
                                    value_foreign_asset(
                                        MoneyAmount::new(row.effect_amount.abs()).map_err(
                                            |_| ApiError::invalid("amount", "out_of_range"),
                                        )?,
                                        &rate,
                                    )
                                    .map_err(|_| ApiError::invalid("amount", "out_of_range"))?
                                    .decimal(),
                                ),
                                None => None,
                            }
                        };
                        activities.push(TransactionActivity {
                            date: row.business_date,
                            kind: activity_kind(&row.transaction_kind)?,
                            eligible: row.budget_eligible
                                && row.opening_treatment.as_deref() != Some("preserve_balance"),
                            category_id: row.category_id.map(|id| id.to_string()),
                            amount_clp,
                            original_currency: Some(row.currency.trim().to_owned()),
                            original_amount: Some(row.effect_amount.abs()),
                        });
                    }
                }
                _ => return Err(ApiError::invalid("kind", "unsupported_transaction_kind")),
            }
        }

        let assignments_clp = data
            .assignments
            .iter()
            .filter(|assignment| assignment.period_start == month)
            .map(|assignment| (assignment.category_id.to_string(), assignment.amount))
            .collect::<BTreeMap<_, _>>();
        let input = PeriodInput {
            period,
            prior_unassigned_clp: prior_unassigned,
            prior_category_remaining_clp: prior_category_remaining.clone(),
            released_category_remainders_clp: std::mem::take(&mut released_category_remainders),
            funding_events,
            activities,
            assignments_clp,
        };
        let breakdown = open_breakdown(&input, &inherited_missing_funding)?;
        let period_funding = calculate_funding(&input.funding_events, &input.activities, period)?;
        let mut result = calculate_period(&input).map_err(calculation_error)?;
        apply_activity_correction_effects(&mut result, &data.activity_effects, month)?;
        if month < target {
            match &result.ready_to_assign_clp {
                CalculatedValue::Complete(value) => prior_unassigned = *value,
                CalculatedValue::Unavailable(reasons) => {
                    for reason in reasons {
                        if !inherited_missing_funding.contains(reason) {
                            inherited_missing_funding.push(*reason);
                        }
                    }
                    prior_unassigned = Decimal::ZERO;
                }
                CalculatedValue::Incomplete { .. } => prior_unassigned = Decimal::ZERO,
            }
            prior_category_remaining = std::mem::take(&mut result.category_remaining_clp)
                .into_iter()
                .filter_map(|(id, value)| match value {
                    CalculatedValue::Complete(amount) => Some((id, amount)),
                    CalculatedValue::Incomplete {
                        known_subtotal,
                        unresolved,
                    } => {
                        inherited_category_unresolved
                            .entry(id.clone())
                            .or_default()
                            .extend(unresolved);
                        Some((id, known_subtotal))
                    }
                    CalculatedValue::Unavailable(_) => None,
                })
                .collect();
            month = next_month;
            continue;
        }

        if month == target {
            if !inherited_missing_funding.is_empty() {
                let mut reasons = inherited_missing_funding;
                if let CalculatedValue::Unavailable(current) = &result.ready_to_assign_clp {
                    for reason in current {
                        if !reasons.contains(reason) {
                            reasons.push(*reason);
                        }
                    }
                }
                result.ready_to_assign_clp = CalculatedValue::Unavailable(reasons);
                result.can_close = false;
            }
            for (id, previous_unresolved) in inherited_category_unresolved {
                let current = result
                    .category_remaining_clp
                    .remove(&id)
                    .unwrap_or(CalculatedValue::Complete(Decimal::ZERO));
                let replacement = match current {
                    CalculatedValue::Complete(known_subtotal) => CalculatedValue::Incomplete {
                        known_subtotal,
                        unresolved: previous_unresolved,
                    },
                    CalculatedValue::Incomplete {
                        known_subtotal,
                        mut unresolved,
                    } => {
                        unresolved.extend(previous_unresolved);
                        CalculatedValue::Incomplete {
                            known_subtotal,
                            unresolved,
                        }
                    }
                    CalculatedValue::Unavailable(reasons) => CalculatedValue::Unavailable(reasons),
                };
                result.category_remaining_clp.insert(id, replacement);
                result.can_close = false;
            }
            current_result = Some(CurrentBudget {
                result,
                funding: period_funding,
                breakdown,
            });
            break;
        }
        month = next_month;
    }
    current_result.ok_or_else(|| ApiError::invalid("period", "invalid_period"))
}

fn apply_activity_correction_effects(
    result: &mut PeriodResult,
    effects: &[ActivityEffectRow],
    month: NaiveDate,
) -> Result<(), ApiError> {
    for effect in effects.iter().filter(|effect| effect.period_start == month) {
        let known_delta = effect
            .target_amount
            .unwrap_or(Decimal::ZERO)
            .checked_sub(effect.baseline_amount.unwrap_or(Decimal::ZERO))
            .ok_or_else(|| {
                calculation_api_error("Activity correction exceeded the supported range.")
            })?;
        let unresolved_target = effect.target_amount.is_none().then(|| UnresolvedActivity {
            kind: if effect.target_kind == "refund" {
                ActivityKind::Refund
            } else {
                ActivityKind::Expense
            },
            category_id: effect.category_id.map(|id| id.to_string()),
            currency: Some(effect.target_currency.trim().to_owned()),
            amount: Some(effect.target_original_amount),
        });
        let unresolved_baseline = effect
            .baseline_amount
            .is_none()
            .then(|| UnresolvedActivity {
                kind: if effect.baseline_kind == "refund" {
                    ActivityKind::Refund
                } else {
                    ActivityKind::Expense
                },
                category_id: effect.category_id.map(|id| id.to_string()),
                currency: Some(effect.baseline_currency.trim().to_owned()),
                amount: Some(effect.baseline_original_amount),
            });
        let unresolved = unresolved_target
            .into_iter()
            .chain(unresolved_baseline)
            .collect::<Vec<_>>();
        if let Some(category_id) = effect.category_id {
            let key = category_id.to_string();
            let current_activity = result
                .category_activity_clp
                .remove(&key)
                .unwrap_or(CalculatedValue::Complete(Decimal::ZERO));
            result.category_activity_clp.insert(
                key.clone(),
                add_activity_delta(current_activity, known_delta, unresolved.clone())?,
            );
            let remaining_delta = Decimal::ZERO.checked_sub(known_delta).ok_or_else(|| {
                calculation_api_error("Activity correction exceeded the supported range.")
            })?;
            let current_remaining = result
                .category_remaining_clp
                .remove(&key)
                .unwrap_or(CalculatedValue::Complete(Decimal::ZERO));
            result.category_remaining_clp.insert(
                key,
                add_activity_delta(current_remaining, remaining_delta, unresolved)?,
            );
        } else {
            result.uncategorized_activity_clp = add_activity_delta(
                result.uncategorized_activity_clp.clone(),
                known_delta,
                unresolved,
            )?;
        }
        if effect.target_amount.is_none() || effect.baseline_amount.is_none() {
            result.can_close = false;
        }
    }
    Ok(())
}

fn add_activity_delta(
    current: CalculatedValue,
    delta: Decimal,
    unresolved: Vec<UnresolvedActivity>,
) -> Result<CalculatedValue, ApiError> {
    let (known, mut unresolved_values) = match current {
        CalculatedValue::Complete(amount) => (amount, Vec::new()),
        CalculatedValue::Incomplete {
            known_subtotal,
            unresolved,
        } => (known_subtotal, unresolved),
        CalculatedValue::Unavailable(reasons) => {
            return Ok(CalculatedValue::Unavailable(reasons));
        }
    };
    let known = known.checked_add(delta).ok_or_else(|| {
        calculation_api_error("Activity correction exceeded the supported range.")
    })?;
    MoneyAmount::new(known).map_err(stored_money_error)?;
    unresolved_values.extend(unresolved);
    if unresolved_values.is_empty() {
        Ok(CalculatedValue::Complete(known))
    } else {
        Ok(CalculatedValue::Incomplete {
            known_subtotal: known,
            unresolved: unresolved_values,
        })
    }
}

async fn lookup_rate(
    pool: &PgPool,
    workspace_id: Uuid,
    currency: &str,
    date: NaiveDate,
    cache: &mut HashMap<(String, NaiveDate), Option<FxObservation>>,
) -> Result<Option<FxObservation>, ApiError> {
    let key = (currency.to_owned(), date);
    if let Some(rate) = cache.get(&key) {
        return Ok(rate.clone());
    }
    let rate = if let Some(stored) = exact_stored_rate(pool, workspace_id, currency, date).await? {
        Some(stored)
    } else if let Ok(provider) = FrankfurterProvider::new() {
        provider_result_to_rate(
            get_observation(pool, workspace_id, currency, "CLP", date, &provider).await,
        )?
    } else {
        None
    };
    cache.insert(key, rate.clone());
    Ok(rate)
}

async fn exact_stored_rate(
    pool: &PgPool,
    workspace_id: Uuid,
    currency: &str,
    date: NaiveDate,
) -> Result<Option<FxObservation>, ApiError> {
    let row = sqlx::query_as::<_, StoredRateRow>(
        "SELECT id, workspace_id, source, base_currency::text AS base_currency, \
                quote_currency::text AS quote_currency, effective_date, rate \
         FROM daily_fx_observations WHERE workspace_id=$1 AND source=$2 \
           AND base_currency=$3 AND quote_currency='CLP' AND effective_date=$4 \
         ORDER BY observed_at DESC, id DESC LIMIT 1",
    )
    .bind(workspace_id)
    .bind(FRANKFURTER_SOURCE)
    .bind(currency)
    .bind(date)
    .fetch_optional(pool)
    .await
    .map_err(ApiError::database)?;
    Ok(row.map(|row| FxObservation {
        id: row.id,
        workspace_id: row.workspace_id,
        source: row.source,
        base_currency: row.base_currency.trim().to_owned(),
        quote_currency: row.quote_currency.trim().to_owned(),
        effective_date: row.effective_date,
        rate: row.rate,
        stale: false,
        provider_unavailable: false,
    }))
}

fn provider_result_to_rate(
    result: Result<FxObservation, FxError>,
) -> Result<Option<FxObservation>, ApiError> {
    match result {
        Ok(rate) => Ok(Some(rate)),
        Err(
            FxError::RateUnavailable
            | FxError::ProviderUnavailable
            | FxError::InvalidProviderResponse,
        ) => Ok(None),
        Err(FxError::Database(error)) => Err(ApiError::database(error)),
        Err(FxError::InvalidCurrency | FxError::AmountOutOfRange | FxError::ArithmeticOverflow) => {
            Err(ApiError::invalid("currency", "invalid_or_out_of_range"))
        }
    }
}

async fn latest_stored_rate(
    pool: &PgPool,
    workspace_id: Uuid,
    currency: &str,
    date: NaiveDate,
) -> Result<Option<FxObservation>, sqlx::Error> {
    let row = sqlx::query_as::<_, StoredRateRow>(
        "SELECT id, workspace_id, source, base_currency::text AS base_currency, \
                quote_currency::text AS quote_currency, effective_date, rate \
         FROM daily_fx_observations \
         WHERE workspace_id=$1 AND source=$2 AND base_currency=$3 \
           AND quote_currency='CLP' AND effective_date <= $4 \
         ORDER BY effective_date DESC, observed_at DESC, id DESC LIMIT 1",
    )
    .bind(workspace_id)
    .bind(FRANKFURTER_SOURCE)
    .bind(currency)
    .bind(date)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|row| FxObservation {
        id: row.id,
        workspace_id: row.workspace_id,
        source: row.source,
        base_currency: row.base_currency.trim().to_owned(),
        quote_currency: row.quote_currency.trim().to_owned(),
        effective_date: row.effective_date,
        rate: row.rate,
        stale: true,
        provider_unavailable: false,
    }))
}

async fn latest_stored_rate_in(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    workspace_id: Uuid,
    currency: &str,
    date: NaiveDate,
) -> Result<Option<FxObservation>, ApiError> {
    let row = sqlx::query_as::<_, StoredRateRow>(
        "SELECT id, workspace_id, source, base_currency::text AS base_currency, \
                quote_currency::text AS quote_currency, effective_date, rate \
         FROM daily_fx_observations WHERE workspace_id=$1 AND source=$2 \
           AND base_currency=$3 AND quote_currency='CLP' AND effective_date <= $4 \
         ORDER BY effective_date DESC, observed_at DESC, id DESC LIMIT 1",
    )
    .bind(workspace_id)
    .bind(FRANKFURTER_SOURCE)
    .bind(currency)
    .bind(date)
    .fetch_optional(&mut **tx)
    .await
    .map_err(ApiError::database)?;
    Ok(row.map(|row| FxObservation {
        id: row.id,
        workspace_id: row.workspace_id,
        source: row.source,
        base_currency: row.base_currency.trim().to_owned(),
        quote_currency: row.quote_currency.trim().to_owned(),
        effective_date: row.effective_date,
        rate: row.rate,
        stale: row.effective_date < date,
        provider_unavailable: false,
    }))
}

fn signed_value(signed_amount: Decimal, magnitude: Decimal) -> Result<Decimal, ApiError> {
    if signed_amount.is_sign_negative() {
        Decimal::ZERO
            .checked_sub(magnitude)
            .ok_or_else(|| calculation_api_error("Carrying value exceeded the supported range."))
    } else {
        Ok(magnitude)
    }
}

fn valued_signed_amount(signed_amount: Decimal, rate: &FxObservation) -> Result<Decimal, ApiError> {
    let amount = MoneyAmount::new(signed_amount.abs())
        .map_err(|_| ApiError::invalid("amount", "out_of_range"))?;
    let value = value_foreign_asset(amount, rate)
        .map_err(|_| ApiError::invalid("amount", "out_of_range"))?
        .decimal();
    signed_value(signed_amount, value)
}

fn calculation_api_error(message: &'static str) -> ApiError {
    ApiError::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        "calculation_overflow",
        message,
    )
}

async fn carrying_flows_before_in(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    workspace_id: Uuid,
    wallet_id: Uuid,
    date: NaiveDate,
    transaction_id: Uuid,
) -> Result<Vec<CarryingFlowRow>, ApiError> {
    sqlx::query_as::<_, CarryingFlowRow>(
        "SELECT business_date,transaction_id,amount,record_kind,transaction_kind,effect_role, \
                confirmed_clp_amount,conversion_evidence,liability_clp_basis FROM ( \
            SELECT o.business_date,NULL::uuid AS transaction_id,o.amount,'opening'::text AS record_kind, \
                   NULL::text AS transaction_kind,NULL::text AS effect_role,NULL::numeric AS confirmed_clp_amount, \
                   NULL::jsonb AS conversion_evidence,b.clp_amount AS liability_clp_basis \
            FROM financial_openings o \
            LEFT JOIN foreign_liability_bases b ON b.workspace_id=o.workspace_id AND b.opening_id=o.id \
            WHERE o.workspace_id=$1 AND o.wallet_id=$2 AND o.business_date <= $3 \
            UNION ALL \
            SELECT t.business_date,t.id,e.amount,'transaction'::text,t.transaction_kind,e.effect_role, \
                   t.confirmed_clp_amount,t.conversion_evidence,NULL::numeric \
            FROM financial_wallet_effects e \
            JOIN financial_transactions t ON t.workspace_id=e.workspace_id AND t.id=e.transaction_id \
            WHERE e.workspace_id=$1 AND e.wallet_id=$2 AND e.opening_treatment IS DISTINCT FROM 'preserve_balance' \
              AND t.id <> $4 AND (t.business_date < $3 OR (t.business_date=$3 AND t.id < $4)) \
        ) flows ORDER BY business_date,record_kind,transaction_id,effect_role",
    )
    .bind(workspace_id)
    .bind(wallet_id)
    .bind(date)
    .bind(transaction_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(ApiError::database)
}

async fn carrying_source_value_in(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    workspace_id: Uuid,
    wallet_id: Uuid,
    currency: &str,
    date: NaiveDate,
    transaction_id: Uuid,
    source_amount: Decimal,
) -> Result<(Option<Decimal>, Vec<Uuid>), ApiError> {
    if currency == "CLP" {
        return Ok((Some(source_amount.abs()), Vec::new()));
    }
    let flows = carrying_flows_before_in(tx, workspace_id, wallet_id, date, transaction_id).await?;
    let mut wallet_balance = Decimal::ZERO;
    let mut carrying_value = Decimal::ZERO;
    let mut liability_units = Decimal::ZERO;
    let mut liability_basis = Decimal::ZERO;
    let mut unbased_liability = false;
    let mut observation_ids = Vec::new();
    for flow in flows {
        if flow.amount.is_zero() {
            continue;
        }
        let signed_clp = if flow.record_kind == "opening" {
            let (value, rate_id) = opening_clp_value(
                tx,
                workspace_id,
                currency,
                flow.business_date,
                flow.amount,
                flow.liability_clp_basis,
            )
            .await?;
            observation_ids.extend(rate_id);
            let Some(value) = value else {
                return Ok((None, observation_ids));
            };
            if flow.amount < Decimal::ZERO {
                liability_units = liability_units.checked_add(flow.amount).ok_or_else(|| {
                    calculation_api_error("Wallet carrying balance exceeded the supported range.")
                })?;
                liability_basis = liability_basis.checked_add(value).ok_or_else(|| {
                    calculation_api_error("Wallet carrying value exceeded the supported range.")
                })?;
            }
            value
        } else if flow.transaction_kind.as_deref() == Some("transfer")
            && flow.effect_role.as_deref() == Some("source")
        {
            if wallet_balance.is_zero() {
                return Ok((None, observation_ids));
            }
            carrying_value
                .checked_mul(flow.amount)
                .and_then(|amount| amount.checked_div(wallet_balance))
                .ok_or_else(|| {
                    calculation_api_error("Transfer carrying value exceeded the supported range.")
                })?
        } else if flow.transaction_kind.as_deref() == Some("transfer")
            && flow.effect_role.as_deref() == Some("destination")
        {
            if let Some(received) = evidence_clp_amount(&flow.conversion_evidence)? {
                signed_value(flow.amount, received)?
            } else {
                let Some(rate) =
                    latest_stored_rate_in(tx, workspace_id, currency, flow.business_date).await?
                else {
                    return Ok((None, observation_ids));
                };
                observation_ids.push(rate.id);
                valued_signed_amount(flow.amount, &rate)?
            }
        } else if let Some(confirmed) = flow.confirmed_clp_amount {
            signed_value(flow.amount, confirmed.abs())?
        } else if currency == "CLP" {
            flow.amount
        } else {
            let Some(rate) =
                latest_stored_rate_in(tx, workspace_id, currency, flow.business_date).await?
            else {
                return Ok((None, observation_ids));
            };
            observation_ids.push(rate.id);
            valued_signed_amount(flow.amount, &rate)?
        };
        let next_balance = wallet_balance.checked_add(flow.amount).ok_or_else(|| {
            calculation_api_error("Wallet carrying balance exceeded the supported range.")
        })?;
        if flow.amount > Decimal::ZERO && liability_units < Decimal::ZERO {
            let repayment = flow.amount.min(liability_units.abs());
            let remaining_units = liability_units.checked_add(repayment).ok_or_else(|| {
                calculation_api_error("Wallet carrying balance exceeded the supported range.")
            })?;
            liability_basis = liability_basis
                .checked_mul(remaining_units.abs())
                .and_then(|value| value.checked_div(liability_units.abs()))
                .ok_or_else(|| {
                    calculation_api_error("Wallet carrying value exceeded the supported range.")
                })?;
            liability_units = remaining_units;
        } else if next_balance < liability_units {
            // The source facts increased debt beyond the explicitly supplied
            // liability basis, so its CLP carrying value is not established.
            unbased_liability = true;
        }
        wallet_balance = next_balance;
        carrying_value = carrying_value.checked_add(signed_clp).ok_or_else(|| {
            calculation_api_error("Wallet carrying value exceeded the supported range.")
        })?;
    }
    if wallet_balance.is_zero() || unbased_liability {
        return Ok((None, observation_ids));
    }
    if source_amount.abs() > wallet_balance.abs() {
        return Ok((None, observation_ids));
    }
    if wallet_balance < Decimal::ZERO {
        if liability_units >= Decimal::ZERO || liability_basis >= Decimal::ZERO {
            return Ok((None, observation_ids));
        }
        let source_clp = liability_basis
            .abs()
            .checked_mul(source_amount.abs())
            .and_then(|value| value.checked_div(liability_units.abs()))
            .ok_or_else(|| {
                calculation_api_error("Transfer carrying value exceeded the supported range.")
            })?;
        MoneyAmount::new(source_clp).map_err(stored_money_error)?;
        return Ok((Some(source_clp), observation_ids));
    }
    // A foreign asset is carried at the latest available value immediately
    // before the transfer. Keep negative opening balances at their separately
    // supplied liability basis while revaluing the foreign asset units.
    if wallet_balance <= Decimal::ZERO || unbased_liability {
        return Ok((None, observation_ids));
    }
    let Some(rate) = latest_stored_rate_in(tx, workspace_id, currency, date).await? else {
        return Ok((None, observation_ids));
    };
    observation_ids.push(rate.id);
    let current_asset_value = value_foreign_asset(
        MoneyAmount::new(wallet_balance).map_err(stored_money_error)?,
        &FxObservation {
            id: rate.id,
            workspace_id: rate.workspace_id,
            source: rate.source,
            base_currency: rate.base_currency,
            quote_currency: rate.quote_currency,
            effective_date: rate.effective_date,
            rate: rate.rate,
            stale: rate.effective_date < date,
            provider_unavailable: false,
        },
    )
    .map_err(|_| ApiError::invalid("amount", "out_of_range"))?
    .decimal();
    let carrying_value = current_asset_value;
    let source_clp = carrying_value
        .checked_mul(source_amount.abs())
        .and_then(|value| value.checked_div(wallet_balance.abs()))
        .ok_or_else(|| {
            calculation_api_error("Transfer carrying value exceeded the supported range.")
        })?;
    MoneyAmount::new(source_clp).map_err(stored_money_error)?;
    Ok((Some(source_clp), observation_ids))
}

async fn carrying_source_value(
    pool: &PgPool,
    workspace_id: Uuid,
    wallet_id: Uuid,
    currency: &str,
    date: NaiveDate,
    transaction_id: Uuid,
    source_amount: Decimal,
) -> Result<Option<Decimal>, ApiError> {
    if currency == "CLP" {
        return Ok(Some(source_amount.abs()));
    }
    let mut tx = pool.begin().await.map_err(ApiError::database)?;
    sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY")
        .execute(&mut *tx)
        .await
        .map_err(ApiError::database)?;
    let (value, _) = carrying_source_value_in(
        &mut tx,
        workspace_id,
        wallet_id,
        currency,
        date,
        transaction_id,
        source_amount,
    )
    .await?;
    tx.commit().await.map_err(ApiError::database)?;
    Ok(value)
}

async fn transfer_event(
    rows: &[&TransactionEffectRow],
    pool: &PgPool,
    workspace_id: Uuid,
    rates: &mut HashMap<(String, NaiveDate), Option<FxObservation>>,
) -> Result<Option<FundingEvent>, ApiError> {
    let source = rows.iter().find(|row| row.effect_role == "source");
    let destination = rows.iter().find(|row| row.effect_role == "destination");
    let (Some(source), Some(destination)) = (source, destination) else {
        return Ok(Some(FundingEvent {
            date: rows[0].business_date,
            kind: FundingKind::Transfer,
            eligible: true,
            value_clp: ValueState::Missing(MissingValue::TransferSourceValuation),
        }));
    };
    let source_value = if !source.budget_eligible {
        ValueState::Known(Decimal::ZERO)
    } else if source.currency == "CLP" {
        ValueState::Known(source.effect_amount.abs())
    } else {
        // Populate the observation cache for the transfer date before the
        // carrying replay reads the selected stored observation.
        let _ = lookup_rate(
            pool,
            workspace_id,
            &source.currency,
            source.business_date,
            rates,
        )
        .await?;
        match carrying_source_value(
            pool,
            workspace_id,
            source.wallet_id,
            &source.currency,
            source.business_date,
            source.id,
            source.effect_amount,
        )
        .await?
        {
            Some(value) => ValueState::Known(value),
            None => ValueState::Missing(MissingValue::TransferSourceValuation),
        }
    };
    let received_clp = evidence_clp_amount(&destination.conversion_evidence)?;
    let destination_value = if !destination.budget_eligible {
        ValueState::Known(Decimal::ZERO)
    } else if let Some(amount) = received_clp {
        ValueState::Known(amount)
    } else if destination.currency == "CLP" {
        ValueState::Known(destination.effect_amount.abs())
    } else {
        match lookup_rate(
            pool,
            workspace_id,
            &destination.currency,
            destination.business_date,
            rates,
        )
        .await?
        {
            Some(rate) => ValueState::Known(
                value_foreign_asset(
                    MoneyAmount::new(destination.effect_amount.abs())
                        .map_err(|_| ApiError::invalid("amount", "out_of_range"))?,
                    &rate,
                )
                .map_err(|_| ApiError::invalid("amount", "out_of_range"))?
                .decimal(),
            ),
            None => ValueState::Missing(MissingValue::TransferDestinationValuation),
        }
    };
    let value = transfer_funding_effect(
        source.budget_eligible,
        source_value,
        destination.budget_eligible,
        destination_value,
    )
    .map_err(calculation_error)?;
    Ok(Some(FundingEvent {
        date: source.business_date,
        kind: FundingKind::Transfer,
        eligible: source.budget_eligible || destination.budget_eligible,
        value_clp: value,
    }))
}

fn evidence_clp_amount(evidence: &Option<Value>) -> Result<Option<Decimal>, ApiError> {
    let Some(evidence) = evidence.as_ref() else {
        return Ok(None);
    };
    let value = evidence
        .get("destination_clp_amount")
        .or_else(|| evidence.get("received_clp_amount"));
    let Some(value) = value else {
        return Ok(None);
    };
    let text = value
        .as_str()
        .ok_or_else(|| ApiError::invalid("conversion_evidence", "clp_amount_must_be_string"))?;
    let amount = MoneyAmount::parse(text)
        .map_err(|_| ApiError::invalid("conversion_evidence", "invalid_clp_amount"))?;
    if amount.decimal() <= Decimal::ZERO {
        return Err(ApiError::invalid(
            "conversion_evidence",
            "clp_amount_must_be_positive",
        ));
    }
    Ok(Some(amount.decimal()))
}

fn calculate_funding(
    funding_events: &[FundingEvent],
    activities: &[TransactionActivity],
    period: BudgetPeriod,
) -> Result<CalculatedValue, ApiError> {
    let activity_funding = attribute_transactions(activities).map_err(calculation_error)?;
    let mut total = Decimal::ZERO;
    let mut missing = Vec::new();
    for event in funding_events
        .iter()
        .chain(activity_funding.funding_events.iter())
    {
        if !event.eligible || !period.contains(event.date) {
            continue;
        }
        match event.value_clp {
            ValueState::Known(amount) => {
                total = total.checked_add(amount).ok_or_else(|| {
                    ApiError::new(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "calculation_overflow",
                        "Budget funding could not be represented exactly.",
                    )
                })?;
                MoneyAmount::new(total).map_err(stored_money_error)?;
            }
            ValueState::Missing(reason) => {
                if !missing.contains(&reason) {
                    missing.push(reason);
                }
            }
        }
    }
    if missing.is_empty() {
        Ok(CalculatedValue::Complete(total))
    } else {
        Ok(CalculatedValue::Unavailable(missing))
    }
}

fn to_budget_response(
    period_start: NaiveDate,
    period_text: String,
    budget: &CurrentBudget,
    data: &BudgetData,
) -> Result<BudgetResponse, ApiError> {
    let result = &budget.result;
    let mut activity_total = Decimal::ZERO;
    let mut activity_unresolved = Vec::new();
    let mut activity_missing = Vec::new();
    for value in result
        .category_activity_clp
        .values()
        .chain(std::iter::once(&result.uncategorized_activity_clp))
    {
        match value {
            CalculatedValue::Complete(amount) => {
                activity_total = activity_total.checked_add(*amount).ok_or_else(|| {
                    ApiError::new(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "calculation_overflow",
                        "Budget activity could not be represented exactly.",
                    )
                })?;
            }
            CalculatedValue::Incomplete {
                known_subtotal,
                unresolved,
            } => {
                activity_total = activity_total.checked_add(*known_subtotal).ok_or_else(|| {
                    ApiError::new(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "calculation_overflow",
                        "Budget activity could not be represented exactly.",
                    )
                })?;
                activity_unresolved.extend(unresolved.iter().map(unresolved_to_wire));
            }
            CalculatedValue::Unavailable(reasons) => {
                activity_missing.extend(reasons.iter().map(missing_reason));
            }
        }
    }
    let activity = if !activity_missing.is_empty() {
        BudgetValue::Unavailable {
            reasons: activity_missing,
        }
    } else if !activity_unresolved.is_empty() {
        BudgetValue::Incomplete {
            known_subtotal: MoneyAmount::new(activity_total).map_err(stored_money_error)?,
            unresolved: activity_unresolved,
        }
    } else {
        BudgetValue::Complete {
            amount: MoneyAmount::new(activity_total).map_err(stored_money_error)?,
        }
    };

    let mut categories = Vec::new();
    let mut archived_categories = Vec::new();
    for category in &data.categories {
        let id = category.id.to_string();
        let assigned = data.assignments.iter().find(|assignment| {
            assignment.period_start == period_start && assignment.category_id == category.id
        });
        let assigned_amount = assigned.map(|row| row.amount).unwrap_or(Decimal::ZERO);
        let target = if category.archived {
            &mut archived_categories
        } else {
            &mut categories
        };
        target.push(BudgetCategory {
            id: category.id,
            group_id: category.group_id,
            archived: category.archived,
            name: category.name.clone(),
            sort_order: category.sort_order,
            assigned: MoneyAmount::new(assigned_amount).map_err(stored_money_error)?,
            assignment_intent: None,
            assignment_revision: assigned.map(|row| row.revision),
            activity: result
                .category_activity_clp
                .get(&id)
                .map(value_to_wire)
                .transpose()?
                .unwrap_or(BudgetValue::Complete {
                    amount: MoneyAmount::new(Decimal::ZERO).map_err(stored_money_error)?,
                }),
            remaining: result
                .category_remaining_clp
                .get(&id)
                .map(value_to_wire)
                .transpose()?
                .unwrap_or(BudgetValue::Complete {
                    amount: MoneyAmount::new(Decimal::ZERO).map_err(stored_money_error)?,
                }),
        });
    }

    Ok(BudgetResponse {
        period: period_text,
        revision: data.revision.clone(),
        closed: false,
        funding: value_to_wire(&budget.funding)?,
        ready_to_assign: value_to_wire(&result.ready_to_assign_clp)?,
        activity,
        uncategorized_activity: value_to_wire(&result.uncategorized_activity_clp)?,
        complete: result.can_close,
        categories,
        archived_categories,
        groups: data.groups.clone(),
        available_to_budget_breakdown: budget.breakdown.clone(),
    })
}

fn value_to_wire(value: &CalculatedValue) -> Result<BudgetValue, ApiError> {
    match value {
        CalculatedValue::Complete(amount) => Ok(BudgetValue::Complete {
            amount: MoneyAmount::new(*amount).map_err(stored_money_error)?,
        }),
        CalculatedValue::Unavailable(reasons) => Ok(BudgetValue::Unavailable {
            reasons: reasons.iter().map(missing_reason).collect(),
        }),
        CalculatedValue::Incomplete {
            known_subtotal,
            unresolved,
        } => Ok(BudgetValue::Incomplete {
            known_subtotal: MoneyAmount::new(*known_subtotal).map_err(stored_money_error)?,
            unresolved: unresolved.iter().map(unresolved_to_wire).collect(),
        }),
    }
}

fn unresolved_to_wire(
    unresolved: &crate::budget_calculations::UnresolvedActivity,
) -> UnresolvedValue {
    UnresolvedValue {
        kind: activity_kind_name(unresolved.kind),
        category_id: unresolved.category_id.clone(),
        currency: unresolved.currency.clone(),
        amount: unresolved
            .amount
            .and_then(|value| MoneyAmount::new(value).ok()),
    }
}

fn activity_kind_name(kind: ActivityKind) -> String {
    match kind {
        ActivityKind::Income => "income",
        ActivityKind::Expense => "expense",
        ActivityKind::Refund => "refund",
        ActivityKind::Transfer => "transfer",
    }
    .to_owned()
}

fn missing_reason(reason: &MissingValue) -> String {
    match reason {
        MissingValue::ForeignAssetRate => "foreign_asset_rate_unavailable",
        MissingValue::ForeignLiabilityBasis => "foreign_liability_basis_unavailable",
        MissingValue::TransferSourceValuation => "transfer_source_valuation_unavailable",
        MissingValue::TransferDestinationValuation => "transfer_destination_valuation_unavailable",
        MissingValue::TransactionClpValue => "transaction_clp_value_unavailable",
    }
    .to_owned()
}

fn activity_kind(kind: &str) -> Result<ActivityKind, ApiError> {
    match kind {
        "income" => Ok(ActivityKind::Income),
        "expense" => Ok(ActivityKind::Expense),
        "refund" => Ok(ActivityKind::Refund),
        "transfer" => Ok(ActivityKind::Transfer),
        _ => Err(ApiError::invalid("kind", "unsupported_transaction_kind")),
    }
}

fn first_of_month(date: NaiveDate) -> NaiveDate {
    NaiveDate::from_ymd_opt(date.year(), date.month(), 1)
        .expect("An existing date has a valid first day of its month")
}

fn next_month(date: NaiveDate) -> Result<NaiveDate, ApiError> {
    let (year, month) = if date.month() == 12 {
        (date.year() + 1, 1)
    } else {
        (date.year(), date.month() + 1)
    };
    NaiveDate::from_ymd_opt(year, month, 1)
        .ok_or_else(|| ApiError::invalid("period", "invalid_period"))
}

fn last_day_of_month(date: NaiveDate) -> Result<NaiveDate, ApiError> {
    next_month(first_of_month(date))?
        .pred_opt()
        .ok_or_else(|| ApiError::invalid("period", "invalid_period"))
}

fn parse_period(period: &str) -> Result<NaiveDate, ApiError> {
    if period.len() != 7 {
        return Err(ApiError::invalid("period", "must_be_yyyy_mm"));
    }
    NaiveDate::parse_from_str(&format!("{period}-01"), "%Y-%m-%d")
        .map_err(|_| ApiError::invalid("period", "must_be_yyyy_mm"))
}

fn validate_category(name: &str, sort_order: i16) -> Result<(), ApiError> {
    if name.trim().is_empty() {
        return Err(ApiError::invalid("name", "required"));
    }
    if sort_order < 0 {
        return Err(ApiError::invalid("sort_order", "must_be_nonnegative"));
    }
    Ok(())
}

fn calculation_error(error: crate::budget_calculations::CalculationError) -> ApiError {
    match error {
        crate::budget_calculations::CalculationError::InvalidPeriod => {
            ApiError::invalid("period", "invalid_period")
        }
        crate::budget_calculations::CalculationError::NegativeAssignment => {
            ApiError::invalid("amount", "must_be_nonnegative")
        }
        crate::budget_calculations::CalculationError::AmountOutOfRange
        | crate::budget_calculations::CalculationError::ArithmeticOverflow => ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "calculation_out_of_range",
            "The stored financial facts could not be represented exactly.",
        ),
    }
}

fn stored_money_error(_: impl std::fmt::Debug) -> ApiError {
    ApiError::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        "invalid_stored_amount",
        "A stored budget value is outside the supported amount range.",
    )
}

fn idempotency_key(headers: &HeaderMap) -> Result<Uuid, ApiError> {
    headers
        .get("Idempotency-Key")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| Uuid::parse_str(value).ok())
        .ok_or_else(|| ApiError::invalid("Idempotency-Key", "required_uuid"))
}

fn parse_body<T: for<'de> Deserialize<'de>>(body: &[u8]) -> Result<T, ApiError> {
    serde_json::from_slice(body)
        .map_err(|_| ApiError::bad_request("invalid_json", "Request body must be valid JSON."))
}

async fn find_receipt(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    workspace_id: Uuid,
    key: Uuid,
    method: &str,
    route: &str,
    hash: &[u8],
) -> Result<Option<StoredReceipt>, ApiError> {
    let receipt = sqlx::query_as::<_, StoredReceipt>(
        "SELECT http_method, route, request_hash, response_status, resource_id, resource_revision \
         FROM financial_idempotency_receipts WHERE workspace_id=$1 AND idempotency_key=$2",
    )
    .bind(workspace_id)
    .bind(key)
    .fetch_optional(&mut **tx)
    .await
    .map_err(ApiError::database)?;
    if let Some(receipt) = receipt {
        if receipt.http_method != method || receipt.route != route || receipt.request_hash != hash {
            return Err(ApiError::idempotency_conflict());
        }
        Ok(Some(receipt))
    } else {
        Ok(None)
    }
}

async fn save_receipt(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    workspace_id: Uuid,
    key: Uuid,
    method: &str,
    route: &str,
    hash: &[u8],
    status: StatusCode,
    resource_id: Uuid,
    revision: i64,
) -> Result<(), ApiError> {
    sqlx::query(
        "INSERT INTO financial_idempotency_receipts \
         (workspace_id,idempotency_key,http_method,route,request_hash,response_status,resource_id,resource_revision) \
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8)",
    )
    .bind(workspace_id)
    .bind(key)
    .bind(method)
    .bind(route)
    .bind(hash)
    .bind(status.as_u16() as i16)
    .bind(resource_id)
    .bind(revision)
    .execute(&mut **tx)
    .await
    .map_err(ApiError::database)?;
    Ok(())
}

// Keep the eligibility display and destructive guard identical. Include retained
// correction/snapshot references even after a transaction has been recategorized.

fn check_organization_revision(expected: Option<i64>, current: i64) -> Result<(), ApiError> {
    if expected.is_some_and(|revision| revision != current) {
        return Err(ApiError::stale_revision());
    }
    Ok(())
}

async fn active_group_in(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    workspace: Uuid,
    group: Option<Uuid>,
) -> Result<Uuid, ApiError> {
    sqlx::query_scalar("SELECT id FROM budget_category_groups WHERE workspace_id=$1 AND NOT archived AND (($2::uuid IS NOT NULL AND id=$2) OR ($2 IS NULL AND slug='general'))")
        .bind(workspace).bind(group).fetch_optional(&mut **tx).await.map_err(ApiError::database)?
        .ok_or_else(|| ApiError::invalid("group_id", "active_group_required"))
}

async fn list_groups(State(state): State<AppState>) -> Result<Json<Vec<BudgetGroup>>, ApiError> {
    let groups = sqlx::query_as::<_, BudgetGroup>("SELECT id,name,sort_order,archived,revision FROM budget_category_groups WHERE workspace_id=$1 ORDER BY sort_order,id")
        .bind(state.workspace_id).fetch_all(&state.pool).await.map_err(ApiError::database)?;
    Ok(Json(groups))
}

async fn create_group(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<impl IntoResponse, ApiError> {
    let key = idempotency_key(&headers)?;
    let generation = installation_generation(&headers)?;
    retry_transaction(|| group_attempt(state.clone(), generation, None, key, body.clone())).await
}

async fn edit_group(
    State(state): State<AppState>,
    path: Result<Path<Uuid>, PathRejection>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<impl IntoResponse, ApiError> {
    let Path(id) = path.map_err(|_| ApiError::invalid_request_parameter())?;
    let key = idempotency_key(&headers)?;
    let generation = installation_generation(&headers)?;
    retry_transaction(|| group_attempt(state.clone(), generation, Some(id), key, body.clone()))
        .await
}

async fn group_attempt(
    state: AppState,
    generation: Uuid,
    existing: Option<Uuid>,
    key: Uuid,
    body: Bytes,
) -> Result<impl IntoResponse, ApiError> {
    let (method, route) = existing
        .map(|id| ("PUT", format!("/api/budget-groups/{id}")))
        .unwrap_or(("POST", "/api/budget-groups".to_owned()));
    let hash = Sha256::digest(&body).to_vec();
    let mut tx = state.pool.begin().await.map_err(ApiError::database)?;
    lock_workspace(&mut tx, state.workspace_id, generation).await?;
    if let Some(receipt) =
        find_receipt(&mut tx, state.workspace_id, key, method, &route, &hash).await?
    {
        tx.commit().await.map_err(ApiError::database)?;
        return Ok((
            StatusCode::from_u16(receipt.response_status as u16).unwrap_or(StatusCode::OK),
            Json(MutationReceipt {
                resource_id: receipt.resource_id.ok_or_else(ApiError::not_found)?,
                revision: receipt.resource_revision.unwrap_or(1),
                period_revision: None,
            }),
        ));
    }
    let request: GroupRequest = parse_body(&body)?;
    validate_category(&request.name, request.sort_order)?;
    let id = existing.unwrap_or_else(Uuid::new_v4);
    let revision = if existing.is_some() {
        let current: i64 = sqlx::query_scalar("SELECT revision FROM budget_category_groups WHERE workspace_id=$1 AND id=$2 FOR UPDATE")
            .bind(state.workspace_id).bind(id).fetch_optional(&mut *tx).await.map_err(ApiError::database)?.ok_or_else(ApiError::not_found)?;
        check_organization_revision(request.expected_revision, current)?;
        if request.archived {
            let active: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM expense_categories WHERE workspace_id=$1 AND group_id=$2 AND NOT archived)")
                .bind(state.workspace_id).bind(id).fetch_one(&mut *tx).await.map_err(ApiError::database)?;
            if active {
                return Err(ApiError::new(
                    StatusCode::CONFLICT,
                    "group_has_active_categories",
                    "Move or archive active categories before archiving their group.",
                ));
            }
        }
        sqlx::query("UPDATE budget_category_groups SET name=$3,sort_order=$4,archived=$5,revision=revision+1,updated_at=now() WHERE workspace_id=$1 AND id=$2")
            .bind(state.workspace_id).bind(id).bind(request.name.trim()).bind(request.sort_order).bind(request.archived).execute(&mut *tx).await.map_err(ApiError::database)?;
        current + 1
    } else {
        if request.archived {
            return Err(ApiError::invalid("archived", "new_group_must_be_active"));
        }
        sqlx::query("INSERT INTO budget_category_groups (id,workspace_id,slug,name,sort_order) VALUES ($1,$2,$3,$4,$5)")
            .bind(id).bind(state.workspace_id).bind(format!("custom-{}",id.simple())).bind(request.name.trim()).bind(request.sort_order).execute(&mut *tx).await.map_err(ApiError::database)?;
        1
    };
    let status = if existing.is_some() {
        StatusCode::OK
    } else {
        StatusCode::CREATED
    };
    save_receipt(
        &mut tx,
        state.workspace_id,
        key,
        method,
        &route,
        &hash,
        status,
        id,
        revision,
    )
    .await?;
    tx.commit().await.map_err(ApiError::database)?;
    Ok((
        status,
        Json(MutationReceipt {
            resource_id: id,
            revision,
            period_revision: None,
        }),
    ))
}

async fn delete_category(
    State(state): State<AppState>,
    path: Result<Path<Uuid>, PathRejection>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<impl IntoResponse, ApiError> {
    let Path(id) = path.map_err(|_| ApiError::invalid_request_parameter())?;
    let key = idempotency_key(&headers)?;
    let generation = installation_generation(&headers)?;
    retry_transaction(|| delete_category_attempt(state.clone(), generation, id, key, body.clone()))
        .await
}

async fn delete_category_attempt(
    state: AppState,
    generation: Uuid,
    id: Uuid,
    key: Uuid,
    body: Bytes,
) -> Result<impl IntoResponse, ApiError> {
    let route = format!("/api/categories/{id}");
    let hash = Sha256::digest(&body).to_vec();
    let mut tx = state.pool.begin().await.map_err(ApiError::database)?;
    lock_workspace(&mut tx, state.workspace_id, generation).await?;
    if let Some(receipt) =
        find_receipt(&mut tx, state.workspace_id, key, "DELETE", &route, &hash).await?
    {
        tx.commit().await.map_err(ApiError::database)?;
        return Ok(Json(MutationReceipt {
            resource_id: id,
            revision: receipt.resource_revision.unwrap_or(1),
            period_revision: None,
        }));
    }
    let request: DeleteCategoryRequest = parse_body(&body)?;
    let revision: i64 = sqlx::query_scalar(
        "SELECT revision FROM expense_categories WHERE workspace_id=$1 AND id=$2 FOR UPDATE",
    )
    .bind(state.workspace_id)
    .bind(id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(ApiError::database)?
    .ok_or_else(ApiError::not_found)?;
    check_organization_revision(request.expected_revision, revision)?;
    let sql = concat!(
        "SELECT (",
        category_references!(),
        ") FROM expense_categories c WHERE c.workspace_id=$1 AND c.id=$2"
    );
    let referenced: bool = sqlx::query_scalar(sql)
        .bind(state.workspace_id)
        .bind(id)
        .fetch_one(&mut *tx)
        .await
        .map_err(ApiError::database)?;
    if referenced {
        return Err(ApiError::new(
            StatusCode::CONFLICT,
            "category_in_use",
            "Archive this category to preserve its transactions and budget history.",
        ));
    }
    sqlx::query("DELETE FROM expense_categories WHERE workspace_id=$1 AND id=$2")
        .bind(state.workspace_id)
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(ApiError::database)?;
    save_receipt(
        &mut tx,
        state.workspace_id,
        key,
        "DELETE",
        &route,
        &hash,
        StatusCode::OK,
        id,
        revision,
    )
    .await?;
    tx.commit().await.map_err(ApiError::database)?;
    Ok(Json(MutationReceipt {
        resource_id: id,
        revision,
        period_revision: None,
    }))
}

fn known_budget_value(amount: Decimal) -> Result<BudgetValue, ApiError> {
    Ok(BudgetValue::Complete {
        amount: MoneyAmount::new(amount).map_err(stored_money_error)?,
    })
}

fn sum_budget_amounts(values: impl IntoIterator<Item = Decimal>) -> Result<Decimal, ApiError> {
    values.into_iter().try_fold(Decimal::ZERO, |total, amount| {
        let value = total.checked_add(amount).ok_or_else(|| {
            calculation_api_error("Budget explanation exceeded the supported range.")
        })?;
        MoneyAmount::new(value).map_err(stored_money_error)?;
        if value.checked_sub(total) != Some(amount) || value.checked_sub(amount) != Some(total) {
            return Err(calculation_api_error(
                "Budget explanation cannot be represented exactly.",
            ));
        }
        Ok(value)
    })
}

fn amount_component(values: impl IntoIterator<Item = Decimal>) -> BudgetValue {
    sum_budget_amounts(values)
        .and_then(known_budget_value)
        .unwrap_or_else(|_| BudgetValue::Unavailable {
            reasons: vec!["component_out_of_range".to_owned()],
        })
}

fn open_breakdown(
    input: &PeriodInput,
    inherited_missing: &[MissingValue],
) -> Result<AvailableToBudgetBreakdown, ApiError> {
    let attributed = attribute_transactions(&input.activities).map_err(calculation_error)?;
    let component = |kind: FundingKind| -> Result<BudgetValue, ApiError> {
        let events = input
            .funding_events
            .iter()
            .chain(attributed.funding_events.iter())
            .filter(|event| {
                event.eligible && input.period.contains(event.date) && event.kind == kind
            });
        let mut values = Vec::new();
        let mut missing = Vec::new();
        for event in events {
            match event.value_clp {
                ValueState::Known(amount) => values.push(amount),
                ValueState::Missing(reason) => {
                    if !missing.contains(&reason) {
                        missing.push(reason);
                    }
                }
            }
        }
        if missing.is_empty() {
            Ok(amount_component(values))
        } else {
            value_to_wire(&CalculatedValue::Unavailable(missing))
        }
    };
    let prior_unassigned = if inherited_missing.is_empty() {
        known_budget_value(input.prior_unassigned_clp)?
    } else {
        value_to_wire(&CalculatedValue::Unavailable(inherited_missing.to_vec()))?
    };
    Ok(AvailableToBudgetBreakdown {
        prior_unassigned,
        funding_total: value_to_wire(&calculate_funding(
            &input.funding_events,
            &input.activities,
            input.period,
        )?)?,
        income: component(FundingKind::Income)?,
        openings: component(FundingKind::Opening)?,
        transfers: component(FundingKind::Transfer)?,
        valuation: component(FundingKind::ForeignAssetRevaluation)?,
        corrections: component(FundingKind::OtherCorrection)?,
        category_releases: amount_component(
            input.released_category_remainders_clp.values().copied(),
        ),
        category_rollover: known_budget_value(Decimal::ZERO)?,
        assignments: amount_component(input.assignments_clp.values().copied()),
    })
}

fn closed_breakdown(
    target: NaiveDate,
    snapshot: &CloseSnapshotRow,
    data: &BudgetData,
) -> Result<AvailableToBudgetBreakdown, ApiError> {
    if let Some(stored) = &snapshot.available_to_budget_breakdown {
        return serde_json::from_value(stored.clone())
            .map_err(|_| stored_money_error("invalid closed explanation"));
    }
    // Older snapshots retain the funding total but not its source components.
    // Derive carry only from immutable snapshot values, never today's facts.
    let prior_close = data
        .close_snapshots
        .iter()
        .filter(|row| row.period_start < target)
        .max_by_key(|row| row.period_start);
    let releases = amount_component(
        data.close_category_snapshots
            .iter()
            .filter(|row| {
                Some(row.snapshot_id) == prior_close.map(|close| close.id)
                    && row.disposition == "release"
            })
            .map(|row| row.remaining),
    );
    let assignments = amount_component(
        data.close_category_snapshots
            .iter()
            .filter(|row| row.snapshot_id == snapshot.id)
            .map(|row| row.assigned),
    );
    let carry = match (complete_amount(&releases), complete_amount(&assignments)) {
        (Some(releases), Some(assignments)) => amount_component([
            snapshot.ready_to_assign,
            assignments,
            -snapshot.funding,
            -releases,
        ]),
        _ => BudgetValue::Unavailable {
            reasons: vec!["component_out_of_range".to_owned()],
        },
    };
    let unavailable = || BudgetValue::Unavailable {
        reasons: vec!["closed_snapshot_breakdown_unavailable".to_owned()],
    };
    Ok(AvailableToBudgetBreakdown {
        prior_unassigned: carry,
        funding_total: known_budget_value(snapshot.funding)?,
        income: unavailable(),
        openings: unavailable(),
        transfers: unavailable(),
        valuation: unavailable(),
        corrections: unavailable(),
        category_releases: releases,
        category_rollover: known_budget_value(Decimal::ZERO)?,
        assignments,
    })
}

#[cfg(test)]
mod workspace_breakdown_tests {
    use super::*;

    #[test]
    fn component_sum_reports_unavailable_instead_of_rounding_money() {
        let max = Decimal::MAX;
        let fraction = Decimal::new(1, 1);
        assert!(sum_budget_amounts([max, fraction]).is_err());
        assert!(matches!(
            amount_component([max, fraction]),
            BudgetValue::Unavailable { .. }
        ));
        assert_eq!(sum_budget_amounts([-max, max, fraction]).unwrap(), fraction);
    }
}
