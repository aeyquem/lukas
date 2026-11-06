use axum::{
    Json, Router,
    body::Bytes,
    extract::{
        Path, Query, State,
        rejection::{PathRejection, QueryRejection},
    },
    http::{HeaderMap, StatusCode},
    routing::get,
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use sqlx::FromRow;
use uuid::Uuid;

use crate::{
    budget_api::{capture_transaction_budget_state, record_closed_transaction_correction},
    financial_values::{BusinessDate, MoneyAmount},
    health::AppState,
    http_error::{ApiError, retry_transaction},
    write_guard::{installation_generation, lock_workspace},
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/transactions", get(list).post(create))
        .route("/api/transactions/{transaction_id}", get(read).put(edit))
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct TransactionRequest {
    kind: String,
    business_date: BusinessDate,
    wallet_id: Option<Uuid>,
    amount: Option<MoneyAmount>,
    opening_treatment: Option<String>,
    source_wallet_id: Option<Uuid>,
    source_amount: Option<MoneyAmount>,
    source_opening_treatment: Option<String>,
    destination_wallet_id: Option<Uuid>,
    destination_amount: Option<MoneyAmount>,
    destination_opening_treatment: Option<String>,
    category_id: Option<Uuid>,
    source_expense_id: Option<Uuid>,
    payee: Option<String>,
    note: Option<String>,
    confirmed_clp_amount: Option<MoneyAmount>,
    conversion_evidence: Option<Value>,
    expected_revision: Option<i64>,
    reason: Option<String>,
}

#[derive(Serialize)]
struct Receipt {
    resource_id: Uuid,
    revision: i64,
}
#[derive(Serialize)]
struct TransactionView {
    id: Uuid,
    kind: String,
    business_date: NaiveDate,
    category_id: Option<Uuid>,
    source_expense_id: Option<Uuid>,
    payee: Option<String>,
    note: Option<String>,
    confirmed_clp_amount: Option<MoneyAmount>,
    conversion_evidence: Option<Value>,
    revision: i64,
    effects: Vec<EffectView>,
}
#[derive(Serialize)]
struct EffectView {
    wallet_id: Uuid,
    currency: String,
    role: String,
    amount: MoneyAmount,
    opening_treatment: Option<String>,
}
#[derive(Serialize)]
struct Page {
    items: Vec<TransactionView>,
    next: Option<Cursor>,
}
#[derive(Serialize)]
struct Cursor {
    before_date: NaiveDate,
    before_id: Uuid,
}
#[derive(Deserialize)]
struct PageQuery {
    page_size: Option<i64>,
    before_date: Option<NaiveDate>,
    before_id: Option<Uuid>,
    account_id: Option<Uuid>,
}
#[derive(FromRow)]
struct ReceiptRow {
    http_method: String,
    route: String,
    request_hash: Vec<u8>,
    response_status: i16,
    resource_id: Option<Uuid>,
    resource_revision: Option<i64>,
}
#[derive(FromRow)]
struct StoredTransaction {
    id: Uuid,
    transaction_kind: String,
    business_date: NaiveDate,
    category_id: Option<Uuid>,
    source_expense_id: Option<Uuid>,
    payee: Option<String>,
    note: Option<String>,
    confirmed_clp_amount: Option<Decimal>,
    conversion_evidence: Option<Value>,
    revision: i64,
}
#[derive(FromRow)]
struct StoredEffect {
    wallet_id: Uuid,
    currency: String,
    effect_role: String,
    amount: Decimal,
    opening_treatment: Option<String>,
}
#[derive(FromRow)]
struct WalletInfo {
    currency: String,
}
#[derive(FromRow)]
struct EditTarget {
    revision: i64,
    transaction_kind: String,
    business_date: NaiveDate,
    category_id: Option<Uuid>,
}
#[derive(FromRow)]
struct LinkedRefund {
    id: Uuid,
    revision: i64,
    business_date: NaiveDate,
}

async fn create(
    State(s): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<Receipt>), ApiError> {
    retry_transaction(|| create_attempt(State(s.clone()), headers.clone(), body.clone())).await
}

async fn create_attempt(
    State(s): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<Receipt>), ApiError> {
    let key = idempotency_key(&headers)?;
    let generation = installation_generation(&headers)?;
    let route = "/api/transactions";
    let hash = Sha256::digest(&body).to_vec();
    let mut tx = s.pool.begin().await.map_err(ApiError::database)?;
    lock_workspace(&mut tx, s.workspace_id, generation).await?;
    if let Some(saved) = replay(&mut tx, s.workspace_id, key, "POST", route, &hash).await? {
        tx.commit().await.map_err(ApiError::database)?;
        return Ok((
            StatusCode::from_u16(saved.0).unwrap_or(StatusCode::OK),
            Json(saved.1),
        ));
    }
    let req: TransactionRequest = serde_json::from_slice(&body).map_err(|_| {
        ApiError::bad_request(
            "invalid_json",
            "Request body must be valid transaction JSON.",
        )
    })?;
    validate_shape(&req, false)?;
    let id = Uuid::new_v4();
    let revision = 1;
    let (category, source_expense) = resolve_links(&mut tx, s.workspace_id, &req, None).await?;
    let effects = resolve_effects(&mut tx, s.workspace_id, &req).await?;
    insert_transaction(
        &mut tx,
        s.workspace_id,
        id,
        &req,
        category,
        source_expense,
        revision,
    )
    .await?;
    for (wallet, role, signed, treatment) in effects {
        insert_effect(&mut tx, s.workspace_id, id, wallet, role, signed, treatment).await?;
    }
    save_receipt(
        &mut tx,
        s.workspace_id,
        key,
        "POST",
        route,
        &hash,
        StatusCode::CREATED.as_u16() as i16,
        id,
        revision,
    )
    .await?;
    tx.commit().await.map_err(ApiError::database)?;
    Ok((
        StatusCode::CREATED,
        Json(Receipt {
            resource_id: id,
            revision,
        }),
    ))
}

async fn edit(
    State(s): State<AppState>,
    path: Result<Path<Uuid>, PathRejection>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<Receipt>, ApiError> {
    let Path(id) = path.map_err(|_| ApiError::invalid_request_parameter())?;
    idempotency_key(&headers)?;
    installation_generation(&headers)?;
    let request: TransactionRequest = serde_json::from_slice(&body).map_err(|_| {
        ApiError::bad_request(
            "invalid_json",
            "Request body must be valid transaction JSON.",
        )
    })?;
    validate_shape(&request, true)?;
    retry_transaction(|| edit_attempt(State(s.clone()), Path(id), headers.clone(), body.clone()))
        .await
}

async fn edit_attempt(
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<Receipt>, ApiError> {
    let key = idempotency_key(&headers)?;
    let generation = installation_generation(&headers)?;
    let route = format!("/api/transactions/{id}");
    let hash = Sha256::digest(&body).to_vec();
    let mut tx = s.pool.begin().await.map_err(ApiError::database)?;
    lock_workspace(&mut tx, s.workspace_id, generation).await?;
    if let Some(saved) = replay(&mut tx, s.workspace_id, key, "PUT", &route, &hash).await? {
        tx.commit().await.map_err(ApiError::database)?;
        return Ok(Json(saved.1));
    }
    let req: TransactionRequest = serde_json::from_slice(&body).map_err(|_| {
        ApiError::bad_request(
            "invalid_json",
            "Request body must be valid transaction JSON.",
        )
    })?;
    validate_shape(&req, true)?;
    let expected = req.expected_revision.unwrap();
    let target = sqlx::query_as::<_, EditTarget>(
        "SELECT revision,transaction_kind,business_date,category_id FROM financial_transactions WHERE workspace_id=$1 AND id=$2 FOR UPDATE",
    )
    .bind(s.workspace_id)
    .bind(id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(ApiError::database)?
    .ok_or_else(ApiError::not_found)?;
    let current = target.revision;
    if current != expected {
        return Err(ApiError::stale_revision());
    }
    let old_budget_state = capture_transaction_budget_state(&mut tx, s.workspace_id, id).await?;
    let (category, source_expense) = resolve_links(&mut tx, s.workspace_id, &req, Some(id)).await?;
    let has_linked_refunds: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM financial_transactions WHERE workspace_id=$1 AND source_expense_id=$2)",
    )
    .bind(s.workspace_id)
    .bind(id)
    .fetch_one(&mut *tx)
    .await
    .map_err(ApiError::database)?;
    if has_linked_refunds && (target.transaction_kind != "expense" || req.kind != "expense") {
        return Err(ApiError::invalid(
            "kind",
            "linked_refunds_require_expense_source",
        ));
    }
    let effects = resolve_effects(&mut tx, s.workspace_id, &req).await?;
    sqlx::query("UPDATE financial_transactions SET transaction_kind=$3,business_date=$4,category_id=$5,source_expense_id=$6,payee=$7,note=$8,confirmed_clp_amount=$9,conversion_evidence=$10,revision=$11,updated_at=now() WHERE workspace_id=$1 AND id=$2")
        .bind(s.workspace_id).bind(id).bind(&req.kind).bind(req.business_date.date()).bind(category).bind(source_expense).bind(&req.payee).bind(&req.note).bind(req.confirmed_clp_amount.map(MoneyAmount::decimal)).bind(req.conversion_evidence.clone()).bind(current+1).execute(&mut *tx).await.map_err(ApiError::database)?;
    if has_linked_refunds && target.category_id != category {
        let linked = sqlx::query_as::<_, LinkedRefund>(
            "SELECT id,revision,business_date FROM financial_transactions WHERE workspace_id=$1 AND source_expense_id=$2 ORDER BY id FOR UPDATE",
        )
        .bind(s.workspace_id)
        .bind(id)
        .fetch_all(&mut *tx)
        .await
        .map_err(ApiError::database)?;
        for refund in linked {
            let old_refund_state =
                capture_transaction_budget_state(&mut tx, s.workspace_id, refund.id).await?;
            sqlx::query("UPDATE financial_transactions SET category_id=$3,revision=$4,updated_at=now() WHERE workspace_id=$1 AND id=$2")
                .bind(s.workspace_id)
                .bind(refund.id)
                .bind(category)
                .bind(refund.revision + 1)
                .execute(&mut *tx)
                .await
                .map_err(ApiError::database)?;
            let linked_correction_period = correction_period(
                &mut tx,
                s.workspace_id,
                refund.business_date,
                refund.business_date,
            )
            .await?;
            let linked_correction_id = Uuid::new_v4();
            sqlx::query("INSERT INTO financial_corrections(id,workspace_id,record_kind,transaction_id,prior_revision,new_revision,reason,correction_period,transaction_target_business_date,transaction_target_kind,transaction_target_category_id,transaction_target_confirmed_clp_amount,transaction_target_conversion_evidence) VALUES($1,$2,'transaction',$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)")
                .bind(linked_correction_id)
                .bind(s.workspace_id)
                .bind(refund.id)
                .bind(refund.revision)
                .bind(refund.revision + 1)
                .bind("Source expense category changed")
                .bind(linked_correction_period)
                .bind(old_refund_state.business_date)
                .bind(&old_refund_state.kind)
                .bind(category)
                .bind(old_refund_state.confirmed_clp_amount)
                .bind(old_refund_state.conversion_evidence.clone())
                .execute(&mut *tx)
                .await
                .map_err(ApiError::database)?;
            if let Some(correction_period) = linked_correction_period {
                let new_refund_state =
                    capture_transaction_budget_state(&mut tx, s.workspace_id, refund.id).await?;
                record_closed_transaction_correction(
                    &mut tx,
                    s.workspace_id,
                    correction_period,
                    linked_correction_id,
                    &old_refund_state,
                    &new_refund_state,
                )
                .await?;
            }
        }
    }
    sqlx::query("DELETE FROM financial_wallet_effects WHERE workspace_id=$1 AND transaction_id=$2")
        .bind(s.workspace_id)
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(ApiError::database)?;
    for (wallet, role, signed, treatment) in effects {
        insert_effect(&mut tx, s.workspace_id, id, wallet, role, signed, treatment).await?;
    }
    let correction_period = correction_period(
        &mut tx,
        s.workspace_id,
        target.business_date,
        req.business_date.date(),
    )
    .await?;
    let correction_id = Uuid::new_v4();
    sqlx::query("INSERT INTO financial_corrections(id,workspace_id,record_kind,transaction_id,prior_revision,new_revision,reason,correction_period,transaction_target_business_date,transaction_target_kind,transaction_target_category_id,transaction_target_confirmed_clp_amount,transaction_target_conversion_evidence) VALUES($1,$2,'transaction',$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)")
        .bind(correction_id).bind(s.workspace_id).bind(id).bind(current).bind(current+1).bind(req.reason.as_deref().unwrap_or("").trim()).bind(correction_period).bind(req.business_date.date()).bind(&req.kind).bind(category).bind(req.confirmed_clp_amount.map(MoneyAmount::decimal)).bind(req.conversion_evidence.clone()).execute(&mut *tx).await.map_err(ApiError::database)?;
    if let Some(correction_period) = correction_period {
        let new_budget_state =
            capture_transaction_budget_state(&mut tx, s.workspace_id, id).await?;
        record_closed_transaction_correction(
            &mut tx,
            s.workspace_id,
            correction_period,
            correction_id,
            &old_budget_state,
            &new_budget_state,
        )
        .await?;
    }
    save_receipt(
        &mut tx,
        s.workspace_id,
        key,
        "PUT",
        &route,
        &hash,
        StatusCode::OK.as_u16() as i16,
        id,
        current + 1,
    )
    .await?;
    tx.commit().await.map_err(ApiError::database)?;
    Ok(Json(Receipt {
        resource_id: id,
        revision: current + 1,
    }))
}

async fn read(
    State(s): State<AppState>,
    path: Result<Path<Uuid>, PathRejection>,
) -> Result<Json<TransactionView>, ApiError> {
    let Path(id) = path.map_err(|_| ApiError::invalid_request_parameter())?;
    let mut tx = s.pool.begin().await.map_err(ApiError::database)?;
    sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY")
        .execute(&mut *tx)
        .await
        .map_err(ApiError::database)?;
    let result = load_view(&mut tx, s.workspace_id, id).await?;
    tx.commit().await.map_err(ApiError::database)?;
    Ok(Json(result))
}
async fn list(
    State(s): State<AppState>,
    query: Result<Query<PageQuery>, QueryRejection>,
) -> Result<Json<Page>, ApiError> {
    let Query(q) = query.map_err(|_| ApiError::invalid_request_parameter())?;
    let size = q.page_size.unwrap_or(50);
    if !(1..=100).contains(&size) || q.before_date.is_some() != q.before_id.is_some() {
        return Err(ApiError::invalid("page_size", "invalid_page"));
    }
    let mut tx = s.pool.begin().await.map_err(ApiError::database)?;
    sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY")
        .execute(&mut *tx)
        .await
        .map_err(ApiError::database)?;
    let rows = match (q.before_date, q.before_id, q.account_id) {
        (Some(date), Some(id), Some(account_id)) => sqlx::query_as::<_, StoredTransaction>("SELECT t.id,t.transaction_kind,t.business_date,t.category_id,t.source_expense_id,t.payee,t.note,t.confirmed_clp_amount,t.conversion_evidence,t.revision FROM financial_transactions t WHERE t.workspace_id=$1 AND (t.business_date,t.id)<($2,$3) AND EXISTS(SELECT 1 FROM financial_wallet_effects e JOIN financial_wallets w ON w.workspace_id=e.workspace_id AND w.id=e.wallet_id WHERE e.workspace_id=t.workspace_id AND e.transaction_id=t.id AND w.account_id=$4) ORDER BY t.business_date DESC,t.id DESC LIMIT $5").bind(s.workspace_id).bind(date).bind(id).bind(account_id).bind(size+1).fetch_all(&mut *tx).await.map_err(ApiError::database)?,
        (Some(date), Some(id), None) => sqlx::query_as::<_, StoredTransaction>("SELECT id,transaction_kind,business_date,category_id,source_expense_id,payee,note,confirmed_clp_amount,conversion_evidence,revision FROM financial_transactions WHERE workspace_id=$1 AND (business_date,id)<($2,$3) ORDER BY business_date DESC,id DESC LIMIT $4").bind(s.workspace_id).bind(date).bind(id).bind(size+1).fetch_all(&mut *tx).await.map_err(ApiError::database)?,
        (None, None, Some(account_id)) => sqlx::query_as::<_, StoredTransaction>("SELECT t.id,t.transaction_kind,t.business_date,t.category_id,t.source_expense_id,t.payee,t.note,t.confirmed_clp_amount,t.conversion_evidence,t.revision FROM financial_transactions t WHERE t.workspace_id=$1 AND EXISTS(SELECT 1 FROM financial_wallet_effects e JOIN financial_wallets w ON w.workspace_id=e.workspace_id AND w.id=e.wallet_id WHERE e.workspace_id=t.workspace_id AND e.transaction_id=t.id AND w.account_id=$2) ORDER BY t.business_date DESC,t.id DESC LIMIT $3").bind(s.workspace_id).bind(account_id).bind(size+1).fetch_all(&mut *tx).await.map_err(ApiError::database)?,
        (None, None, None) => sqlx::query_as::<_, StoredTransaction>("SELECT id,transaction_kind,business_date,category_id,source_expense_id,payee,note,confirmed_clp_amount,conversion_evidence,revision FROM financial_transactions WHERE workspace_id=$1 ORDER BY business_date DESC,id DESC LIMIT $2").bind(s.workspace_id).bind(size+1).fetch_all(&mut *tx).await.map_err(ApiError::database)?,
        _ => return Err(ApiError::invalid("cursor", "invalid_page")),
    };
    let has_more = rows.len() as i64 > size;
    let rows: Vec<_> = rows.into_iter().take(size as usize).collect();
    let mut items = Vec::new();
    for row in rows {
        items.push(view_from_tx(&mut tx, s.workspace_id, row).await?);
    }
    let next = if has_more {
        items.last().map(|item| Cursor {
            before_date: item.business_date,
            before_id: item.id,
        })
    } else {
        None
    };
    tx.commit().await.map_err(ApiError::database)?;
    Ok(Json(Page { items, next }))
}

fn idempotency_key(headers: &HeaderMap) -> Result<Uuid, ApiError> {
    headers
        .get("Idempotency-Key")
        .and_then(|h| h.to_str().ok())
        .and_then(|v| Uuid::parse_str(v).ok())
        .ok_or_else(|| ApiError::invalid("Idempotency-Key", "required_uuid"))
}
async fn replay(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ws: Uuid,
    key: Uuid,
    method: &str,
    route: &str,
    hash: &[u8],
) -> Result<Option<(u16, Receipt)>, ApiError> {
    let row=sqlx::query_as::<_,ReceiptRow>("SELECT http_method,route,request_hash,response_status,resource_id,resource_revision FROM financial_idempotency_receipts WHERE workspace_id=$1 AND idempotency_key=$2").bind(ws).bind(key).fetch_optional(&mut **tx).await.map_err(ApiError::database)?;
    if let Some(r) = row {
        if r.http_method != method || r.route != route || r.request_hash != hash {
            return Err(ApiError::idempotency_conflict());
        }
        return Ok(Some((
            r.response_status as u16,
            Receipt {
                resource_id: r.resource_id.ok_or_else(ApiError::not_found)?,
                revision: r.resource_revision.unwrap_or(1),
            },
        )));
    }
    Ok(None)
}
async fn save_receipt(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ws: Uuid,
    key: Uuid,
    method: &str,
    route: &str,
    hash: &[u8],
    status: i16,
    id: Uuid,
    rev: i64,
) -> Result<(), ApiError> {
    sqlx::query("INSERT INTO financial_idempotency_receipts(workspace_id,idempotency_key,http_method,route,request_hash,response_status,resource_id,resource_revision) VALUES($1,$2,$3,$4,$5,$6,$7,$8)").bind(ws).bind(key).bind(method).bind(route).bind(hash).bind(status).bind(id).bind(rev).execute(&mut **tx).await.map_err(ApiError::database)?;
    Ok(())
}

fn validate_shape(r: &TransactionRequest, editing: bool) -> Result<(), ApiError> {
    if !["income", "expense", "refund", "transfer"].contains(&r.kind.as_str()) {
        return Err(ApiError::invalid("kind", "unsupported"));
    }
    if editing
        && (r.expected_revision.unwrap_or(0) < 1
            || r.reason.as_deref().unwrap_or("").trim().is_empty())
    {
        return Err(ApiError::invalid("reason", "required"));
    }
    if r.confirmed_clp_amount
        .is_some_and(|v| v.decimal() <= Decimal::ZERO)
    {
        return Err(ApiError::invalid(
            "confirmed_clp_amount",
            "must_be_positive",
        ));
    }
    let valid = if r.kind == "transfer" {
        r.source_wallet_id.is_some()
            && r.source_amount.is_some_and(|x| x.decimal() > Decimal::ZERO)
            && r.destination_wallet_id.is_some()
            && r.destination_amount
                .is_some_and(|x| x.decimal() > Decimal::ZERO)
            && r.wallet_id.is_none()
            && r.amount.is_none()
    } else {
        r.wallet_id.is_some()
            && r.amount.is_some_and(|x| x.decimal() > Decimal::ZERO)
            && r.source_wallet_id.is_none()
            && r.destination_wallet_id.is_none()
            && r.source_amount.is_none()
            && r.destination_amount.is_none()
    };
    if !valid {
        return Err(ApiError::invalid("amount", "invalid_transaction_shape"));
    }
    if r.kind == "transfer"
        && (r.category_id.is_some()
            || r.source_expense_id.is_some()
            || r.confirmed_clp_amount.is_some())
    {
        return Err(ApiError::invalid("kind", "invalid_transaction_shape"));
    }
    if r.kind != "refund" && r.source_expense_id.is_some() {
        return Err(ApiError::invalid("source_expense_id", "refund_only"));
    }
    Ok(())
}

async fn resolve_links(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ws: Uuid,
    r: &TransactionRequest,
    editing: Option<Uuid>,
) -> Result<(Option<Uuid>, Option<Uuid>), ApiError> {
    if let Some(cat) = r.category_id {
        let ok:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM expense_categories WHERE workspace_id=$1 AND id=$2 AND archived=FALSE)").bind(ws).bind(cat).fetch_one(&mut **tx).await.map_err(ApiError::database)?;
        if !ok {
            return Err(ApiError::invalid("category_id", "unknown_or_archived"));
        }
    }
    if let Some(source) = r.source_expense_id {
        if editing == Some(source) {
            return Err(ApiError::invalid(
                "source_expense_id",
                "cannot_link_to_self",
            ));
        }
        let cat:Option<Uuid>=sqlx::query_scalar("SELECT category_id FROM financial_transactions WHERE workspace_id=$1 AND id=$2 AND transaction_kind='expense'").bind(ws).bind(source).fetch_optional(&mut **tx).await.map_err(ApiError::database)?.ok_or_else(ApiError::not_found)?;
        if r.category_id.is_some() && r.category_id != cat {
            return Err(ApiError::invalid(
                "category_id",
                "must_match_source_expense",
            ));
        }
        return Ok((cat, Some(source)));
    }
    Ok((r.category_id, None))
}
async fn resolve_effects(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ws: Uuid,
    r: &TransactionRequest,
) -> Result<Vec<(Uuid, &'static str, Decimal, Option<String>)>, ApiError> {
    if r.kind == "transfer" {
        let source = r.source_wallet_id.unwrap();
        let dest = r.destination_wallet_id.unwrap();
        if source == dest {
            return Err(ApiError::invalid("destination_wallet_id", "must_differ"));
        }
        let sw = wallet_info(tx, ws, source).await?;
        let dw = wallet_info(tx, ws, dest).await?;
        let sa = r.source_amount.unwrap().decimal();
        let da = r.destination_amount.unwrap().decimal();
        if sw.currency == dw.currency && sa != da {
            return Err(ApiError::invalid(
                "destination_amount",
                "same_currency_amounts_must_match",
            ));
        }
        if sw.currency != dw.currency && r.conversion_evidence.as_ref().map_or(true, Value::is_null)
        {
            return Err(ApiError::invalid(
                "conversion_evidence",
                "required_for_cross_currency",
            ));
        }
        let st = treatment(
            tx,
            ws,
            source,
            r.business_date.date(),
            r.source_opening_treatment.as_deref(),
        )
        .await?;
        let dt = treatment(
            tx,
            ws,
            dest,
            r.business_date.date(),
            r.destination_opening_treatment.as_deref(),
        )
        .await?;
        Ok(vec![
            (source, "source", -sa, st),
            (dest, "destination", da, dt),
        ])
    } else {
        let wallet = r.wallet_id.unwrap();
        let info = wallet_info(tx, ws, wallet).await?;
        let amount = r.amount.unwrap().decimal();
        let tr = treatment(
            tx,
            ws,
            wallet,
            r.business_date.date(),
            r.opening_treatment.as_deref(),
        )
        .await?;
        if r.kind != "expense" && r.confirmed_clp_amount.is_some() {
            return Err(ApiError::invalid("confirmed_clp_amount", "expense_only"));
        }
        if r.kind == "expense" && info.currency.trim() == "CLP" && r.confirmed_clp_amount.is_some()
        {
            return Err(ApiError::invalid(
                "confirmed_clp_amount",
                "only_for_foreign_expense",
            ));
        }
        let signed = if r.kind == "expense" { -amount } else { amount };
        Ok(vec![(wallet, "single", signed, tr)])
    }
}
async fn wallet_info(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ws: Uuid,
    id: Uuid,
) -> Result<WalletInfo, ApiError> {
    sqlx::query_as("SELECT currency FROM financial_wallets WHERE workspace_id=$1 AND id=$2")
        .bind(ws)
        .bind(id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(ApiError::database)?
        .ok_or_else(|| ApiError::invalid("wallet_id", "unknown_wallet"))
}
async fn treatment(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ws: Uuid,
    wallet: Uuid,
    date: NaiveDate,
    input: Option<&str>,
) -> Result<Option<String>, ApiError> {
    let opening: Option<NaiveDate> = sqlx::query_scalar(
        "SELECT min(business_date) FROM financial_openings WHERE workspace_id=$1 AND wallet_id=$2",
    )
    .bind(ws)
    .bind(wallet)
    .fetch_one(&mut **tx)
    .await
    .map_err(ApiError::database)?;
    let needs = opening.is_some_and(|d| date <= d);
    match (needs, input) {
        (true, Some(v @ ("apply_effect" | "preserve_balance"))) => Ok(Some(v.to_owned())),
        (true, _) => Err(ApiError::invalid(
            "opening_treatment",
            "required_for_opening_date",
        )),
        (false, None) => Ok(None),
        (false, Some("apply_effect" | "preserve_balance")) => Ok(None),
        _ => Err(ApiError::invalid("opening_treatment", "unsupported")),
    }
}

async fn correction_period(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    workspace_id: Uuid,
    old_date: NaiveDate,
    new_date: NaiveDate,
) -> Result<Option<NaiveDate>, ApiError> {
    sqlx::query_scalar(
        "WITH affected AS (SELECT EXISTS(SELECT 1 FROM budget_close_snapshots WHERE workspace_id=$1 AND period_start IN (date_trunc('month',$2::date)::date,date_trunc('month',$3::date)::date)) AS is_closed), current_month AS (SELECT date_trunc('month',now() AT TIME ZONE 'America/Santiago')::date AS period_start) SELECT CASE WHEN NOT affected.is_closed THEN NULL WHEN EXISTS(SELECT 1 FROM budget_close_snapshots c,current_month m WHERE c.workspace_id=$1 AND c.period_start=m.period_start) THEN (SELECT (max(period_start)+interval '1 month')::date FROM budget_close_snapshots WHERE workspace_id=$1) ELSE (SELECT period_start FROM current_month) END FROM affected",
    )
    .bind(workspace_id)
    .bind(old_date)
    .bind(new_date)
    .fetch_one(&mut **tx)
    .await
    .map_err(ApiError::database)
}

async fn insert_transaction(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ws: Uuid,
    id: Uuid,
    r: &TransactionRequest,
    category: Option<Uuid>,
    source: Option<Uuid>,
    revision: i64,
) -> Result<(), ApiError> {
    sqlx::query("INSERT INTO financial_transactions(id,workspace_id,transaction_kind,business_date,category_id,source_expense_id,payee,note,confirmed_clp_amount,conversion_evidence,revision) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)").bind(id).bind(ws).bind(&r.kind).bind(r.business_date.date()).bind(category).bind(source).bind(&r.payee).bind(&r.note).bind(r.confirmed_clp_amount.map(MoneyAmount::decimal)).bind(r.conversion_evidence.clone()).bind(revision).execute(&mut **tx).await.map_err(ApiError::database)?;
    Ok(())
}
async fn insert_effect(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ws: Uuid,
    id: Uuid,
    wallet: Uuid,
    role: &str,
    amount: Decimal,
    treatment: Option<String>,
) -> Result<(), ApiError> {
    sqlx::query("INSERT INTO financial_wallet_effects(id,workspace_id,transaction_id,wallet_id,effect_role,amount,opening_treatment) VALUES($1,$2,$3,$4,$5,$6,$7)").bind(Uuid::new_v4()).bind(ws).bind(id).bind(wallet).bind(role).bind(amount).bind(treatment).execute(&mut **tx).await.map_err(ApiError::database)?;
    Ok(())
}
async fn load_view(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ws: Uuid,
    id: Uuid,
) -> Result<TransactionView, ApiError> {
    let row=sqlx::query_as::<_,StoredTransaction>("SELECT id,transaction_kind,business_date,category_id,source_expense_id,payee,note,confirmed_clp_amount,conversion_evidence,revision FROM financial_transactions WHERE workspace_id=$1 AND id=$2").bind(ws).bind(id).fetch_optional(&mut **tx).await.map_err(ApiError::database)?.ok_or_else(ApiError::not_found)?;
    let effects=sqlx::query_as::<_,StoredEffect>("SELECT e.wallet_id,w.currency,e.effect_role,e.amount,e.opening_treatment FROM financial_wallet_effects e JOIN financial_wallets w ON w.workspace_id=e.workspace_id AND w.id=e.wallet_id WHERE e.workspace_id=$1 AND e.transaction_id=$2 ORDER BY e.effect_role,e.wallet_id").bind(ws).bind(id).fetch_all(&mut **tx).await.map_err(ApiError::database)?;
    view(row, effects)
}
async fn view_from_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ws: Uuid,
    row: StoredTransaction,
) -> Result<TransactionView, ApiError> {
    let effects=sqlx::query_as::<_,StoredEffect>("SELECT e.wallet_id,w.currency,e.effect_role,e.amount,e.opening_treatment FROM financial_wallet_effects e JOIN financial_wallets w ON w.workspace_id=e.workspace_id AND w.id=e.wallet_id WHERE e.workspace_id=$1 AND e.transaction_id=$2 ORDER BY e.effect_role,e.wallet_id").bind(ws).bind(row.id).fetch_all(&mut **tx).await.map_err(ApiError::database)?;
    view(row, effects)
}
fn view(row: StoredTransaction, effects: Vec<StoredEffect>) -> Result<TransactionView, ApiError> {
    let confirmed_clp_amount = row
        .confirmed_clp_amount
        .map(MoneyAmount::new)
        .transpose()
        .map_err(|_| {
            ApiError::bad_request(
                "invalid_stored_amount",
                "Stored transaction amount is outside the supported range.",
            )
        })?;
    let mut mapped = Vec::with_capacity(effects.len());
    for e in effects {
        mapped.push(EffectView {
            wallet_id: e.wallet_id,
            currency: e.currency.trim().to_owned(),
            role: e.effect_role,
            amount: MoneyAmount::new(e.amount).map_err(|_| {
                ApiError::bad_request(
                    "invalid_stored_amount",
                    "Stored transaction amount is outside the supported range.",
                )
            })?,
            opening_treatment: e.opening_treatment,
        });
    }
    Ok(TransactionView {
        id: row.id,
        kind: row.transaction_kind,
        business_date: row.business_date,
        category_id: row.category_id,
        source_expense_id: row.source_expense_id,
        payee: row.payee,
        note: row.note,
        confirmed_clp_amount,
        conversion_evidence: row.conversion_evidence,
        revision: row.revision,
        effects: mapped,
    })
}
