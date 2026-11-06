use std::time::Duration;

use axum::{
    body::{Body, to_bytes},
    http::Request,
};
use chrono::Datelike;
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

use lukas_api::{config::Config, health::router, workspace::bootstrap};

fn config() -> Config {
    Config {
        database_url: "postgres://lukas:lukas@localhost/lukas".into(),
        canonical_origin: "http://localhost:4173".into(),
        bind_address: "127.0.0.1:8080".parse().unwrap(),
        max_connections: 5,
        connect_timeout: Duration::from_secs(5),
        acquire_timeout: Duration::from_secs(1),
        database_startup_timeout: Duration::from_secs(30),
        shutdown_timeout: Duration::from_secs(10),
    }
}

async fn json_response(response: axum::response::Response) -> (axum::http::StatusCode, Value) {
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (status, serde_json::from_slice(&bytes).unwrap())
}
async fn send(
    pool: PgPool,
    workspace: Uuid,
    method: &str,
    uri: &str,
    key: Option<Uuid>,
    body: Option<String>,
) -> axum::response::Response {
    let generation: String =
        sqlx::query_scalar("SELECT installation_generation::text FROM workspaces WHERE id=$1")
            .bind(workspace)
            .fetch_one(&pool)
            .await
            .unwrap();
    send_with_generation(pool, workspace, method, uri, key, body, generation).await
}

async fn send_with_generation(
    pool: PgPool,
    workspace: Uuid,
    method: &str,
    uri: &str,
    key: Option<Uuid>,
    body: Option<String>,
    generation: String,
) -> axum::response::Response {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("Origin", "http://localhost:4173")
        .header("Installation-Generation", generation);
    if let Some(key) = key {
        builder = builder.header("Idempotency-Key", key.to_string());
    }
    if body.is_some() {
        builder = builder.header("Content-Type", "application/json");
    }
    router(pool, workspace, config())
        .oneshot(
            builder
                .body(body.map(Body::from).unwrap_or_else(Body::empty))
                .unwrap(),
        )
        .await
        .unwrap()
}

#[sqlx::test]
async fn stale_generation_rejects_transaction_retry_before_receipt_lookup(pool: PgPool) {
    let (workspace, category, wallet, _, _) = setup(&pool).await;
    let key = Uuid::new_v4();
    let body = json!({"kind":"expense","business_date":"2026-02-02","wallet_id":wallet,"amount":"25","category_id":category}).to_string();
    let generation: String =
        sqlx::query_scalar("SELECT installation_generation::text FROM workspaces WHERE id=$1")
            .bind(workspace)
            .fetch_one(&pool)
            .await
            .unwrap();

    let (created_status, receipt) = json_response(
        send_with_generation(
            pool.clone(),
            workspace,
            "POST",
            "/api/transactions",
            Some(key),
            Some(body.clone()),
            generation.clone(),
        )
        .await,
    )
    .await;
    assert_eq!(created_status, axum::http::StatusCode::CREATED);

    lukas_api::database::rotate_installation_generation(&pool, workspace)
        .await
        .unwrap();

    let (retry_status, retry) = json_response(
        send_with_generation(
            pool.clone(),
            workspace,
            "POST",
            "/api/transactions",
            Some(key),
            Some(body),
            generation,
        )
        .await,
    )
    .await;
    assert_eq!(retry_status, axum::http::StatusCode::CONFLICT);
    assert_eq!(retry["error"]["code"], "installation_generation_mismatch");

    let transaction_id = receipt["resource_id"].as_str().unwrap();
    let transaction_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM financial_transactions WHERE workspace_id=$1")
            .bind(workspace)
            .fetch_one(&pool)
            .await
            .unwrap();
    let effect_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM financial_wallet_effects WHERE transaction_id=$1")
            .bind(Uuid::parse_str(transaction_id).unwrap())
            .fetch_one(&pool)
            .await
            .unwrap();
    let receipt_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM financial_idempotency_receipts WHERE workspace_id=$1 AND idempotency_key=$2",
    )
    .bind(workspace)
    .bind(key)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(transaction_count, 1);
    assert_eq!(effect_count, 1);
    assert_eq!(receipt_count, 1);
}

