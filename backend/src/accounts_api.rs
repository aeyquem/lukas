use axum::{
    Json, Router,
    body::Bytes,
    extract::{Path, State, rejection::PathRejection},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::get,
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{FromRow, PgConnection};
use uuid::Uuid;

use crate::{
    budget_api::{correction_period_for_source, record_closed_opening_correction},
    financial_values::{BusinessDate, MoneyAmount},
    health::AppState,
    http_error::{ApiError, retry_transaction},
    persistence::{AccountRow, CategoryRow, WalletRow},
    write_guard::{installation_generation, lock_workspace},
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/bootstrap", get(bootstrap))
        .route("/api/accounts", get(list_accounts).post(create_account))
        .route(
            "/api/accounts/{account_id}",
            get(get_account).put(edit_account),
        )
        .route(
            "/api/accounts/{account_id}/openings/{opening_id}",
            axum::routing::put(correct_opening),
        )
}

#[derive(Serialize)]
struct BootstrapResponse {
    workspace_id: Uuid,
    installation_generation: Uuid,
    categories: Vec<CategoryResponse>,
}

#[derive(Serialize)]
struct CategoryResponse {
    id: Uuid,
    slug: String,
    name: String,
    sort_order: i16,
    archived: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateAccountRequest {
    name: String,
    account_type: String,
    base_currency: String,
    budget_eligible: Option<bool>,
    wallets: Vec<CreateWalletRequest>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateWalletRequest {
    currency: String,
    opening_amount: MoneyAmount,
    opening_date: BusinessDate,
    liability_clp_basis: Option<MoneyAmount>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EditAccountRequest {
    name: String,
    expected_revision: i64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CorrectOpeningRequest {
    amount: MoneyAmount,
    expected_revision: i64,
    expected_basis_revision: Option<i64>,
    liability_clp_basis: Option<MoneyAmount>,
    reason: String,
}

#[derive(Serialize)]
struct MutationReceipt {
    resource_id: Uuid,
    revision: i64,
}

#[derive(Serialize)]
struct AccountResponse {
    id: Uuid,
    name: String,
    account_type: String,
    base_currency: String,
    budget_eligible: bool,
    revision: i64,
    wallets: Vec<WalletBalance>,
}

#[derive(Serialize)]
struct WalletBalance {
    id: Uuid,
    currency: String,
    balance: MoneyAmount,
    openings: Vec<OpeningResponse>,
}

#[derive(Serialize)]
struct OpeningResponse {
    id: Uuid,
    amount: MoneyAmount,
    business_date: NaiveDate,
    revision: i64,
    liability_clp_basis: Option<MoneyAmount>,
    liability_basis_provenance: Option<String>,
    liability_basis_revision: Option<i64>,
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

#[derive(FromRow)]
struct OpeningForCorrection {
    revision: i64,
    amount: Decimal,
    business_date: NaiveDate,
    currency: String,
    budget_eligible: bool,
}

#[derive(FromRow)]
struct ExistingLiabilityBasis {
    id: Uuid,
    revision: i64,
    clp_amount: Decimal,
}

async fn bootstrap(State(state): State<AppState>) -> Result<Json<BootstrapResponse>, ApiError> {
    let generation: Uuid =
        sqlx::query_scalar("SELECT installation_generation FROM workspaces WHERE id = $1")
            .bind(state.workspace_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(ApiError::database)?
            .ok_or_else(ApiError::not_found)?;

    let categories = sqlx::query_as::<_, CategoryRow>(
        "SELECT id, workspace_id, slug, name, sort_order, archived, created_at, updated_at \
         FROM expense_categories WHERE workspace_id = $1 ORDER BY sort_order, id",
    )
    .bind(state.workspace_id)
    .fetch_all(&state.pool)
    .await
    .map_err(ApiError::database)?
    .into_iter()
    .map(|category| CategoryResponse {
        id: category.id,
        slug: category.slug,
        name: category.name,
        sort_order: category.sort_order,
        archived: category.archived,
    })
    .collect();

    Ok(Json(BootstrapResponse {
        workspace_id: state.workspace_id,
        installation_generation: generation,
        categories,
    }))
}

async fn list_accounts(
    State(state): State<AppState>,
) -> Result<Json<Vec<AccountResponse>>, ApiError> {
    let mut tx = state.pool.begin().await.map_err(ApiError::database)?;
    sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY")
        .execute(&mut *tx)
        .await
        .map_err(ApiError::database)?;
    let accounts = sqlx::query_as::<_, AccountRow>(
        "SELECT id, workspace_id, name, account_type, base_currency, budget_eligible, \
         revision, created_at, updated_at FROM financial_accounts \
         WHERE workspace_id = $1 ORDER BY name, id",
    )
    .bind(state.workspace_id)
    .fetch_all(&mut *tx)
    .await
    .map_err(ApiError::database)?;

    let mut response = Vec::with_capacity(accounts.len());
    for account in accounts {
        response.push(account_response(&mut tx, state.workspace_id, account).await?);
    }
    tx.commit().await.map_err(ApiError::database)?;
    Ok(Json(response))
}

async fn get_account(
    State(state): State<AppState>,
    path: Result<Path<Uuid>, PathRejection>,
) -> Result<Json<AccountResponse>, ApiError> {
    let Path(account_id) = path.map_err(|_| ApiError::invalid_request_parameter())?;
    let mut tx = state.pool.begin().await.map_err(ApiError::database)?;
    sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY")
        .execute(&mut *tx)
        .await
        .map_err(ApiError::database)?;
    let account = sqlx::query_as::<_, AccountRow>(
        "SELECT id, workspace_id, name, account_type, base_currency, budget_eligible, \
         revision, created_at, updated_at FROM financial_accounts \
         WHERE workspace_id = $1 AND id = $2",
    )
    .bind(state.workspace_id)
    .bind(account_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(ApiError::database)?
    .ok_or_else(ApiError::not_found)?;

    let response = account_response(&mut tx, state.workspace_id, account).await?;
    tx.commit().await.map_err(ApiError::database)?;
    Ok(Json(response))
}

async fn account_response(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    account: AccountRow,
) -> Result<AccountResponse, ApiError> {
    let wallets = sqlx::query_as::<_, WalletRow>(
        "SELECT id, workspace_id, account_id, currency, created_at FROM financial_wallets \
         WHERE workspace_id = $1 AND account_id = $2 ORDER BY currency, id",
    )
    .bind(workspace_id)
    .bind(account.id)
    .fetch_all(&mut *connection)
    .await
    .map_err(ApiError::database)?;

    let mut balances = Vec::with_capacity(wallets.len());
    for wallet in wallets {
        let balance: Decimal = sqlx::query_scalar(
            "SELECT \
             COALESCE((SELECT sum(amount) FROM financial_openings \
                       WHERE workspace_id = $1 AND wallet_id = $2), 0::NUMERIC) + \
             COALESCE((SELECT sum(amount) FROM financial_wallet_effects \
                       WHERE workspace_id = $1 AND wallet_id = $2 \
                         AND opening_treatment IS DISTINCT FROM 'preserve_balance'), 0::NUMERIC)",
        )
        .bind(workspace_id)
        .bind(wallet.id)
        .fetch_one(&mut *connection)
        .await
        .map_err(ApiError::database)?;
        let balance = MoneyAmount::new(balance)
            .map_err(|_| ApiError::invalid("balance", "outside_decimal_envelope"))?;
        balances.push(WalletBalance {
            id: wallet.id,
            currency: wallet.currency,
            balance,
            openings: fetch_opening_responses(connection, workspace_id, wallet.id).await?,
        });
    }

    Ok(AccountResponse {
        id: account.id,
        name: account.name,
        account_type: account.account_type,
        base_currency: account.base_currency,
        budget_eligible: account.budget_eligible,
        revision: account.revision,
        wallets: balances,
    })
}

async fn fetch_opening_responses(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    wallet_id: Uuid,
) -> Result<Vec<OpeningResponse>, ApiError> {
    #[derive(FromRow)]
    struct OpeningRecord {
        id: Uuid,
        amount: Decimal,
        business_date: NaiveDate,
        revision: i64,
        liability_clp_basis: Option<Decimal>,
        liability_basis_provenance: Option<String>,
        liability_basis_revision: Option<i64>,
    }

    let rows = sqlx::query_as::<_, OpeningRecord>(
        "SELECT o.id, o.amount, o.business_date, o.revision, b.clp_amount AS liability_clp_basis, \
         b.provenance AS liability_basis_provenance, b.revision AS liability_basis_revision \
         FROM financial_openings o LEFT JOIN foreign_liability_bases b \
           ON b.workspace_id = o.workspace_id AND b.opening_id = o.id \
         WHERE o.workspace_id = $1 AND o.wallet_id = $2 ORDER BY o.business_date, o.id",
    )
    .bind(workspace_id)
    .bind(wallet_id)
    .fetch_all(&mut *connection)
    .await
    .map_err(ApiError::database)?;

    rows.into_iter()
        .map(|row| {
            Ok(OpeningResponse {
                id: row.id,
                amount: MoneyAmount::new(row.amount)
                    .map_err(|_| ApiError::invalid("opening.amount", "outside_decimal_envelope"))?,
                business_date: row.business_date,
                revision: row.revision,
                liability_clp_basis: row
                    .liability_clp_basis
                    .map(MoneyAmount::new)
                    .transpose()
                    .map_err(|_| {
                        ApiError::invalid("opening.liability_clp_basis", "outside_decimal_envelope")
                    })?,
                liability_basis_provenance: row.liability_basis_provenance,
                liability_basis_revision: row.liability_basis_revision,
            })
        })
        .collect()
}

async fn create_account(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<impl IntoResponse, ApiError> {
    retry_transaction(|| {
        create_account_attempt(State(state.clone()), headers.clone(), body.clone())
    })
    .await
}

async fn create_account_attempt(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<impl IntoResponse, ApiError> {
    let idempotency_key = idempotency_key(&headers)?;
    let generation = installation_generation(&headers)?;
    let route = "/api/accounts";
    let request_hash = Sha256::digest(&body).to_vec();
    let mut tx = state.pool.begin().await.map_err(ApiError::database)?;
    lock_workspace(&mut tx, state.workspace_id, generation).await?;
    if let Some(receipt) = find_receipt(
        &mut tx,
        state.workspace_id,
        idempotency_key,
        "POST",
        route,
        &request_hash,
    )
    .await?
    {
        tx.commit().await.map_err(ApiError::database)?;
        return Ok((
            StatusCode::from_u16(receipt.response_status as u16).unwrap_or(StatusCode::CREATED),
            Json(MutationReceipt {
                resource_id: receipt.resource_id.ok_or_else(ApiError::not_found)?,
                revision: receipt.resource_revision.unwrap_or(1),
            }),
        ));
    }

    let request: CreateAccountRequest = parse_body(&body)?;
    let budget_eligible = validate_create_request(&request)?;
    let account_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO financial_accounts \
         (id, workspace_id, name, account_type, base_currency, budget_eligible) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(account_id)
    .bind(state.workspace_id)
    .bind(request.name.trim())
    .bind(&request.account_type)
    .bind(&request.base_currency)
    .bind(budget_eligible)
    .execute(&mut *tx)
    .await
    .map_err(ApiError::database)?;

    for wallet in request.wallets {
        let wallet_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO financial_wallets (id, workspace_id, account_id, currency) \
             VALUES ($1, $2, $3, $4)",
        )
        .bind(wallet_id)
        .bind(state.workspace_id)
        .bind(account_id)
        .bind(&wallet.currency)
        .execute(&mut *tx)
        .await
        .map_err(ApiError::database)?;

        let opening_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO financial_openings (id, workspace_id, wallet_id, amount, business_date) \
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(opening_id)
        .bind(state.workspace_id)
        .bind(wallet_id)
        .bind(wallet.opening_amount.decimal())
        .bind(wallet.opening_date.date())
        .execute(&mut *tx)
        .await
        .map_err(ApiError::database)?;

        if let Some(basis) = wallet.liability_clp_basis {
            sqlx::query(
                "INSERT INTO foreign_liability_bases \
                 (id, workspace_id, opening_id, clp_amount, provenance) \
                 VALUES ($1, $2, $3, $4, 'owner-confirmed')",
            )
            .bind(Uuid::new_v4())
            .bind(state.workspace_id)
            .bind(opening_id)
            .bind(basis.decimal())
            .execute(&mut *tx)
            .await
            .map_err(ApiError::database)?;
        }
    }

    insert_receipt(
        &mut tx,
        state.workspace_id,
        idempotency_key,
        "POST",
        route,
        &request_hash,
        StatusCode::CREATED,
        account_id,
        1,
    )
    .await?;
    tx.commit().await.map_err(ApiError::database)?;

    Ok((
        StatusCode::CREATED,
        Json(MutationReceipt {
            resource_id: account_id,
            revision: 1,
        }),
    ))
}

async fn edit_account(
    State(state): State<AppState>,
    path: Result<Path<Uuid>, PathRejection>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<impl IntoResponse, ApiError> {
    let Path(account_id) = path.map_err(|_| ApiError::invalid_request_parameter())?;
    retry_transaction(|| {
        edit_account_attempt(
            State(state.clone()),
            Path(account_id),
            headers.clone(),
            body.clone(),
        )
    })
    .await
}

async fn edit_account_attempt(
    State(state): State<AppState>,
    Path(account_id): Path<Uuid>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<impl IntoResponse, ApiError> {
    let idempotency_key = idempotency_key(&headers)?;
    let generation = installation_generation(&headers)?;
    let route = format!("/api/accounts/{account_id}");
    let request_hash = Sha256::digest(&body).to_vec();
    let mut tx = state.pool.begin().await.map_err(ApiError::database)?;
    lock_workspace(&mut tx, state.workspace_id, generation).await?;
    if let Some(receipt) = find_receipt(
        &mut tx,
        state.workspace_id,
        idempotency_key,
        "PUT",
        &route,
        &request_hash,
    )
    .await?
    {
        tx.commit().await.map_err(ApiError::database)?;
        return Ok((
            StatusCode::from_u16(receipt.response_status as u16).unwrap_or(StatusCode::OK),
            Json(MutationReceipt {
                resource_id: receipt.resource_id.ok_or_else(ApiError::not_found)?,
                revision: receipt.resource_revision.unwrap_or(1),
            }),
        ));
    }

    let request: EditAccountRequest = parse_body(&body)?;
    let name = request.name.trim();
    if name.is_empty() {
        return Err(ApiError::invalid("name", "required"));
    }

    let current_revision: Option<i64> = sqlx::query_scalar(
        "SELECT revision FROM financial_accounts WHERE workspace_id = $1 AND id = $2 FOR UPDATE",
    )
    .bind(state.workspace_id)
    .bind(account_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(ApiError::database)?;
    let current_revision = current_revision.ok_or_else(ApiError::not_found)?;
    if request.expected_revision != current_revision {
        return Err(ApiError::stale_revision());
    }

    let next_revision = current_revision + 1;
    sqlx::query(
        "UPDATE financial_accounts SET name = $1, revision = $2, updated_at = now() \
         WHERE workspace_id = $3 AND id = $4 AND revision = $5",
    )
    .bind(name)
    .bind(next_revision)
    .bind(state.workspace_id)
    .bind(account_id)
    .bind(current_revision)
    .execute(&mut *tx)
    .await
    .map_err(ApiError::database)?;

    insert_receipt(
        &mut tx,
        state.workspace_id,
        idempotency_key,
        "PUT",
        &route,
        &request_hash,
        StatusCode::OK,
        account_id,
        next_revision,
    )
    .await?;
    tx.commit().await.map_err(ApiError::database)?;
    Ok((
        StatusCode::OK,
        Json(MutationReceipt {
            resource_id: account_id,
            revision: next_revision,
        }),
    ))
}

async fn correct_opening(
    State(state): State<AppState>,
    path: Result<Path<(Uuid, Uuid)>, PathRejection>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<impl IntoResponse, ApiError> {
    let Path((account_id, opening_id)) = path.map_err(|_| ApiError::invalid_request_parameter())?;
    retry_transaction(|| {
        correct_opening_attempt(
            State(state.clone()),
            Path((account_id, opening_id)),
            headers.clone(),
            body.clone(),
        )
    })
    .await
}

async fn correct_opening_attempt(
    State(state): State<AppState>,
    Path((account_id, opening_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<impl IntoResponse, ApiError> {
    let idempotency_key = idempotency_key(&headers)?;
    let generation = installation_generation(&headers)?;
    let route = format!("/api/accounts/{account_id}/openings/{opening_id}");
    let request_hash = Sha256::digest(&body).to_vec();
    let mut tx = state.pool.begin().await.map_err(ApiError::database)?;
    lock_workspace(&mut tx, state.workspace_id, generation).await?;
    if let Some(receipt) = find_receipt(
        &mut tx,
        state.workspace_id,
        idempotency_key,
        "PUT",
        &route,
        &request_hash,
    )
    .await?
    {
        tx.commit().await.map_err(ApiError::database)?;
        return Ok((
            StatusCode::from_u16(receipt.response_status as u16).unwrap_or(StatusCode::OK),
            Json(MutationReceipt {
                resource_id: receipt.resource_id.ok_or_else(ApiError::not_found)?,
                revision: receipt.resource_revision.unwrap_or(1),
            }),
        ));
    }

    let request: CorrectOpeningRequest = parse_body(&body)?;
    if request.reason.trim().is_empty() {
        return Err(ApiError::invalid("reason", "required"));
    }

    let opening = sqlx::query_as::<_, OpeningForCorrection>(
        "SELECT o.revision, o.amount, o.business_date, w.currency, a.budget_eligible \
         FROM financial_openings o \
         JOIN financial_wallets w ON w.workspace_id = o.workspace_id AND w.id = o.wallet_id \
         JOIN financial_accounts a ON a.workspace_id = w.workspace_id AND a.id = w.account_id \
         WHERE o.workspace_id = $1 AND a.id = $2 AND o.id = $3 FOR UPDATE OF o",
    )
    .bind(state.workspace_id)
    .bind(account_id)
    .bind(opening_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(ApiError::database)?
    .ok_or_else(ApiError::not_found)?;
    if opening.revision != request.expected_revision {
        return Err(ApiError::stale_revision());
    }

    let existing_basis = sqlx::query_as::<_, ExistingLiabilityBasis>(
        "SELECT id, revision, clp_amount FROM foreign_liability_bases \
         WHERE workspace_id = $1 AND opening_id = $2 FOR UPDATE",
    )
    .bind(state.workspace_id)
    .bind(opening_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(ApiError::database)?;

    let new_amount = request.amount.decimal();
    let target_basis = request
        .liability_clp_basis
        .map(MoneyAmount::decimal)
        .or_else(|| existing_basis.as_ref().map(|basis| basis.clp_amount));
    let is_foreign_liability = opening.budget_eligible
        && opening.currency != "CLP"
        && new_amount.is_sign_negative()
        && !new_amount.is_zero();
    let basis_is_paired = existing_basis.is_some();
    let basis_required = is_foreign_liability || basis_is_paired;
    if basis_required && request.liability_clp_basis.is_none() {
        return Err(ApiError::invalid(
            "liability_clp_basis",
            "required_for_principal_correction",
        ));
    }
    if request.liability_clp_basis.is_some() && !is_foreign_liability && !basis_is_paired {
        return Err(ApiError::invalid(
            "liability_clp_basis",
            "only_valid_for_foreign_liability",
        ));
    }
    if let Some(basis) = request.liability_clp_basis {
        let basis_value = basis.decimal();
        let sign_matches = if new_amount.is_zero() {
            basis_value.is_zero()
        } else {
            basis_value.is_sign_negative() == new_amount.is_sign_negative()
                && !basis_value.is_zero()
        };
        if !sign_matches {
            return Err(ApiError::invalid(
                "liability_clp_basis",
                "must_match_principal_sign",
            ));
        }
    }

    if let Some(existing_basis) = &existing_basis {
        if request.expected_basis_revision != Some(existing_basis.revision) {
            return Err(ApiError::stale_revision());
        }
    } else if request.expected_basis_revision.is_some() {
        return Err(ApiError::stale_revision());
    }

    let correction_period = correction_period_for_source(
        &mut tx,
        state.workspace_id,
        opening.business_date,
        opening.business_date,
    )
    .await?;

    let next_revision = opening.revision + 1;
    sqlx::query(
        "UPDATE financial_openings SET amount = $1, revision = $2 \
         WHERE workspace_id = $3 AND id = $4 AND revision = $5",
    )
    .bind(new_amount)
    .bind(next_revision)
    .bind(state.workspace_id)
    .bind(opening_id)
    .bind(opening.revision)
    .execute(&mut *tx)
    .await
    .map_err(ApiError::database)?;

    let correction_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO financial_corrections \
         (id, workspace_id, record_kind, opening_id, prior_revision, new_revision, reason, correction_period, opening_target_amount, opening_target_liability_basis) \
         VALUES ($1, $2, 'opening', $3, $4, $5, $6, $7, $8, $9)",
    )
    .bind(correction_id)
    .bind(state.workspace_id)
    .bind(opening_id)
    .bind(opening.revision)
    .bind(next_revision)
    .bind(request.reason.trim())
    .bind(correction_period)
    .bind(new_amount)
    .bind(target_basis)
    .execute(&mut *tx)
    .await
    .map_err(ApiError::database)?;

    if let Some(basis) = request.liability_clp_basis {
        if let Some(existing_basis) = existing_basis.as_ref() {
            let next_basis_revision = existing_basis.revision + 1;
            sqlx::query(
                "UPDATE foreign_liability_bases SET clp_amount = $1, provenance = 'owner-confirmed', revision = $2 \
                 WHERE workspace_id = $3 AND id = $4 AND revision = $5",
            )
            .bind(basis.decimal())
            .bind(next_basis_revision)
            .bind(state.workspace_id)
            .bind(existing_basis.id)
            .bind(existing_basis.revision)
            .execute(&mut *tx)
            .await
            .map_err(ApiError::database)?;
            sqlx::query(
                "INSERT INTO financial_corrections \
                 (id, workspace_id, record_kind, liability_basis_id, prior_revision, new_revision, reason) \
                 VALUES ($1, $2, 'liability_basis', $3, $4, $5, $6)",
            )
            .bind(Uuid::new_v4())
            .bind(state.workspace_id)
            .bind(existing_basis.id)
            .bind(existing_basis.revision)
            .bind(next_basis_revision)
            .bind(request.reason.trim())
            .execute(&mut *tx)
            .await
            .map_err(ApiError::database)?;
        } else {
            sqlx::query(
                "INSERT INTO foreign_liability_bases \
                 (id, workspace_id, opening_id, clp_amount, provenance) \
                 VALUES ($1, $2, $3, $4, 'owner-confirmed')",
            )
            .bind(Uuid::new_v4())
            .bind(state.workspace_id)
            .bind(opening_id)
            .bind(basis.decimal())
            .execute(&mut *tx)
            .await
            .map_err(ApiError::database)?;
        }
    }

    if let Some(correction_period) = correction_period {
        record_closed_opening_correction(
            &mut tx,
            state.workspace_id,
            correction_period,
            correction_id,
            opening_id,
            opening.business_date,
            &opening.currency,
            opening.budget_eligible,
            opening.amount,
            existing_basis.as_ref().map(|basis| basis.clp_amount),
            new_amount,
            target_basis,
        )
        .await?;
    }

    insert_receipt(
        &mut tx,
        state.workspace_id,
        idempotency_key,
        "PUT",
        &route,
        &request_hash,
        StatusCode::OK,
        opening_id,
        next_revision,
    )
    .await?;
    tx.commit().await.map_err(ApiError::database)?;

    Ok((
        StatusCode::OK,
        Json(MutationReceipt {
            resource_id: opening_id,
            revision: next_revision,
        }),
    ))
}

fn validate_create_request(request: &CreateAccountRequest) -> Result<bool, ApiError> {
    if request.name.trim().is_empty() {
        return Err(ApiError::invalid("name", "required"));
    }
    if !matches!(
        request.account_type.as_str(),
        "bank" | "cash" | "savings" | "card" | "investment"
    ) {
        return Err(ApiError::invalid("account_type", "unsupported_value"));
    }
    validate_currency("base_currency", &request.base_currency)?;
    if request.wallets.is_empty() {
        return Err(ApiError::invalid("wallets", "at_least_one_required"));
    }
    if !matches!(request.account_type.as_str(), "card" | "cash") && request.wallets.len() > 1 {
        return Err(ApiError::invalid(
            "wallets",
            "multiple_currencies_unsupported_for_account_type",
        ));
    }
    let eligible = match request.budget_eligible {
        Some(value) => value,
        None if request.account_type == "investment" => false,
        None => return Err(ApiError::invalid("budget_eligible", "required")),
    };

    let mut currencies = std::collections::HashSet::new();
    for (index, wallet) in request.wallets.iter().enumerate() {
        validate_currency(&format!("wallets[{index}].currency"), &wallet.currency)?;
        if !currencies.insert(wallet.currency.as_str()) {
            return Err(ApiError::invalid(
                format!("wallets[{index}].currency"),
                "duplicate_currency",
            ));
        }

        let is_foreign_liability = eligible
            && wallet.currency != "CLP"
            && wallet.opening_amount.decimal().is_sign_negative()
            && !wallet.opening_amount.decimal().is_zero();
        if is_foreign_liability && wallet.liability_clp_basis.is_none() {
            return Err(ApiError::invalid(
                format!("wallets[{index}].liability_clp_basis"),
                "required_for_foreign_liability",
            ));
        }
        if is_foreign_liability
            && wallet.liability_clp_basis.is_some_and(|basis| {
                basis.decimal().is_zero() || !basis.decimal().is_sign_negative()
            })
        {
            return Err(ApiError::invalid(
                format!("wallets[{index}].liability_clp_basis"),
                "must_match_liability_sign",
            ));
        }
        if wallet.liability_clp_basis.is_some() && !is_foreign_liability {
            return Err(ApiError::invalid(
                format!("wallets[{index}].liability_clp_basis"),
                "only_valid_for_eligible_foreign_liability",
            ));
        }
    }

    Ok(eligible)
}

fn validate_currency(field: &str, currency: &str) -> Result<(), ApiError> {
    if currency.len() != 3 || !currency.bytes().all(|byte| byte.is_ascii_uppercase()) {
        return Err(ApiError::invalid(
            field.to_owned(),
            "must_be_iso_style_code",
        ));
    }
    Ok(())
}

fn parse_body<T: for<'de> Deserialize<'de>>(body: &[u8]) -> Result<T, ApiError> {
    serde_json::from_slice(body)
        .map_err(|_| ApiError::bad_request("malformed_json", "Request body must be valid JSON."))
}

fn idempotency_key(headers: &HeaderMap) -> Result<Uuid, ApiError> {
    let value = headers
        .get("Idempotency-Key")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| {
            ApiError::bad_request("idempotency_key_required", "Provide an idempotency key.")
        })?;
    Uuid::parse_str(value).map_err(|_| {
        ApiError::bad_request("invalid_idempotency_key", "Idempotency key must be a UUID.")
    })
}

async fn find_receipt(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    workspace_id: Uuid,
    idempotency_key: Uuid,
    method: &str,
    route: &str,
    request_hash: &[u8],
) -> Result<Option<StoredReceipt>, ApiError> {
    let receipt = sqlx::query_as::<_, StoredReceipt>(
        "SELECT http_method, route, request_hash, response_status, resource_id, resource_revision \
         FROM financial_idempotency_receipts WHERE workspace_id = $1 AND idempotency_key = $2",
    )
    .bind(workspace_id)
    .bind(idempotency_key)
    .fetch_optional(&mut **tx)
    .await
    .map_err(ApiError::database)?;

    if let Some(receipt) = &receipt
        && (receipt.http_method != method
            || receipt.route != route
            || receipt.request_hash != request_hash)
    {
        return Err(ApiError::idempotency_conflict());
    }
    Ok(receipt)
}

async fn insert_receipt(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    workspace_id: Uuid,
    idempotency_key: Uuid,
    method: &str,
    route: &str,
    request_hash: &[u8],
    status: StatusCode,
    resource_id: Uuid,
    revision: i64,
) -> Result<(), ApiError> {
    sqlx::query(
        "INSERT INTO financial_idempotency_receipts \
         (workspace_id, idempotency_key, http_method, route, request_hash, response_status, resource_id, resource_revision) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(workspace_id)
    .bind(idempotency_key)
    .bind(method)
    .bind(route)
    .bind(request_hash)
    .bind(status.as_u16() as i16)
    .bind(resource_id)
    .bind(revision)
    .execute(&mut **tx)
    .await
    .map_err(ApiError::database)?;
    Ok(())
}