#[sqlx::test]
async fn stale_generation_and_exact_edit_replay_do_not_write_fx_observations(pool: PgPool) {
    let (ws, _category, clp, usd, _clp2) = setup(&pool).await;
    let generation: String =
        sqlx::query_scalar("SELECT installation_generation::text FROM workspaces WHERE id=$1")
            .bind(ws)
            .fetch_one(&pool)
            .await
            .unwrap();
    let initial = json!({
        "kind": "transfer",
        "business_date": "2026-02-02",
        "source_wallet_id": clp,
        "source_amount": "100",
        "destination_wallet_id": usd,
        "destination_amount": "1",
        "conversion_evidence": {"source": "statement", "rate": "80"}
    })
    .to_string();
    let (created_status, receipt) = json_response(
        send_with_generation(
            pool.clone(),
            ws,
            "POST",
            "/api/transactions",
            Some(Uuid::new_v4()),
            Some(initial),
            generation.clone(),
        )
        .await,
    )
    .await;
    assert_eq!(created_status, axum::http::StatusCode::CREATED);
    let transaction_id = receipt["resource_id"].as_str().unwrap();
    let body = json!({
        "kind": "transfer",
        "business_date": "2026-02-03",
        "source_wallet_id": clp,
        "source_amount": "100",
        "destination_wallet_id": usd,
        "destination_amount": "1",
        "conversion_evidence": {"source": "statement", "rate": "80"},
        "expected_revision": 1,
        "reason": "Correct transfer date"
    })
    .to_string();
    let key = Uuid::new_v4();
    let before_rotation: i64 =
        sqlx::query_scalar("SELECT count(*) FROM daily_fx_observations WHERE workspace_id=$1")
            .bind(ws)
            .fetch_one(&pool)
            .await
            .unwrap();
    lukas_api::database::rotate_installation_generation(&pool, ws)
        .await
        .unwrap();
    let (stale_status, stale) = json_response(
        send_with_generation(
            pool.clone(),
            ws,
            "PUT",
            &format!("/api/transactions/{transaction_id}"),
            Some(key),
            Some(body.clone()),
            generation,
        )
        .await,
    )
    .await;
    assert_eq!(stale_status, axum::http::StatusCode::CONFLICT);
    assert_eq!(stale["error"]["code"], "installation_generation_mismatch");
    let after_stale: i64 =
        sqlx::query_scalar("SELECT count(*) FROM daily_fx_observations WHERE workspace_id=$1")
            .bind(ws)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(after_stale, before_rotation);

    let current_generation: String =
        sqlx::query_scalar("SELECT installation_generation::text FROM workspaces WHERE id=$1")
            .bind(ws)
            .fetch_one(&pool)
            .await
            .unwrap();
    let (edit_status, edit_receipt) = json_response(
        send_with_generation(
            pool.clone(),
            ws,
            "PUT",
            &format!("/api/transactions/{transaction_id}"),
            Some(key),
            Some(body.clone()),
            current_generation.clone(),
        )
        .await,
    )
    .await;
    assert_eq!(edit_status, axum::http::StatusCode::OK);
    let after_edit: i64 =
        sqlx::query_scalar("SELECT count(*) FROM daily_fx_observations WHERE workspace_id=$1")
            .bind(ws)
            .fetch_one(&pool)
            .await
            .unwrap();
    let (retry_status, retry_receipt) = json_response(
        send_with_generation(
            pool.clone(),
            ws,
            "PUT",
            &format!("/api/transactions/{transaction_id}"),
            Some(key),
            Some(body),
            current_generation,
        )
        .await,
    )
    .await;
    assert_eq!(retry_status, edit_status);
    assert_eq!(retry_receipt, edit_receipt);
    let after_retry: i64 =
        sqlx::query_scalar("SELECT count(*) FROM daily_fx_observations WHERE workspace_id=$1")
            .bind(ws)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(after_retry, after_edit);
}

async fn setup(pool: &PgPool) -> (Uuid, Uuid, Uuid, Uuid, Uuid) {
    let workspace = bootstrap(pool).await.unwrap();
    let (_status, boot) =
        json_response(send(pool.clone(), workspace, "GET", "/api/bootstrap", None, None).await)
            .await;
    let category = boot["categories"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["slug"] == "alimentacion")
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    let body=json!({"name":"Travel card","account_type":"card","base_currency":"CLP","budget_eligible":true,"wallets":[{"currency":"CLP","opening_amount":"1000","opening_date":"2026-01-01"},{"currency":"USD","opening_amount":"0","opening_date":"2026-01-01"}]}).to_string();
    let (status, account) = json_response(
        send(
            pool.clone(),
            workspace,
            "POST",
            "/api/accounts",
            Some(Uuid::new_v4()),
            Some(body),
        )
        .await,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    let (_status, account) = json_response(
        send(
            pool.clone(),
            workspace,
            "GET",
            &format!("/api/accounts/{}", account["resource_id"].as_str().unwrap()),
            None,
            None,
        )
        .await,
    )
    .await;
    let wallets = account["wallets"].as_array().unwrap();
    let clp = wallets.iter().find(|w| w["currency"] == "CLP").unwrap()["id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    let usd = wallets.iter().find(|w| w["currency"] == "USD").unwrap()["id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    let second=json!({"name":"Second","account_type":"cash","base_currency":"CLP","budget_eligible":true,"wallets":[{"currency":"CLP","opening_amount":"0","opening_date":"2026-01-01"}]}).to_string();
    let (status, second) = json_response(
        send(
            pool.clone(),
            workspace,
            "POST",
            "/api/accounts",
            Some(Uuid::new_v4()),
            Some(second),
        )
        .await,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    let (_status, second) = json_response(
        send(
            pool.clone(),
            workspace,
            "GET",
            &format!("/api/accounts/{}", second["resource_id"].as_str().unwrap()),
            None,
            None,
        )
        .await,
    )
    .await;
    let clp2 = second["wallets"][0]["id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    (workspace, category, clp, usd, clp2)
}

#[sqlx::test]
async fn transfers_preserve_linked_effects_and_retries_are_exact(pool: PgPool) {
    let (ws, _category, clp, usd, clp2) = setup(&pool).await;
    let body=json!({"kind":"transfer","business_date":"2026-02-02","source_wallet_id":clp,"source_amount":"100","destination_wallet_id":clp2,"destination_amount":"99"}).to_string();
    let (status, _) = json_response(
        send(
            pool.clone(),
            ws,
            "POST",
            "/api/transactions",
            Some(Uuid::new_v4()),
            Some(body),
        )
        .await,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::UNPROCESSABLE_ENTITY);
    let same=json!({"kind":"transfer","business_date":"2026-02-02","source_wallet_id":clp,"source_amount":"100","destination_wallet_id":clp2,"destination_amount":"100"}).to_string();
    let (status, same_receipt) = json_response(
        send(
            pool.clone(),
            ws,
            "POST",
            "/api/transactions",
            Some(Uuid::new_v4()),
            Some(same),
        )
        .await,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    let (_status, same_view) = json_response(
        send(
            pool.clone(),
            ws,
            "GET",
            &format!(
                "/api/transactions/{}",
                same_receipt["resource_id"].as_str().unwrap()
            ),
            None,
            None,
        )
        .await,
    )
    .await;
    assert_eq!(same_view["effects"].as_array().unwrap().len(), 2);
    let body=json!({"kind":"transfer","business_date":"2026-02-02","source_wallet_id":clp,"source_amount":"100","destination_wallet_id":usd,"destination_amount":"1.25","conversion_evidence":{"source":"statement","rate":"80"}}).to_string();
    let key = Uuid::new_v4();
    let (status, receipt) = json_response(
        send(
            pool.clone(),
            ws,
            "POST",
            "/api/transactions",
            Some(key),
            Some(body.clone()),
        )
        .await,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    let (retry_status, retry) = json_response(
        send(
            pool.clone(),
            ws,
            "POST",
            "/api/transactions",
            Some(key),
            Some(body),
        )
        .await,
    )
    .await;
    assert_eq!(retry_status, status);
    assert_eq!(retry, receipt);
    let changed = json!({"kind":"transfer","business_date":"2026-02-02","source_wallet_id":clp,"source_amount":"101","destination_wallet_id":usd,"destination_amount":"1.25","conversion_evidence":{"source":"statement","rate":"80"}}).to_string();
    let (changed_status, _) = json_response(
        send(
            pool.clone(),
            ws,
            "POST",
            "/api/transactions",
            Some(key),
            Some(changed),
        )
        .await,
    )
    .await;
    assert_eq!(changed_status, axum::http::StatusCode::CONFLICT);
    let (_status, view) = json_response(
        send(
            pool.clone(),
            ws,
            "GET",
            &format!(
                "/api/transactions/{}",
                receipt["resource_id"].as_str().unwrap()
            ),
            None,
            None,
        )
        .await,
    )
    .await;
    assert_eq!(view["kind"], "transfer");
    assert_eq!(view["effects"].as_array().unwrap().len(), 2);
}

#[sqlx::test]
async fn opening_treatment_refund_correction_and_pagination(pool: PgPool) {
    let (ws, category, clp, _usd, _clp2) = setup(&pool).await;
    let missing=json!({"kind":"expense","business_date":"2025-12-31","wallet_id":clp,"amount":"25","category_id":category}).to_string();
    let (status, _) = json_response(
        send(
            pool.clone(),
            ws,
            "POST",
            "/api/transactions",
            Some(Uuid::new_v4()),
            Some(missing),
        )
        .await,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::UNPROCESSABLE_ENTITY);
    let same_day=json!({"kind":"expense","business_date":"2026-01-01","wallet_id":clp,"amount":"5","category_id":category}).to_string();
    let (same_day_status, _) = json_response(
        send(
            pool.clone(),
            ws,
            "POST",
            "/api/transactions",
            Some(Uuid::new_v4()),
            Some(same_day),
        )
        .await,
    )
    .await;
    assert_eq!(
        same_day_status,
        axum::http::StatusCode::UNPROCESSABLE_ENTITY
    );
    let expense=json!({"kind":"expense","business_date":"2025-12-31","wallet_id":clp,"amount":"25","category_id":category,"opening_treatment":"preserve_balance"}).to_string();
    let (status, expense) = json_response(
        send(
            pool.clone(),
            ws,
            "POST",
            "/api/transactions",
            Some(Uuid::new_v4()),
            Some(expense),
        )
        .await,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    let refund=json!({"kind":"refund","business_date":"2026-02-03","wallet_id":clp,"amount":"10","source_expense_id":expense["resource_id"]}).to_string();
    let (status, refund) = json_response(
        send(
            pool.clone(),
            ws,
            "POST",
            "/api/transactions",
            Some(Uuid::new_v4()),
            Some(refund),
        )
        .await,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    let edit=json!({"kind":"refund","business_date":"2026-02-03","wallet_id":clp,"amount":"12","source_expense_id":expense["resource_id"],"expected_revision":1,"reason":"Corrected statement amount"}).to_string();
    let (status, edited) = json_response(
        send(
            pool.clone(),
            ws,
            "PUT",
            &format!(
                "/api/transactions/{}",
                refund["resource_id"].as_str().unwrap()
            ),
            Some(Uuid::new_v4()),
            Some(edit),
        )
        .await,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(edited["revision"], 2);
    let clear_source_category=json!({"kind":"expense","business_date":"2025-12-31","wallet_id":clp,"amount":"25","opening_treatment":"preserve_balance","expected_revision":1,"reason":"Clear category after review"}).to_string();
    let (source_edit_status, _) = json_response(
        send(
            pool.clone(),
            ws,
            "PUT",
            &format!(
                "/api/transactions/{}",
                expense["resource_id"].as_str().unwrap()
            ),
            Some(Uuid::new_v4()),
            Some(clear_source_category),
        )
        .await,
    )
    .await;
    assert_eq!(source_edit_status, axum::http::StatusCode::OK);
    let (_status, updated_refund) = json_response(
        send(
            pool.clone(),
            ws,
            "GET",
            &format!(
                "/api/transactions/{}",
                refund["resource_id"].as_str().unwrap()
            ),
            None,
            None,
        )
        .await,
    )
    .await;
    assert!(updated_refund["category_id"].is_null());
    assert_eq!(updated_refund["revision"], 3);
    let (_status, page) = json_response(
        send(
            pool.clone(),
            ws,
            "GET",
            "/api/transactions?page_size=1",
            None,
            None,
        )
        .await,
    )
    .await;
    assert_eq!(page["items"].as_array().unwrap().len(), 1);
    assert!(!page["next"].is_null());
}

#[sqlx::test]
async fn closed_expense_category_correction_revises_linked_refund_activity_effects(pool: PgPool) {
    let (ws, category, wallet, _usd, _clp2) = setup(&pool).await;
    let other_category: Uuid = sqlx::query_scalar(
        "SELECT id FROM expense_categories WHERE workspace_id=$1 AND id<>$2 ORDER BY sort_order,id LIMIT 1",
    )
    .bind(ws)
    .bind(category)
    .fetch_one(&pool)
    .await
    .unwrap();
    let third_category: Uuid = sqlx::query_scalar(
        "SELECT id FROM expense_categories WHERE workspace_id=$1 AND id NOT IN ($2,$3) ORDER BY sort_order,id LIMIT 1",
    )
    .bind(ws)
    .bind(category)
    .bind(other_category)
    .fetch_one(&pool)
    .await
    .unwrap();
    let expense_body = json!({
        "kind": "expense",
        "business_date": "2026-01-10",
        "wallet_id": wallet,
        "amount": "25",
        "category_id": category
    })
    .to_string();
    let (expense_status, expense) = json_response(
        send(
            pool.clone(),
            ws,
            "POST",
            "/api/transactions",
            Some(Uuid::new_v4()),
            Some(expense_body),
        )
        .await,
    )
    .await;
    assert_eq!(expense_status, axum::http::StatusCode::CREATED);
    let refund_body = json!({
        "kind": "refund",
        "business_date": "2026-01-15",
        "wallet_id": wallet,
        "amount": "10",
        "source_expense_id": expense["resource_id"]
    })
    .to_string();
    let (refund_status, refund) = json_response(
        send(
            pool.clone(),
            ws,
            "POST",
            "/api/transactions",
            Some(Uuid::new_v4()),
            Some(refund_body),
        )
        .await,
    )
    .await;
    assert_eq!(refund_status, axum::http::StatusCode::CREATED);

    let (preview_status, preview) = json_response(
        send(
            pool.clone(),
            ws,
            "GET",
            "/api/budgets/2026-01/close-preview",
            None,
            None,
        )
        .await,
    )
    .await;
    assert_eq!(preview_status, axum::http::StatusCode::OK);
    assert_eq!(preview["close"]["eligible"], true);
    let categories = preview["close"]["categories"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| {
            json!({
                "category_id": item["category_id"],
                "disposition": "carry"
            })
        })
        .collect::<Vec<_>>();
    let close_body = json!({
        "expected_period_revision": preview["close"]["expected_period_revision"],
        "categories": categories
    })
    .to_string();
    let (close_status, _) = json_response(
        send(
            pool.clone(),
            ws,
            "POST",
            "/api/budgets/2026-01/close",
            Some(Uuid::new_v4()),
            Some(close_body),
        )
        .await,
    )
    .await;
    assert_eq!(close_status, axum::http::StatusCode::CREATED);
    let (_, closed_before) =
        json_response(send(pool.clone(), ws, "GET", "/api/budgets/2026-01", None, None).await)
            .await;

    let edit = json!({
        "kind": "expense",
        "business_date": "2026-01-10",
        "wallet_id": wallet,
        "amount": "25",
        "category_id": other_category,
        "expected_revision": 1,
        "reason": "Reclassify expense and linked refund"
    })
    .to_string();
    let (edit_status, _) = json_response(
        send(
            pool.clone(),
            ws,
            "PUT",
            &format!(
                "/api/transactions/{}",
                expense["resource_id"].as_str().unwrap()
            ),
            Some(Uuid::new_v4()),
            Some(edit),
        )
        .await,
    )
    .await;
    assert_eq!(edit_status, axum::http::StatusCode::OK);
    let (_, updated_refund) = json_response(
        send(
            pool.clone(),
            ws,
            "GET",
            &format!(
                "/api/transactions/{}",
                refund["resource_id"].as_str().unwrap()
            ),
            None,
            None,
        )
        .await,
    )
    .await;
    assert_eq!(updated_refund["category_id"], other_category.to_string());

    let second_edit = json!({
        "kind": "expense",
        "business_date": "2026-01-10",
        "wallet_id": wallet,
        "amount": "25",
        "category_id": third_category,
        "expected_revision": 2,
        "reason": "Move reclassified expense again"
    })
    .to_string();
    let (second_edit_status, _) = json_response(
        send(
            pool.clone(),
            ws,
            "PUT",
            &format!(
                "/api/transactions/{}",
                expense["resource_id"].as_str().unwrap()
            ),
            Some(Uuid::new_v4()),
            Some(second_edit),
        )
        .await,
    )
    .await;
    assert_eq!(second_edit_status, axum::http::StatusCode::OK);
    let (_, moved_refund) = json_response(
        send(
            pool.clone(),
            ws,
            "GET",
            &format!(
                "/api/transactions/{}",
                refund["resource_id"].as_str().unwrap()
            ),
            None,
            None,
        )
        .await,
    )
    .await;
    assert_eq!(moved_refund["category_id"], third_category.to_string());

    let (_, closed_after) =
        json_response(send(pool.clone(), ws, "GET", "/api/budgets/2026-01", None, None).await)
            .await;
    assert_eq!(closed_after["categories"], closed_before["categories"]);
    let corrected_activity_sources: i64 = sqlx::query_scalar(
        "SELECT count(DISTINCT transaction_id) FROM budget_activity_effects \
         WHERE workspace_id=$1 AND period_start=date_trunc('month',now() AT TIME ZONE 'America/Santiago')::date \
           AND transaction_id IN ($2,$3)",
    )
    .bind(ws)
    .bind(Uuid::parse_str(expense["resource_id"].as_str().unwrap()).unwrap())
    .bind(Uuid::parse_str(refund["resource_id"].as_str().unwrap()).unwrap())
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(corrected_activity_sources, 2);
    let stale_middle_target: String = sqlx::query_scalar(
        "SELECT target_amount::text FROM budget_activity_effects WHERE workspace_id=$1 AND transaction_id=$2 AND category_id=$3 AND period_start=date_trunc('month',now() AT TIME ZONE 'America/Santiago')::date",
    )
    .bind(ws)
    .bind(Uuid::parse_str(expense["resource_id"].as_str().unwrap()).unwrap())
    .bind(other_category)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(stale_middle_target, "0");
    let latest_target: String = sqlx::query_scalar(
        "SELECT target_amount::text FROM budget_activity_effects WHERE workspace_id=$1 AND transaction_id=$2 AND category_id=$3 AND period_start=date_trunc('month',now() AT TIME ZONE 'America/Santiago')::date",
    )
    .bind(ws)
    .bind(Uuid::parse_str(expense["resource_id"].as_str().unwrap()).unwrap())
    .bind(third_category)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(latest_target, "25");
}

#[sqlx::test]
async fn closed_expense_corrections_net_per_open_correction_period(pool: PgPool) {
    let (ws, category, wallet, _usd, _clp2) = setup(&pool).await;
    let expense = json!({
        "kind": "expense",
        "business_date": "2026-01-10",
        "wallet_id": wallet,
        "amount": "25",
        "category_id": category
    })
    .to_string();
    let (status, expense) = json_response(
        send(
            pool.clone(),
            ws,
            "POST",
            "/api/transactions",
            Some(Uuid::new_v4()),
            Some(expense),
        )
        .await,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    let transaction_id = Uuid::parse_str(expense["resource_id"].as_str().unwrap()).unwrap();

    let (_, preview) = json_response(
        send(
            pool.clone(),
            ws,
            "GET",
            "/api/budgets/2026-01/close-preview",
            None,
            None,
        )
        .await,
    )
    .await;
    assert_eq!(preview["close"]["eligible"], true);
    let choices = preview["close"]["categories"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| json!({"category_id": item["category_id"], "disposition": "carry"}))
        .collect::<Vec<_>>();
    let close = json!({
        "expected_period_revision": preview["close"]["expected_period_revision"],
        "categories": choices
    })
    .to_string();
    let (status, _) = json_response(
        send(
            pool.clone(),
            ws,
            "POST",
            "/api/budgets/2026-01/close",
            Some(Uuid::new_v4()),
            Some(close),
        )
        .await,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    let frozen_activity: String = sqlx::query_scalar(
        "SELECT activity_amount::text FROM budget_close_transaction_snapshots \
         WHERE workspace_id=$1 AND transaction_id=$2",
    )
    .bind(ws)
    .bind(transaction_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(frozen_activity, "25");

    let current_period: chrono::NaiveDate = sqlx::query_scalar(
        "SELECT date_trunc('month',now() AT TIME ZONE 'America/Santiago')::date",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    for (expected_revision, amount) in [(1, "30"), (2, "35")] {
        let body = json!({
            "kind": "expense",
            "business_date": "2026-01-10",
            "wallet_id": wallet,
            "amount": amount,
            "category_id": category,
            "expected_revision": expected_revision,
            "reason": "Corrected closed period expense"
        })
        .to_string();
        let (status, _) = json_response(
            send(
                pool.clone(),
                ws,
                "PUT",
                &format!("/api/transactions/{transaction_id}"),
                Some(Uuid::new_v4()),
                Some(body),
            )
            .await,
        )
        .await;
        assert_eq!(status, axum::http::StatusCode::OK);
    }
    let same_period: (i64, String, String) = sqlx::query_as(
        "SELECT count(*)::bigint,min(baseline_amount)::text,max(target_amount)::text \
         FROM budget_activity_effects WHERE workspace_id=$1 AND transaction_id=$2 AND period_start=$3",
    )
    .bind(ws)
    .bind(transaction_id)
    .bind(current_period)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(same_period, (1, "25".to_owned(), "35".to_owned()));

    sqlx::query(
        "INSERT INTO budget_close_snapshots \
         (id,workspace_id,period_start,revision,funding,ready_to_assign,activity,uncategorized_activity) \
         VALUES($1,$2,$3,1,0,0,0,0)",
    )
    .bind(Uuid::new_v4())
    .bind(ws)
    .bind(current_period)
    .execute(&pool)
    .await
    .unwrap();
    let body = json!({
        "kind": "expense",
        "business_date": "2026-01-10",
        "wallet_id": wallet,
        "amount": "40",
        "category_id": category,
        "expected_revision": 3,
        "reason": "Corrected after correction period close"
    })
    .to_string();
    let (status, _) = json_response(
        send(
            pool.clone(),
            ws,
            "PUT",
            &format!("/api/transactions/{transaction_id}"),
            Some(Uuid::new_v4()),
            Some(body),
        )
        .await,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);

    let next_period = if current_period.month() == 12 {
        chrono::NaiveDate::from_ymd_opt(current_period.year() + 1, 1, 1).unwrap()
    } else {
        chrono::NaiveDate::from_ymd_opt(current_period.year(), current_period.month() + 1, 1)
            .unwrap()
    };
    let next_effect: (String, String) = sqlx::query_as(
        "SELECT baseline_amount::text,target_amount::text FROM budget_activity_effects \
         WHERE workspace_id=$1 AND transaction_id=$2 AND period_start=$3",
    )
    .bind(ws)
    .bind(transaction_id)
    .bind(next_period)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(next_effect, ("35".to_owned(), "40".to_owned()));
    let frozen_after: String = sqlx::query_scalar(
        "SELECT activity_amount::text FROM budget_close_transaction_snapshots \
         WHERE workspace_id=$1 AND transaction_id=$2",
    )
    .bind(ws)
    .bind(transaction_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(frozen_after, "25");
}

#[sqlx::test]
async fn closed_open_date_and_kind_corrections_reverse_or_add_only_the_required_effects(
    pool: PgPool,
) {
    let (ws, category, wallet, _usd, _clp2) = setup(&pool).await;
    let closed_expense = json!({
        "kind": "expense",
        "business_date": "2026-01-10",
        "wallet_id": wallet,
        "amount": "25",
        "category_id": category
    })
    .to_string();
    let (_, closed_expense) = json_response(
        send(
            pool.clone(),
            ws,
            "POST",
            "/api/transactions",
            Some(Uuid::new_v4()),
            Some(closed_expense),
        )
        .await,
    )
    .await;
    let (_, preview) = json_response(
        send(
            pool.clone(),
            ws,
            "GET",
            "/api/budgets/2026-01/close-preview",
            None,
            None,
        )
        .await,
    )
    .await;
    let choices = preview["close"]["categories"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| json!({"category_id": item["category_id"], "disposition": "carry"}))
        .collect::<Vec<_>>();
    let close = json!({
        "expected_period_revision": preview["close"]["expected_period_revision"],
        "categories": choices
    })
    .to_string();
    let (close_status, _) = json_response(
        send(
            pool.clone(),
            ws,
            "POST",
            "/api/budgets/2026-01/close",
            Some(Uuid::new_v4()),
            Some(close),
        )
        .await,
    )
    .await;
    assert_eq!(close_status, axum::http::StatusCode::CREATED);

    let current_period: chrono::NaiveDate = sqlx::query_scalar(
        "SELECT date_trunc('month',now() AT TIME ZONE 'America/Santiago')::date",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    let current_date = current_period.format("%Y-%m-%d").to_string();
    let move_to_open_income = json!({
        "kind": "income",
        "business_date": current_date,
        "wallet_id": wallet,
        "amount": "25",
        "expected_revision": 1,
        "reason": "Move closed expense into current income"
    })
    .to_string();
    let expense_id = closed_expense["resource_id"].as_str().unwrap();
    let (move_status, _) = json_response(
        send(
            pool.clone(),
            ws,
            "PUT",
            &format!("/api/transactions/{expense_id}"),
            Some(Uuid::new_v4()),
            Some(move_to_open_income),
        )
        .await,
    )
    .await;
    assert_eq!(move_status, axum::http::StatusCode::OK);
    let closed_to_open_target: String = sqlx::query_scalar(
        "SELECT target_amount::text FROM budget_activity_effects WHERE workspace_id=$1 AND transaction_id=$2::uuid \
         AND period_start=$3 AND category_id=$4",
    )
    .bind(ws)
    .bind(expense_id)
    .bind(current_period)
    .bind(category)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(closed_to_open_target, "0");
    let (_, budget) = json_response(
        send(
            pool.clone(),
            ws,
            "GET",
            &format!("/api/budgets/{}", current_period.format("%Y-%m")),
            None,
            None,
        )
        .await,
    )
    .await;
    assert_eq!(budget["funding"]["amount"], "25");

    let open_expense = json!({
        "kind": "expense",
        "business_date": current_date,
        "wallet_id": wallet,
        "amount": "40",
        "category_id": category
    })
    .to_string();
    let (_, open_expense) = json_response(
        send(
            pool.clone(),
            ws,
            "POST",
            "/api/transactions",
            Some(Uuid::new_v4()),
            Some(open_expense),
        )
        .await,
    )
    .await;
    let move_to_closed = json!({
        "kind": "expense",
        "business_date": "2026-01-20",
        "wallet_id": wallet,
        "amount": "40",
        "category_id": category,
        "expected_revision": 1,
        "reason": "Move open expense into closed period"
    })
    .to_string();
    let open_id = open_expense["resource_id"].as_str().unwrap();
    let (move_status, _) = json_response(
        send(
            pool.clone(),
            ws,
            "PUT",
            &format!("/api/transactions/{open_id}"),
            Some(Uuid::new_v4()),
            Some(move_to_closed),
        )
        .await,
    )
    .await;
    assert_eq!(move_status, axum::http::StatusCode::OK);
    let (baseline, target): (String, String) = sqlx::query_as(
        "SELECT baseline_amount::text,target_amount::text FROM budget_activity_effects \
         WHERE workspace_id=$1 AND transaction_id=$2::uuid AND period_start=$3 AND category_id=$4",
    )
    .bind(ws)
    .bind(open_id)
    .bind(current_period)
    .bind(category)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!((baseline, target), ("0".to_owned(), "40".to_owned()));
}

#[sqlx::test]
async fn closed_foreign_transfer_correction_uses_frozen_and_target_carrying_values(pool: PgPool) {
    let (ws, _category, _clp, usd, destination) = setup(&pool).await;
    let opening_date = chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let foreign_destination_body = json!({
        "name": "EUR destination wallet",
        "account_type": "bank",
        "base_currency": "EUR",
        "budget_eligible": true,
        "wallets": [{
            "currency": "EUR",
            "opening_amount": "0",
            "opening_date": opening_date.format("%Y-%m-%d").to_string()
        }]
    })
    .to_string();
    let (_, foreign_destination_receipt) = json_response(
        send(
            pool.clone(),
            ws,
            "POST",
            "/api/accounts",
            Some(Uuid::new_v4()),
            Some(foreign_destination_body),
        )
        .await,
    )
    .await;
    let (_, foreign_destination_account) = json_response(
        send(
            pool.clone(),
            ws,
            "GET",
            &format!(
                "/api/accounts/{}",
                foreign_destination_receipt["resource_id"].as_str().unwrap()
            ),
            None,
            None,
        )
        .await,
    )
    .await;
    let foreign_destination: Uuid = foreign_destination_account["wallets"][0]["id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    sqlx::query(
        "INSERT INTO financial_openings(id,workspace_id,wallet_id,amount,business_date) \
         VALUES($1,$2,$3,100,$4)",
    )
    .bind(Uuid::new_v4())
    .bind(ws)
    .bind(usd)
    .bind(opening_date)
    .execute(&pool)
    .await
    .unwrap();
    for (date, rate) in [
        (opening_date, "100"),
        (chrono::NaiveDate::from_ymd_opt(2026, 1, 10).unwrap(), "110"),
        (chrono::NaiveDate::from_ymd_opt(2026, 1, 11).unwrap(), "120"),
        (chrono::NaiveDate::from_ymd_opt(2026, 1, 12).unwrap(), "120"),
    ] {
        sqlx::query(
            "INSERT INTO daily_fx_observations(id,workspace_id,source,base_currency,quote_currency,effective_date,rate) \
             VALUES($1,$2,'Frankfurter v2','USD','CLP',$3,$4)",
        )
        .bind(Uuid::new_v4())
        .bind(ws)
        .bind(date)
        .bind(rust_decimal::Decimal::from_str_exact(rate).unwrap())
        .execute(&pool)
        .await
        .unwrap();
    }
    let transfer = json!({
        "kind": "transfer",
        "business_date": "2026-01-10",
        "source_wallet_id": usd,
        "source_amount": "10",
        "destination_wallet_id": destination,
        "destination_amount": "1100",
        "conversion_evidence": {"destination_clp_amount": "1100"}
    })
    .to_string();
    let (status, receipt) = json_response(
        send(
            pool.clone(),
            ws,
            "POST",
            "/api/transactions",
            Some(Uuid::new_v4()),
            Some(transfer),
        )
        .await,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    let transaction_id = Uuid::parse_str(receipt["resource_id"].as_str().unwrap()).unwrap();

    let (_, preview) = json_response(
        send(
            pool.clone(),
            ws,
            "GET",
            "/api/budgets/2026-01/close-preview",
            None,
            None,
        )
        .await,
    )
    .await;
    let choices = preview["close"]["categories"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| json!({"category_id": item["category_id"], "disposition": "carry"}))
        .collect::<Vec<_>>();
    let close = json!({
        "expected_period_revision": preview["close"]["expected_period_revision"],
        "categories": choices
    })
    .to_string();
    let (close_status, _) = json_response(
        send(
            pool.clone(),
            ws,
            "POST",
            "/api/budgets/2026-01/close",
            Some(Uuid::new_v4()),
            Some(close),
        )
        .await,
    )
    .await;
    assert_eq!(close_status, axum::http::StatusCode::CREATED);
    let frozen_funding: String = sqlx::query_scalar(
        "SELECT funding_amount::text FROM budget_close_transaction_snapshots \
         WHERE workspace_id=$1 AND transaction_id=$2",
    )
    .bind(ws)
    .bind(transaction_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(frozen_funding, "0");
    let frozen_rate_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM budget_close_transaction_rate_observations \
         WHERE workspace_id=$1 AND transaction_id=$2",
    )
    .bind(ws)
    .bind(transaction_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(frozen_rate_count, 2);

    let corrected = json!({
        "kind": "transfer",
        "business_date": "2026-01-11",
        "source_wallet_id": usd,
        "source_amount": "20",
        "destination_wallet_id": destination,
        "destination_amount": "2200",
        "conversion_evidence": {"destination_clp_amount": "2200"},
        "expected_revision": 1,
        "reason": "Correct transfer settlement"
    })
    .to_string();
    let (correction_status, _) = json_response(
        send(
            pool.clone(),
            ws,
            "PUT",
            &format!("/api/transactions/{transaction_id}"),
            Some(Uuid::new_v4()),
            Some(corrected),
        )
        .await,
    )
    .await;
    assert_eq!(correction_status, axum::http::StatusCode::OK);
    let current_period: chrono::NaiveDate = sqlx::query_scalar(
        "SELECT date_trunc('month',now() AT TIME ZONE 'America/Santiago')::date",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    let correction_effect: rust_decimal::Decimal = sqlx::query_scalar(
        "SELECT amount FROM budget_funding_effects WHERE workspace_id=$1 AND period_start=$2 \
         AND source_kind='correction' AND correction_id=( \
           SELECT id FROM financial_corrections WHERE workspace_id=$1 AND transaction_id=$3 ORDER BY new_revision LIMIT 1)",
    )
    .bind(ws)
    .bind(current_period)
    .bind(transaction_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(correction_effect, rust_decimal::Decimal::from(-200));
    let target_rate: Uuid = sqlx::query_scalar(
        "SELECT target_rate_observation_id FROM budget_funding_correction_states \
         WHERE workspace_id=$1 AND period_start=$2 AND transaction_id=$3",
    )
    .bind(ws)
    .bind(current_period)
    .bind(transaction_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    let expected_rate: Uuid = sqlx::query_scalar(
        "SELECT id FROM daily_fx_observations WHERE workspace_id=$1 AND base_currency='USD' \
         AND quote_currency='CLP' AND effective_date='2026-01-11'",
    )
    .bind(ws)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(target_rate, expected_rate);

    let current_snapshot_period: chrono::NaiveDate = sqlx::query_scalar(
        "SELECT date_trunc('month',now() AT TIME ZONE 'America/Santiago')::date",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO budget_close_snapshots \
         (id,workspace_id,period_start,revision,funding,ready_to_assign,activity,uncategorized_activity) \
         VALUES($1,$2,$3,1,0,0,0,0)",
    )
    .bind(Uuid::new_v4())
    .bind(ws)
    .bind(current_snapshot_period)
    .execute(&pool)
    .await
    .unwrap();
    let later_target = json!({
        "kind": "transfer",
        "business_date": "2026-01-12",
        "source_wallet_id": usd,
        "source_amount": "20",
        "destination_wallet_id": foreign_destination,
        "destination_amount": "15",
        "conversion_evidence": {"source": "bank transfer record"},
        "expected_revision": 2,
        "reason": "Correct foreign destination settlement"
    })
    .to_string();
    let (later_status, later_response) = json_response(
        send(
            pool.clone(),
            ws,
            "PUT",
            &format!("/api/transactions/{transaction_id}"),
            Some(Uuid::new_v4()),
            Some(later_target),
        )
        .await,
    )
    .await;
    assert_eq!(later_status, axum::http::StatusCode::OK, "{later_response}");
    let (missing_target, missing_rate_id): (Option<rust_decimal::Decimal>, Option<Uuid>) =
        sqlx::query_as(
            "SELECT target_amount,target_additional_rate_observation_id \
             FROM budget_funding_correction_states WHERE workspace_id=$1 AND period_start=( \
               SELECT max(period_start) FROM budget_funding_correction_states WHERE workspace_id=$1 AND transaction_id=$2) \
               AND transaction_id=$2",
        )
        .bind(ws)
        .bind(transaction_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(missing_target, None);
    assert_eq!(missing_rate_id, None);
    let missing_effect: Option<rust_decimal::Decimal> = sqlx::query_scalar(
        "SELECT amount FROM budget_funding_effects WHERE workspace_id=$1 AND source_kind='correction' \
         AND period_start=(SELECT max(period_start) FROM budget_funding_effects WHERE workspace_id=$1 AND correction_id=( \
           SELECT id FROM financial_corrections WHERE workspace_id=$1 AND transaction_id=$2 ORDER BY new_revision DESC LIMIT 1)) \
         AND correction_id=(SELECT id FROM financial_corrections WHERE workspace_id=$1 AND transaction_id=$2 ORDER BY new_revision DESC LIMIT 1)",
    )
    .bind(ws)
    .bind(transaction_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(missing_effect, None);
    sqlx::query(
        "INSERT INTO daily_fx_observations(id,workspace_id,source,base_currency,quote_currency,effective_date,rate,observed_at) \
         VALUES($1,$2,'Frankfurter v2','EUR','CLP','2026-01-12',150,clock_timestamp()+interval '1 second')",
    )
    .bind(Uuid::new_v4())
    .bind(ws)
    .execute(&pool)
    .await
    .unwrap();
    let next_period = if current_snapshot_period.month() == 12 {
        chrono::NaiveDate::from_ymd_opt(current_snapshot_period.year() + 1, 1, 1).unwrap()
    } else {
        chrono::NaiveDate::from_ymd_opt(
            current_snapshot_period.year(),
            current_snapshot_period.month() + 1,
            1,
        )
        .unwrap()
    };
    let (next_status, _) = json_response(
        send(
            pool.clone(),
            ws,
            "GET",
            &format!("/api/budgets/{}", next_period.format("%Y-%m")),
            None,
            None,
        )
        .await,
    )
    .await;
    assert_eq!(next_status, axum::http::StatusCode::OK);
    let refreshed_state: (rust_decimal::Decimal, Option<Uuid>, Option<Uuid>) =
        sqlx::query_as(
            "SELECT target_amount,target_rate_observation_id,target_additional_rate_observation_id \
             FROM budget_funding_correction_states WHERE workspace_id=$1 AND period_start=$2 AND transaction_id=$3",
        )
        .bind(ws)
        .bind(next_period)
        .bind(transaction_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(refreshed_state.0, rust_decimal::Decimal::from(-150));
    let net_effect: rust_decimal::Decimal = sqlx::query_scalar(
        "SELECT amount FROM budget_funding_effects WHERE workspace_id=$1 AND period_start=$2 \
         AND source_kind='correction' AND correction_id=( \
           SELECT id FROM financial_corrections WHERE workspace_id=$1 AND transaction_id=$3 ORDER BY new_revision DESC LIMIT 1)",
    )
    .bind(ws)
    .bind(next_period)
    .bind(transaction_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(net_effect, rust_decimal::Decimal::from(50));
    let revised_source_rate: Uuid = sqlx::query_scalar(
        "SELECT id FROM daily_fx_observations WHERE workspace_id=$1 AND base_currency='USD' \
         AND quote_currency='CLP' AND effective_date='2026-01-12' ORDER BY observed_at DESC,id DESC LIMIT 1",
    )
    .bind(ws)
    .fetch_one(&pool)
    .await
    .unwrap();
    let revised_destination_rate = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM daily_fx_observations WHERE workspace_id=$1 AND base_currency='EUR' \
         AND quote_currency='CLP' AND effective_date='2026-01-12' ORDER BY observed_at DESC,id DESC LIMIT 1",
    )
    .bind(ws)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(refreshed_state.1, Some(revised_source_rate));
    assert_eq!(refreshed_state.2, Some(revised_destination_rate));
    assert_ne!(refreshed_state.1, refreshed_state.2);
}

#[sqlx::test]
async fn malformed_route_parameters_use_the_api_error_envelope(pool: PgPool) {
    let workspace = bootstrap(&pool).await.unwrap();
    for uri in [
        "/api/transactions?page_size=not-a-number",
        "/api/transactions/not-a-uuid",
    ] {
        let (status, error) =
            json_response(send(pool.clone(), workspace, "GET", uri, None, None).await).await;
        assert_eq!(status, axum::http::StatusCode::BAD_REQUEST);
        assert_eq!(error["error"]["code"], "invalid_request_parameter");
        assert!(error["request_id"].is_string());
    }
}
