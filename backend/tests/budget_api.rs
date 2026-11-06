use std::time::Duration;

use axum::{
    body::{Body, to_bytes},
    http::Request,
};
use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
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
    let generation: Uuid =
        sqlx::query_scalar("SELECT installation_generation FROM workspaces WHERE id=$1")
            .bind(workspace)
            .fetch_one(&pool)
            .await
            .unwrap();
    send_with_generation(pool, workspace, generation, method, uri, key, body).await
}

async fn send_with_generation(
    pool: PgPool,
    workspace: Uuid,
    generation: Uuid,
    method: &str,
    uri: &str,
    key: Option<Uuid>,
    body: Option<String>,
) -> axum::response::Response {
    let mut request = Request::builder()
        .method(method)
        .uri(uri)
        .header("Origin", "http://localhost:4173")
        .header("Installation-Generation", generation.to_string());
    if let Some(key) = key {
        request = request.header("Idempotency-Key", key.to_string());
    }
    if body.is_some() {
        request = request.header("Content-Type", "application/json");
    }
    router(pool, workspace, config())
        .oneshot(
            request
                .body(body.map(Body::from).unwrap_or_else(Body::empty))
                .unwrap(),
        )
        .await
        .unwrap()
}

async fn create_account(pool: &PgPool, workspace: Uuid, opening_date: NaiveDate) -> Uuid {
    let body = json!({
        "name": "Budget checking",
        "account_type": "bank",
        "base_currency": "CLP",
        "budget_eligible": true,
        "wallets": [{
            "currency": "CLP",
            "opening_amount": "100000",
            "opening_date": opening_date.format("%Y-%m-%d").to_string()
        }]
    })
    .to_string();
    let (status, receipt) = json_response(
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
            &format!("/api/accounts/{}", receipt["resource_id"].as_str().unwrap()),
            None,
            None,
        )
        .await,
    )
    .await;
    account["wallets"][0]["id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap()
}

async fn current_period(pool: &PgPool) -> NaiveDate {
    sqlx::query_scalar("SELECT date_trunc('month', now() AT TIME ZONE 'America/Santiago')::date")
        .fetch_one(pool)
        .await
        .unwrap()
}

#[sqlx::test]
async fn starter_categories_are_idempotent_and_custom_categories_can_be_reordered(pool: PgPool) {
    let workspace = bootstrap(&pool).await.unwrap();
    let _ = bootstrap(&pool).await.unwrap();
    let (status, first) = json_response(
        send(
            pool.clone(),
            workspace,
            "GET",
            "/api/categories",
            None,
            None,
        )
        .await,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(first.as_array().unwrap().len(), 15);
    let (status, bootstrap_view) =
        json_response(send(pool.clone(), workspace, "GET", "/api/bootstrap", None, None).await)
            .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(bootstrap_view["categories"].as_array().unwrap().len(), 15);

    let key = Uuid::new_v4();
    let body = json!({ "name": "Pet care", "sort_order": 30 }).to_string();
    let (status, created) = json_response(
        send(
            pool.clone(),
            workspace,
            "POST",
            "/api/categories",
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
            workspace,
            "POST",
            "/api/categories",
            Some(key),
            Some(body),
        )
        .await,
    )
    .await;
    assert_eq!(retry_status, status);
    assert_eq!(retry, created);

    let category_id = created["resource_id"].as_str().unwrap();
    let edit = json!({ "name": "Pets", "sort_order": 2, "archived": false }).to_string();
    let (edit_status, _) = json_response(
        send(
            pool.clone(),
            workspace,
            "PUT",
            &format!("/api/categories/{category_id}"),
            Some(Uuid::new_v4()),
            Some(edit),
        )
        .await,
    )
    .await;
    assert_eq!(edit_status, axum::http::StatusCode::OK);
    let (_status, rows) = json_response(
        send(
            pool.clone(),
            workspace,
            "GET",
            "/api/categories",
            None,
            None,
        )
        .await,
    )
    .await;
    let pet = rows
        .as_array()
        .unwrap()
        .iter()
        .find(|category| category["id"] == category_id)
        .unwrap();
    assert_eq!(pet["name"], "Pets");
    assert_eq!(pet["sort_order"], 2);

    let archive = json!({ "name": "Pets", "sort_order": 2, "archived": true }).to_string();
    let (archive_status, _) = json_response(
        send(
            pool.clone(),
            workspace,
            "PUT",
            &format!("/api/categories/{category_id}"),
            Some(Uuid::new_v4()),
            Some(archive),
        )
        .await,
    )
    .await;
    assert_eq!(archive_status, axum::http::StatusCode::OK);
}

#[sqlx::test]
async fn stale_installation_generation_rejects_category_and_assignment_writes_without_mutation(
    pool: PgPool,
) {
    let workspace = bootstrap(&pool).await.unwrap();
    let generation: Uuid =
        sqlx::query_scalar("SELECT installation_generation FROM workspaces WHERE id=$1")
            .bind(workspace)
            .fetch_one(&pool)
            .await
            .unwrap();
    let stale_generation = Uuid::new_v4();
    assert_ne!(generation, stale_generation);
    let period = current_period(&pool).await;
    let period_text = period.format("%Y-%m").to_string();
    let category_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM expense_categories WHERE workspace_id=$1 AND slug='alimentacion'",
    )
    .bind(workspace)
    .fetch_one(&pool)
    .await
    .unwrap();

    for (method, uri, body) in [
        (
            "POST",
            "/api/categories".to_owned(),
            json!({ "name": "Should not exist", "sort_order": 90 }).to_string(),
        ),
        (
            "PUT",
            format!("/api/categories/{category_id}"),
            json!({ "name": "Changed", "sort_order": 1, "archived": false }).to_string(),
        ),
        (
            "PUT",
            format!("/api/budgets/{period_text}/assignments/{category_id}"),
            json!({ "amount": "5000" }).to_string(),
        ),
    ] {
        let (status, error) = json_response(
            send_with_generation(
                pool.clone(),
                workspace,
                stale_generation,
                method,
                &uri,
                Some(Uuid::new_v4()),
                Some(body),
            )
            .await,
        )
        .await;
        assert_eq!(status, axum::http::StatusCode::CONFLICT);
        assert_eq!(error["error"]["code"], "installation_generation_mismatch");
    }

    let category_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM expense_categories WHERE workspace_id=$1")
            .bind(workspace)
            .fetch_one(&pool)
            .await
            .unwrap();
    let category_name: String =
        sqlx::query_scalar("SELECT name FROM expense_categories WHERE workspace_id=$1 AND id=$2")
            .bind(workspace)
            .bind(category_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    let assignment_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM budget_assignments WHERE workspace_id=$1")
            .bind(workspace)
            .fetch_one(&pool)
            .await
            .unwrap();
    let receipt_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM financial_idempotency_receipts WHERE workspace_id=$1",
    )
    .bind(workspace)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(category_count, 15);
    assert_eq!(category_name, "Alimentación");
    assert_eq!(assignment_count, 0);
    assert_eq!(receipt_count, 0);
}

#[sqlx::test]
async fn budget_read_returns_signed_values_uncategorized_activity_and_assignment_revision(
    pool: PgPool,
) {
    let workspace = bootstrap(&pool).await.unwrap();
    let period = current_period(&pool).await;
    let opening_date = period;
    let wallet = create_account(&pool, workspace, opening_date).await;
    let next_day = opening_date.succ_opt().unwrap();
    let after_day = next_day.succ_opt().unwrap();
    let (_status, bootstrap_view) =
        json_response(send(pool.clone(), workspace, "GET", "/api/bootstrap", None, None).await)
            .await;
    let food_category = bootstrap_view["categories"]
        .as_array()
        .unwrap()
        .iter()
        .find(|category| category["slug"] == "alimentacion")
        .unwrap()["id"]
        .as_str()
        .unwrap();

    for (date, kind, amount, category) in [
        (next_day, "income", "10000", Value::Null),
        (after_day, "expense", "5000", json!(food_category)),
        (after_day, "expense", "1000", Value::Null),
    ] {
        let mut request = json!({
            "kind": kind,
            "business_date": date.format("%Y-%m-%d").to_string(),
            "wallet_id": wallet,
            "amount": amount,
        });
        if !category.is_null() {
            request["category_id"] = category;
        }
        let (status, _) = json_response(
            send(
                pool.clone(),
                workspace,
                "POST",
                "/api/transactions",
                Some(Uuid::new_v4()),
                Some(request.to_string()),
            )
            .await,
        )
        .await;
        assert_eq!(status, axum::http::StatusCode::CREATED);
    }

    let period_text = period.format("%Y-%m").to_string();
    let (initial_read_status, initial_budget) = json_response(
        send(
            pool.clone(),
            workspace,
            "GET",
            &format!("/api/budgets/{period_text}"),
            None,
            None,
        )
        .await,
    )
    .await;
    assert_eq!(initial_read_status, axum::http::StatusCode::OK);
    let stale_period_revision = initial_budget["revision"].as_str().unwrap().to_owned();

    let (extra_income_status, _) = json_response(
        send(
            pool.clone(),
            workspace,
            "POST",
            "/api/transactions",
            Some(Uuid::new_v4()),
            Some(
                json!({
                    "kind": "income",
                    "business_date": next_day.format("%Y-%m-%d").to_string(),
                    "wallet_id": wallet,
                    "amount": "1"
                })
                .to_string(),
            ),
        )
        .await,
    )
    .await;
    assert_eq!(extra_income_status, axum::http::StatusCode::CREATED);

    let (stale_period_status, stale_period) = json_response(
        send(
            pool.clone(),
            workspace,
            "PUT",
            &format!("/api/budgets/{period_text}/assignments/{food_category}"),
            Some(Uuid::new_v4()),
            Some(
                json!({ "amount": "115000", "expected_period_revision": stale_period_revision })
                    .to_string(),
            ),
        )
        .await,
    )
    .await;
    assert_eq!(stale_period_status, axum::http::StatusCode::CONFLICT);
    assert_eq!(stale_period["error"]["code"], "stale_revision");

    let (fresh_read_status, fresh_budget) = json_response(
        send(
            pool.clone(),
            workspace,
            "GET",
            &format!("/api/budgets/{period_text}"),
            None,
            None,
        )
        .await,
    )
    .await;
    assert_eq!(fresh_read_status, axum::http::StatusCode::OK);
    let period_revision = fresh_budget["revision"].as_str().unwrap().to_owned();
    let (assignment_status, receipt) = json_response(
        send(
            pool.clone(),
            workspace,
            "PUT",
            &format!("/api/budgets/{period_text}/assignments/{food_category}"),
            Some(Uuid::new_v4()),
            Some(
                json!({ "amount": "115000", "expected_period_revision": period_revision })
                    .to_string(),
            ),
        )
        .await,
    )
    .await;
    assert_eq!(assignment_status, axum::http::StatusCode::OK);
    assert_eq!(receipt["revision"], 1);

    let (update_status, update) = json_response(
        send(
            pool.clone(),
            workspace,
            "PUT",
            &format!("/api/budgets/{period_text}/assignments/{food_category}"),
            Some(Uuid::new_v4()),
            Some(json!({ "amount": "120000", "expected_period_revision": receipt["period_revision"], "expected_revision": 1 }).to_string()),
        )
        .await,
    )
    .await;
    assert_eq!(update_status, axum::http::StatusCode::OK);
    assert_eq!(update["revision"], 2);

    let (stale_status, stale) = json_response(
        send(
            pool.clone(),
            workspace,
            "PUT",
            &format!("/api/budgets/{period_text}/assignments/{food_category}"),
            Some(Uuid::new_v4()),
            Some(json!({ "amount": "130000", "expected_period_revision": receipt["period_revision"], "expected_revision": 1 }).to_string()),
        )
        .await,
    )
    .await;
    assert_eq!(stale_status, axum::http::StatusCode::CONFLICT);
    assert_eq!(stale["error"]["code"], "stale_revision");

    let (status, budget) = json_response(
        send(
            pool.clone(),
            workspace,
            "GET",
            &format!("/api/budgets/{period_text}"),
            None,
            None,
        )
        .await,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(budget["funding"]["state"], "complete");
    assert_eq!(budget["funding"]["amount"], "110001");
    assert_eq!(budget["ready_to_assign"]["state"], "complete");
    assert_eq!(budget["ready_to_assign"]["amount"], "-9999");
    assert_eq!(budget["uncategorized_activity"]["amount"], "1000");
    let food = budget["categories"]
        .as_array()
        .unwrap()
        .iter()
        .find(|category| category["id"] == food_category)
        .unwrap();
    assert_eq!(food["assigned"], "120000");
    assert_eq!(food["assignment_revision"], 2);
    assert_eq!(food["activity"]["amount"], "5000");
    assert_eq!(food["remaining"]["amount"], "115000");

    let archive = json!({ "name": "Alimentación", "sort_order": 2, "archived": true }).to_string();
    let (archive_status, error) = json_response(
        send(
            pool.clone(),
            workspace,
            "PUT",
            &format!("/api/categories/{food_category}"),
            Some(Uuid::new_v4()),
            Some(archive),
        )
        .await,
    )
    .await;
    assert_eq!(archive_status, axum::http::StatusCode::CONFLICT);
    assert_eq!(error["error"]["code"], "category_in_use");
}

#[sqlx::test]
async fn unresolved_only_foreign_expense_is_reported_in_its_category(pool: PgPool) {
    let workspace = bootstrap(&pool).await.unwrap();
    let period = current_period(&pool).await;
    let date_text = period.format("%Y-%m-%d").to_string();
    let category_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM expense_categories WHERE workspace_id=$1 AND slug='alimentacion'",
    )
    .bind(workspace)
    .fetch_one(&pool)
    .await
    .unwrap();
    let account_body = json!({
        "name": "Unvalued foreign expense",
        "account_type": "card",
        "base_currency": "CLP",
        "budget_eligible": true,
        "wallets": [{
            "currency": "USD",
            "opening_amount": "0",
            "opening_date": date_text
        }]
    })
    .to_string();
    let (account_status, account_receipt) = json_response(
        send(
            pool.clone(),
            workspace,
            "POST",
            "/api/accounts",
            Some(Uuid::new_v4()),
            Some(account_body),
        )
        .await,
    )
    .await;
    assert_eq!(account_status, axum::http::StatusCode::CREATED);
    let (read_status, account) = json_response(
        send(
            pool.clone(),
            workspace,
            "GET",
            &format!(
                "/api/accounts/{}",
                account_receipt["resource_id"].as_str().unwrap()
            ),
            None,
            None,
        )
        .await,
    )
    .await;
    assert_eq!(read_status, axum::http::StatusCode::OK);
    let wallet_id = account["wallets"][0]["id"].as_str().unwrap();
    let expense = json!({
        "kind": "expense",
        "business_date": date_text,
        "wallet_id": wallet_id,
        "amount": "10",
        "category_id": category_id,
        "opening_treatment": "apply_effect"
    })
    .to_string();
    let (expense_status, _) = json_response(
        send(
            pool.clone(),
            workspace,
            "POST",
            "/api/transactions",
            Some(Uuid::new_v4()),
            Some(expense),
        )
        .await,
    )
    .await;
    assert_eq!(expense_status, axum::http::StatusCode::CREATED);

    let period_text = period.format("%Y-%m").to_string();
    let (budget_status, budget) = json_response(
        send(
            pool.clone(),
            workspace,
            "GET",
            &format!("/api/budgets/{period_text}"),
            None,
            None,
        )
        .await,
    )
    .await;
    assert_eq!(budget_status, axum::http::StatusCode::OK);
    assert_eq!(budget["complete"], false);
    assert_eq!(budget["activity"]["state"], "incomplete");
    assert_eq!(budget["activity"]["known_subtotal"], "0");
    assert_eq!(budget["activity"]["unresolved"][0]["currency"], "USD");
    assert_eq!(budget["activity"]["unresolved"][0]["amount"], "10");
    assert_eq!(budget["categories"].as_array().unwrap().len(), 15);
    let category = budget["categories"]
        .as_array()
        .unwrap()
        .iter()
        .find(|category| category["id"] == category_id.to_string())
        .unwrap();
    assert_eq!(category["activity"]["state"], "incomplete");
    assert_eq!(category["activity"]["known_subtotal"], "0");
    assert_eq!(category["activity"]["unresolved"][0]["currency"], "USD");
    assert_eq!(category["remaining"]["state"], "incomplete");
}

#[sqlx::test]
async fn close_snapshots_replay_and_closed_assignment_corrections_are_period_net(pool: PgPool) {
    let workspace = bootstrap(&pool).await.unwrap();
    let current = current_period(&pool).await;
    let target = if current.month() == 1 {
        NaiveDate::from_ymd_opt(current.year() - 1, 12, 1).unwrap()
    } else {
        NaiveDate::from_ymd_opt(current.year(), current.month() - 1, 1).unwrap()
    };
    create_account(&pool, workspace, target).await;
    let period_text = target.format("%Y-%m").to_string();
    let next_text = current.format("%Y-%m").to_string();
    let (_category_status, categories) = json_response(
        send(
            pool.clone(),
            workspace,
            "GET",
            "/api/categories",
            None,
            None,
        )
        .await,
    )
    .await;
    let category_id: Uuid = categories[0]["id"].as_str().unwrap().parse().unwrap();
    let (_read_status, initial) = json_response(
        send(
            pool.clone(),
            workspace,
            "GET",
            &format!("/api/budgets/{period_text}"),
            None,
            None,
        )
        .await,
    )
    .await;
    let (assign_status, assign_receipt) = json_response(
        send(
            pool.clone(),
            workspace,
            "PUT",
            &format!("/api/budgets/{period_text}/assignments/{category_id}"),
            Some(Uuid::new_v4()),
            Some(
                json!({
                    "amount": "100",
                    "expected_period_revision": initial["revision"],
                    "expected_revision": null
                })
                .to_string(),
            ),
        )
        .await,
    )
    .await;
    assert_eq!(assign_status, axum::http::StatusCode::OK);
    let initial_assignment_revision = assign_receipt["revision"].as_i64().unwrap();

    let (preview_status, mut preview) = json_response(
        send(
            pool.clone(),
            workspace,
            "GET",
            &format!("/api/budgets/{period_text}/close-preview"),
            None,
            None,
        )
        .await,
    )
    .await;
    assert_eq!(preview_status, axum::http::StatusCode::OK);
    assert_eq!(preview["close"]["eligible"], true);
    let mut choices = preview["close"]["categories"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .map(|(index, category)| {
            json!({
                "category_id": category["category_id"],
                "disposition": if index == 0 { "release" } else { "carry" }
            })
        })
        .collect::<Vec<_>>();
    let second_category: Uuid = choices[1]["category_id"].as_str().unwrap().parse().unwrap();
    let (other_assignment_status, _) = json_response(
        send(
            pool.clone(),
            workspace,
            "PUT",
            &format!("/api/budgets/{period_text}/assignments/{second_category}"),
            Some(Uuid::new_v4()),
            Some(
                json!({
                    "amount": "50",
                    "expected_period_revision": preview["revision"],
                    "expected_revision": null
                })
                .to_string(),
            ),
        )
        .await,
    )
    .await;
    assert_eq!(other_assignment_status, axum::http::StatusCode::OK);
    let stale_close_body = json!({
        "expected_period_revision": preview["close"]["expected_period_revision"],
        "categories": choices
    })
    .to_string();
    let (stale_close_status, stale_close) = json_response(
        send(
            pool.clone(),
            workspace,
            "POST",
            &format!("/api/budgets/{period_text}/close"),
            Some(Uuid::new_v4()),
            Some(stale_close_body),
        )
        .await,
    )
    .await;
    assert_eq!(stale_close_status, axum::http::StatusCode::CONFLICT);
    assert_eq!(stale_close["error"]["code"], "stale_revision");
    preview = json_response(
        send(
            pool.clone(),
            workspace,
            "GET",
            &format!("/api/budgets/{period_text}/close-preview"),
            None,
            None,
        )
        .await,
    )
    .await
    .1;
    choices = preview["close"]["categories"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .map(|(index, category)| {
            json!({
                "category_id": category["category_id"],
                "disposition": if index == 0 { "release" } else { "carry" }
            })
        })
        .collect();
    let close_key = Uuid::new_v4();
    let close_body = json!({
        "expected_period_revision": preview["close"]["expected_period_revision"],
        "categories": choices
    })
    .to_string();
    let (close_status, close_receipt) = json_response(
        send(
            pool.clone(),
            workspace,
            "POST",
            &format!("/api/budgets/{period_text}/close"),
            Some(close_key),
            Some(close_body.clone()),
        )
        .await,
    )
    .await;
    assert_eq!(close_status, axum::http::StatusCode::CREATED);
    let (retry_status, retry_receipt) = json_response(
        send(
            pool.clone(),
            workspace,
            "POST",
            &format!("/api/budgets/{period_text}/close"),
            Some(close_key),
            Some(close_body.clone()),
        )
        .await,
    )
    .await;
    assert_eq!(retry_status, close_status);
    assert_eq!(retry_receipt, close_receipt);
    let (duplicate_status, duplicate) = json_response(
        send(
            pool.clone(),
            workspace,
            "POST",
            &format!("/api/budgets/{period_text}/close"),
            Some(Uuid::new_v4()),
            Some(close_body.clone()),
        )
        .await,
    )
    .await;
    assert_eq!(duplicate_status, axum::http::StatusCode::CONFLICT);
    assert_eq!(duplicate["error"]["code"], "period_already_closed");

    let (closed_status, closed) = json_response(
        send(
            pool.clone(),
            workspace,
            "GET",
            &format!("/api/budgets/{period_text}"),
            None,
            None,
        )
        .await,
    )
    .await;
    assert_eq!(closed_status, axum::http::StatusCode::OK);
    assert_eq!(closed["closed"], true);
    assert_eq!(closed["categories"][0]["assigned"], "100");
    let (next_status, next) = json_response(
        send(
            pool.clone(),
            workspace,
            "GET",
            &format!("/api/budgets/{next_text}"),
            None,
            None,
        )
        .await,
    )
    .await;
    assert_eq!(next_status, axum::http::StatusCode::OK);
    assert_eq!(next["funding"]["amount"], "0");
    let expected_next_ready =
        Decimal::from_str_exact(closed["ready_to_assign"]["amount"].as_str().unwrap()).unwrap()
            + Decimal::from_str_exact(
                closed["categories"][0]["remaining"]["amount"]
                    .as_str()
                    .unwrap(),
            )
            .unwrap();
    assert_eq!(
        next["ready_to_assign"]["amount"],
        expected_next_ready.to_string()
    );
    assert_eq!(
        next["categories"][1]["remaining"]["amount"],
        closed["categories"][1]["remaining"]["amount"]
    );
    let (current_preview_status, current_preview) = json_response(
        send(
            pool.clone(),
            workspace,
            "GET",
            &format!("/api/budgets/{next_text}/close-preview"),
            None,
            None,
        )
        .await,
    )
    .await;
    assert_eq!(current_preview_status, axum::http::StatusCode::OK);
    assert_eq!(current_preview["close"]["eligible"], false);
    assert_eq!(current_preview["close"]["reason"], "period_not_ended");

    let closed_revision = closed["revision"].as_str().unwrap();
    let (missing_reason_status, missing_reason) = json_response(
        send(
            pool.clone(),
            workspace,
            "PUT",
            &format!("/api/budgets/{period_text}/assignments/{category_id}"),
            Some(Uuid::new_v4()),
            Some(
                json!({
                    "amount": "80",
                    "expected_period_revision": closed_revision,
                    "expected_revision": initial_assignment_revision
                })
                .to_string(),
            ),
        )
        .await,
    )
    .await;
    assert_eq!(
        missing_reason_status,
        axum::http::StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(missing_reason["error"]["fields"][0]["field"], "reason");
    let stale_body = json!({
        "amount": "80",
        "expected_period_revision": closed_revision,
        "expected_revision": initial_assignment_revision,
        "reason": "correct allocation"
    })
    .to_string();
    let (correction_status, correction_receipt) = json_response(
        send(
            pool.clone(),
            workspace,
            "PUT",
            &format!("/api/budgets/{period_text}/assignments/{category_id}"),
            Some(Uuid::new_v4()),
            Some(stale_body),
        )
        .await,
    )
    .await;
    assert_eq!(correction_status, axum::http::StatusCode::OK);
    let first_correction_revision = correction_receipt["revision"].as_i64().unwrap();

    let latest_closed = json_response(
        send(
            pool.clone(),
            workspace,
            "GET",
            &format!("/api/budgets/{period_text}"),
            None,
            None,
        )
        .await,
    )
    .await
    .1;
    let (stale_status, stale) = json_response(
        send(
            pool.clone(),
            workspace,
            "PUT",
            &format!("/api/budgets/{period_text}/assignments/{category_id}"),
            Some(Uuid::new_v4()),
            Some(
                json!({
                    "amount": "75",
                    "expected_period_revision": latest_closed["revision"],
                    "expected_revision": initial_assignment_revision,
                    "reason": "stale editor"
                })
                .to_string(),
            ),
        )
        .await,
    )
    .await;
    assert_eq!(stale_status, axum::http::StatusCode::CONFLICT);
    assert_eq!(stale["error"]["code"], "stale_revision");

    let (second_status, second_receipt) = json_response(
        send(
            pool.clone(),
            workspace,
            "PUT",
            &format!("/api/budgets/{period_text}/assignments/{category_id}"),
            Some(Uuid::new_v4()),
            Some(
                json!({
                    "amount": "70",
                    "expected_period_revision": latest_closed["revision"],
                    "expected_revision": first_correction_revision,
                    "reason": "refine allocation"
                })
                .to_string(),
            ),
        )
        .await,
    )
    .await;
    assert_eq!(second_status, axum::http::StatusCode::OK);
    assert_eq!(second_receipt["revision"], first_correction_revision + 1);
    let (latest_status, latest) = json_response(
        send(
            pool.clone(),
            workspace,
            "GET",
            &format!("/api/budgets/{period_text}"),
            None,
            None,
        )
        .await,
    )
    .await;
    assert_eq!(latest_status, axum::http::StatusCode::OK);
    assert_eq!(latest["categories"][0]["assigned"], "100");
    assert_eq!(latest["categories"][0]["assignment_intent"], "70");
    let original_amount: String = sqlx::query_scalar(
        "SELECT amount::text FROM budget_assignments WHERE workspace_id=$1 AND period_start=$2 AND category_id=$3",
    )
    .bind(workspace)
    .bind(target)
    .bind(category_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(original_amount, "100");
    let effect: (i64, Decimal) = sqlx::query_as(
        "SELECT count(*), max(amount) FROM budget_funding_effects f \
         JOIN financial_corrections c ON c.workspace_id=f.workspace_id AND c.id=f.correction_id \
         WHERE f.workspace_id=$1 AND f.period_start=$2 AND c.assignment_id=$3",
    )
    .bind(workspace)
    .bind(current)
    .bind(
        assign_receipt["resource_id"]
            .as_str()
            .unwrap()
            .parse::<Uuid>()
            .unwrap(),
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(effect.0, 1);
    assert_eq!(effect.1, Decimal::from(30));
    let snapshot_id: Uuid = close_receipt["resource_id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    let snapshot_amount: String = sqlx::query_scalar(
        "SELECT assigned::text FROM budget_close_category_snapshots WHERE workspace_id=$1 AND snapshot_id=$2 AND category_id=$3",
    )
    .bind(workspace)
    .bind(snapshot_id)
    .bind(category_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(snapshot_amount, "100");

    // Simulate the correction period being closed, then verify the next edit
    // starts from the previous target and writes only the incremental effect.
    let current_ready = next["ready_to_assign"]["amount"].as_str().unwrap();
    sqlx::query(
        "INSERT INTO budget_close_snapshots \
         (id,workspace_id,period_start,revision,funding,ready_to_assign,activity,uncategorized_activity) \
         VALUES ($1,$2,$3,1,0,$4,0,0)",
    )
    .bind(Uuid::new_v4())
    .bind(workspace)
    .bind(current)
    .bind(Decimal::from_str_exact(current_ready).unwrap())
    .execute(&pool)
    .await
    .unwrap();
    let next_period = if current.month() == 12 {
        NaiveDate::from_ymd_opt(current.year() + 1, 1, 1).unwrap()
    } else {
        NaiveDate::from_ymd_opt(current.year(), current.month() + 1, 1).unwrap()
    };
    let fresh_closed = json_response(
        send(
            pool.clone(),
            workspace,
            "GET",
            &format!("/api/budgets/{period_text}"),
            None,
            None,
        )
        .await,
    )
    .await
    .1;
    let (third_status, third_receipt) = json_response(
        send(
            pool.clone(),
            workspace,
            "PUT",
            &format!("/api/budgets/{period_text}/assignments/{category_id}"),
            Some(Uuid::new_v4()),
            Some(
                json!({
                    "amount": "65",
                    "expected_period_revision": fresh_closed["revision"],
                    "expected_revision": second_receipt["revision"],
                    "reason": "correction period closed"
                })
                .to_string(),
            ),
        )
        .await,
    )
    .await;
    assert_eq!(third_status, axum::http::StatusCode::OK);
    assert_eq!(
        third_receipt["revision"],
        second_receipt["revision"].as_i64().unwrap() + 1
    );
    let effect_rows: Vec<(NaiveDate, Decimal)> = sqlx::query_as(
        "SELECT f.period_start, f.amount FROM budget_funding_effects f \
         JOIN financial_corrections c ON c.workspace_id=f.workspace_id AND c.id=f.correction_id \
         WHERE f.workspace_id=$1 AND c.assignment_id=$2 ORDER BY f.period_start",
    )
    .bind(workspace)
    .bind(
        assign_receipt["resource_id"]
            .as_str()
            .unwrap()
            .parse::<Uuid>()
            .unwrap(),
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        effect_rows,
        vec![
            (current, Decimal::from(30)),
            (next_period, Decimal::from(5))
        ]
    );
}

#[sqlx::test]
async fn incomplete_budget_cannot_close_and_later_close_preview_is_out_of_order(pool: PgPool) {
    let workspace = bootstrap(&pool).await.unwrap();
    let current = current_period(&pool).await;
    let target = if current.month() == 1 {
        NaiveDate::from_ymd_opt(current.year() - 1, 12, 1).unwrap()
    } else {
        NaiveDate::from_ymd_opt(current.year(), current.month() - 1, 1).unwrap()
    };
    let account = json!({
        "name": "Unvalued wallet",
        "account_type": "bank",
        "base_currency": "ZZZ",
        "budget_eligible": true,
        "wallets": [{
            "currency": "ZZZ",
            "opening_amount": "100",
            "opening_date": target.format("%Y-%m-%d").to_string()
        }]
    })
    .to_string();
    let (create_status, receipt) = json_response(
        send(
            pool.clone(),
            workspace,
            "POST",
            "/api/accounts",
            Some(Uuid::new_v4()),
            Some(account),
        )
        .await,
    )
    .await;
    assert_eq!(
        create_status,
        axum::http::StatusCode::CREATED,
        "account creation failed: {receipt}"
    );
    let account_id = receipt["resource_id"]
        .as_str()
        .expect("account creation receipt has a resource id");
    let (read_status, account) = json_response(
        send(
            pool.clone(),
            workspace,
            "GET",
            &format!("/api/accounts/{account_id}"),
            None,
            None,
        )
        .await,
    )
    .await;
    assert_eq!(
        read_status,
        axum::http::StatusCode::OK,
        "account read failed: {account}"
    );
    assert!(
        account["wallets"]
            .as_array()
            .is_some_and(|wallets| !wallets.is_empty()),
        "account read returned no wallets: {account}"
    );
    let wallet: Uuid = account["wallets"][0]["id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    sqlx::query(
        "INSERT INTO daily_fx_observations \
         (id,workspace_id,source,base_currency,quote_currency,effective_date,rate) \
         VALUES ($1,$2,'Frankfurter v2','ZZZ','CLP',$3,100)",
    )
    .bind(Uuid::new_v4())
    .bind(workspace)
    .bind(target)
    .execute(&pool)
    .await
    .unwrap();
    let (_status, categories) = json_response(
        send(
            pool.clone(),
            workspace,
            "GET",
            "/api/categories",
            None,
            None,
        )
        .await,
    )
    .await;
    let category: Uuid = categories[0]["id"].as_str().unwrap().parse().unwrap();
    let (expense_status, _) = json_response(
        send(
            pool.clone(),
            workspace,
            "POST",
            "/api/transactions",
            Some(Uuid::new_v4()),
            Some(
                json!({
                    "kind": "expense",
                    "business_date": target.format("%Y-%m-%d").to_string(),
                    "wallet_id": wallet,
                    "amount": "1",
                    "category_id": category,
                    "opening_treatment": "apply_effect"
                })
                .to_string(),
            ),
        )
        .await,
    )
    .await;
    assert_eq!(expense_status, axum::http::StatusCode::CREATED);

    let target_text = target.format("%Y-%m").to_string();
    let (preview_status, preview) = json_response(
        send(
            pool.clone(),
            workspace,
            "GET",
            &format!("/api/budgets/{target_text}/close-preview"),
            None,
            None,
        )
        .await,
    )
    .await;
    assert_eq!(preview_status, axum::http::StatusCode::OK);
    assert_eq!(preview["close"]["eligible"], false);
    assert_eq!(preview["close"]["reason"], "incomplete_values");
    let choices = preview["close"]["categories"]
        .as_array()
        .unwrap()
        .iter()
        .map(|category| {
            json!({
                "category_id": category["category_id"],
                "disposition": "carry"
            })
        })
        .collect::<Vec<_>>();
    let (close_status, error) = json_response(
        send(
            pool.clone(),
            workspace,
            "POST",
            &format!("/api/budgets/{target_text}/close"),
            Some(Uuid::new_v4()),
            Some(
                json!({
                    "expected_period_revision": preview["revision"],
                    "categories": choices
                })
                .to_string(),
            ),
        )
        .await,
    )
    .await;
    assert_eq!(close_status, axum::http::StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(error["error"]["fields"][0]["code"], "incomplete_values");

    let later = if current.month() == 12 {
        NaiveDate::from_ymd_opt(current.year() + 1, 1, 1).unwrap()
    } else {
        NaiveDate::from_ymd_opt(current.year(), current.month() + 1, 1).unwrap()
    };
    let (later_status, later_preview) = json_response(
        send(
            pool.clone(),
            workspace,
            "GET",
            &format!("/api/budgets/{}/close-preview", later.format("%Y-%m")),
            None,
            None,
        )
        .await,
    )
    .await;
    assert_eq!(later_status, axum::http::StatusCode::OK);
    assert_eq!(later_preview["close"]["eligible"], false);
    assert_eq!(later_preview["close"]["reason"], "out_of_order");
}

#[sqlx::test]
async fn foreign_transfer_uses_net_asset_balance_after_liability_repayment(pool: PgPool) {
    let workspace = bootstrap(&pool).await.unwrap();
    let period = current_period(&pool).await;
    let income_date = period.succ_opt().unwrap();
    let transfer_date = income_date.succ_opt().unwrap();

    let foreign_body = json!({
        "name": "Foreign liability",
        "account_type": "bank",
        "base_currency": "USD",
        "budget_eligible": true,
        "wallets": [{
            "currency": "USD",
            "opening_amount": "-100",
            "opening_date": period.format("%Y-%m-%d").to_string(),
            "liability_clp_basis": "-10000"
        }]
    })
    .to_string();
    let (foreign_status, foreign_receipt) = json_response(
        send(
            pool.clone(),
            workspace,
            "POST",
            "/api/accounts",
            Some(Uuid::new_v4()),
            Some(foreign_body),
        )
        .await,
    )
    .await;
    assert_eq!(foreign_status, axum::http::StatusCode::CREATED);
    let (_, foreign_account) = json_response(
        send(
            pool.clone(),
            workspace,
            "GET",
            &format!(
                "/api/accounts/{}",
                foreign_receipt["resource_id"].as_str().unwrap()
            ),
            None,
            None,
        )
        .await,
    )
    .await;
    let foreign_wallet: Uuid = foreign_account["wallets"][0]["id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();

    let clp_body = json!({
        "name": "Transfer destination",
        "account_type": "bank",
        "base_currency": "CLP",
        "budget_eligible": true,
        "wallets": [{
            "currency": "CLP",
            "opening_amount": "0",
            "opening_date": period.format("%Y-%m-%d").to_string()
        }]
    })
    .to_string();
    let (_, clp_receipt) = json_response(
        send(
            pool.clone(),
            workspace,
            "POST",
            "/api/accounts",
            Some(Uuid::new_v4()),
            Some(clp_body),
        )
        .await,
    )
    .await;
    let (_, clp_account) = json_response(
        send(
            pool.clone(),
            workspace,
            "GET",
            &format!(
                "/api/accounts/{}",
                clp_receipt["resource_id"].as_str().unwrap()
            ),
            None,
            None,
        )
        .await,
    )
    .await;
    let clp_wallet: Uuid = clp_account["wallets"][0]["id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();

    for (date, rate) in [
        (period, "100"),
        (income_date, "105"),
        (transfer_date, "110"),
    ] {
        sqlx::query(
            "INSERT INTO daily_fx_observations(id,workspace_id,source,base_currency,quote_currency,effective_date,rate) \
             VALUES($1,$2,'Frankfurter v2','USD','CLP',$3,$4)",
        )
        .bind(Uuid::new_v4())
        .bind(workspace)
        .bind(date)
        .bind(Decimal::from_str_exact(rate).unwrap())
        .execute(&pool)
        .await
        .unwrap();
    }

    let income = json!({
        "kind": "income",
        "business_date": income_date.format("%Y-%m-%d").to_string(),
        "wallet_id": foreign_wallet,
        "amount": "150"
    })
    .to_string();
    let (income_status, _) = json_response(
        send(
            pool.clone(),
            workspace,
            "POST",
            "/api/transactions",
            Some(Uuid::new_v4()),
            Some(income),
        )
        .await,
    )
    .await;
    assert_eq!(income_status, axum::http::StatusCode::CREATED);

    let transfer = json!({
        "kind": "transfer",
        "business_date": transfer_date.format("%Y-%m-%d").to_string(),
        "source_wallet_id": foreign_wallet,
        "source_amount": "50",
        "destination_wallet_id": clp_wallet,
        "destination_amount": "5500",
        "conversion_evidence": {"destination_clp_amount": "5500"}
    })
    .to_string();
    let (transfer_status, _) = json_response(
        send(
            pool.clone(),
            workspace,
            "POST",
            "/api/transactions",
            Some(Uuid::new_v4()),
            Some(transfer),
        )
        .await,
    )
    .await;
    assert_eq!(transfer_status, axum::http::StatusCode::CREATED);

    let (budget_status, budget) = json_response(
        send(
            pool.clone(),
            workspace,
            "GET",
            &format!("/api/budgets/{}", period.format("%Y-%m")),
            None,
            None,
        )
        .await,
    )
    .await;
    assert_eq!(budget_status, axum::http::StatusCode::OK);
    // The +150 flow first retires the −100 foreign liability at its supplied
    // basis. The remaining 50 units are marked at 110 immediately before the
    // transfer, so the source carrying value equals its 5,500 CLP receipt.
    assert_eq!(budget["funding"]["state"], "complete");
    assert_eq!(budget["funding"]["amount"], "5750");
}

#[sqlx::test]
async fn foreign_asset_revaluation_uses_flow_basis_and_records_net_rate_change_once(pool: PgPool) {
    let workspace = bootstrap(&pool).await.unwrap();
    let period = current_period(&pool).await;
    let as_of: NaiveDate =
        sqlx::query_scalar("SELECT (now() AT TIME ZONE 'America/Santiago')::date")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(as_of > period);
    let account_body = json!({
        "name": "Foreign wallet",
        "account_type": "investment",
        "base_currency": "ZZZ",
        "budget_eligible": true,
        "wallets": [{
            "currency": "ZZZ",
                "opening_amount": "100",
            "opening_date": period.format("%Y-%m-%d").to_string()
        }]
    })
    .to_string();
    let (account_status, account) = json_response(
        send(
            pool.clone(),
            workspace,
            "POST",
            "/api/accounts",
            Some(Uuid::new_v4()),
            Some(account_body),
        )
        .await,
    )
    .await;
    assert_eq!(account_status, axum::http::StatusCode::CREATED);
    let (_account_read_status, account_view) = json_response(
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
    let wallet_id = account_view["wallets"][0]["id"].as_str().unwrap();

    sqlx::query(
        "INSERT INTO daily_fx_observations \
         (id, workspace_id, source, base_currency, quote_currency, effective_date, rate) \
         VALUES ($1, $2, 'Frankfurter v2', 'ZZZ', 'CLP', $3, 100)",
    )
    .bind(Uuid::new_v4())
    .bind(workspace)
    .bind(period)
    .execute(&pool)
    .await
    .unwrap();
    let period_text = period.format("%Y-%m").to_string();
    let (first_status, first_budget) = json_response(
        send(
            pool.clone(),
            workspace,
            "GET",
            &format!("/api/budgets/{period_text}"),
            None,
            None,
        )
        .await,
    )
    .await;
    assert_eq!(first_status, axum::http::StatusCode::OK);
    assert_eq!(first_budget["funding"]["amount"], "10000");

    let flow_date = period.succ_opt().unwrap();
    sqlx::query(
        "INSERT INTO daily_fx_observations \
         (id, workspace_id, source, base_currency, quote_currency, effective_date, rate) \
         VALUES ($1, $2, 'Frankfurter v2', 'ZZZ', 'CLP', $3, 105)",
    )
    .bind(Uuid::new_v4())
    .bind(workspace)
    .bind(flow_date)
    .execute(&pool)
    .await
    .unwrap();
    let (income_status, _) = json_response(
        send(
            pool.clone(),
            workspace,
            "POST",
            "/api/transactions",
            Some(Uuid::new_v4()),
            Some(
                json!({
                    "kind": "income",
                    "business_date": flow_date.format("%Y-%m-%d").to_string(),
                    "wallet_id": wallet_id,
                    "amount": "100"
                })
                .to_string(),
            ),
        )
        .await,
    )
    .await;
    assert_eq!(income_status, axum::http::StatusCode::CREATED);

    sqlx::query(
        "INSERT INTO daily_fx_observations \
         (id, workspace_id, source, base_currency, quote_currency, effective_date, rate) \
         VALUES ($1, $2, 'Frankfurter v2', 'ZZZ', 'CLP', $3, 110)",
    )
    .bind(Uuid::new_v4())
    .bind(workspace)
    .bind(as_of)
    .execute(&pool)
    .await
    .unwrap();
    for _ in 0..2 {
        let (status, budget) = json_response(
            send(
                pool.clone(),
                workspace,
                "GET",
                &format!("/api/budgets/{period_text}"),
                None,
                None,
            )
            .await,
        )
        .await;
        assert_eq!(status, axum::http::StatusCode::OK);
        assert_eq!(budget["funding"]["amount"], "22000");
    }
    sqlx::query(
        "INSERT INTO daily_fx_observations \
         (id, workspace_id, source, base_currency, quote_currency, effective_date, rate) \
         VALUES ($1, $2, 'Frankfurter v2', 'ZZZ', 'CLP', $3, 115)",
    )
    .bind(Uuid::new_v4())
    .bind(workspace)
    .bind(as_of)
    .execute(&pool)
    .await
    .unwrap();
    for _ in 0..2 {
        let (status, budget) = json_response(
            send(
                pool.clone(),
                workspace,
                "GET",
                &format!("/api/budgets/{period_text}"),
                None,
                None,
            )
            .await,
        )
        .await;
        assert_eq!(status, axum::http::StatusCode::OK);
        assert_eq!(budget["funding"]["amount"], "23000");
    }
    sqlx::query(
        "UPDATE daily_fx_observations SET observed_at=now() + interval '1 minute' \
         WHERE workspace_id=$1 AND base_currency='ZZZ' AND quote_currency='CLP' \
           AND effective_date=$2 AND rate=110",
    )
    .bind(workspace)
    .bind(as_of)
    .execute(&pool)
    .await
    .unwrap();
    for _ in 0..2 {
        let (status, budget) = json_response(
            send(
                pool.clone(),
                workspace,
                "GET",
                &format!("/api/budgets/{period_text}"),
                None,
                None,
            )
            .await,
        )
        .await;
        assert_eq!(status, axum::http::StatusCode::OK);
        assert_eq!(budget["funding"]["amount"], "22000");
    }
    sqlx::query(
        "UPDATE daily_fx_observations SET observed_at=now() + interval '2 minutes' \
         WHERE workspace_id=$1 AND base_currency='ZZZ' AND quote_currency='CLP' \
           AND effective_date=$2 AND rate=115",
    )
    .bind(workspace)
    .bind(as_of)
    .execute(&pool)
    .await
    .unwrap();
    let (return_status, return_budget) = json_response(
        send(
            pool.clone(),
            workspace,
            "GET",
            &format!("/api/budgets/{period_text}"),
            None,
            None,
        )
        .await,
    )
    .await;
    assert_eq!(return_status, axum::http::StatusCode::OK);
    assert_eq!(return_budget["funding"]["amount"], "23000");
    let revaluation_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM budget_funding_effects \
         WHERE workspace_id=$1 AND source_kind='valuation' AND wallet_id=$2",
    )
    .bind(workspace)
    .bind(Uuid::parse_str(wallet_id).unwrap())
    .fetch_one(&pool)
    .await
    .unwrap();
    let revaluation_deltas: Vec<rust_decimal::Decimal> = sqlx::query_scalar(
        "SELECT amount FROM budget_funding_effects \
         WHERE workspace_id=$1 AND source_kind='valuation' AND wallet_id=$2",
    )
    .bind(workspace)
    .bind(Uuid::parse_str(wallet_id).unwrap())
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(revaluation_count, 2);
    assert!(
        revaluation_deltas
            .iter()
            .any(|amount| amount.to_string() == "500")
    );
    assert!(
        revaluation_deltas
            .iter()
            .any(|amount| amount.to_string() == "2000")
    );
}

#[sqlx::test]
async fn revaluation_handles_zero_opening_wallet_funded_by_later_foreign_income(pool: PgPool) {
    let workspace = bootstrap(&pool).await.unwrap();
    let period = current_period(&pool).await;
    let flow_date = period.succ_opt().unwrap();
    let as_of: NaiveDate =
        sqlx::query_scalar("SELECT (now() AT TIME ZONE 'America/Santiago')::date")
            .fetch_one(&pool)
            .await
            .unwrap();
    let (create_status, receipt) = json_response(
        send(
            pool.clone(),
            workspace,
            "POST",
            "/api/accounts",
            Some(Uuid::new_v4()),
            Some(
                json!({
                    "name": "Foreign wallet opened empty",
                    "account_type": "investment",
                    "base_currency": "ZZZ",
                    "budget_eligible": true,
                    "wallets": [{
                        "currency": "ZZZ",
                        "opening_amount": "0",
                        "opening_date": period.format("%Y-%m-%d").to_string()
                    }]
                })
                .to_string(),
            ),
        )
        .await,
    )
    .await;
    assert_eq!(create_status, axum::http::StatusCode::CREATED);
    let (_read_status, account) = json_response(
        send(
            pool.clone(),
            workspace,
            "GET",
            &format!("/api/accounts/{}", receipt["resource_id"].as_str().unwrap()),
            None,
            None,
        )
        .await,
    )
    .await;
    let wallet_id = account["wallets"][0]["id"].as_str().unwrap();
    for (date, rate) in [(flow_date, 100), (as_of, 110)] {
        sqlx::query(
            "INSERT INTO daily_fx_observations \
             (id, workspace_id, source, base_currency, quote_currency, effective_date, rate) \
             VALUES ($1, $2, 'Frankfurter v2', 'ZZZ', 'CLP', $3, $4)",
        )
        .bind(Uuid::new_v4())
        .bind(workspace)
        .bind(date)
        .bind(rust_decimal::Decimal::from(rate))
        .execute(&pool)
        .await
        .unwrap();
    }
    let (income_status, _) = json_response(
        send(
            pool.clone(),
            workspace,
            "POST",
            "/api/transactions",
            Some(Uuid::new_v4()),
            Some(
                json!({
                    "kind": "income",
                    "business_date": flow_date.format("%Y-%m-%d").to_string(),
                    "wallet_id": wallet_id,
                    "amount": "100"
                })
                .to_string(),
            ),
        )
        .await,
    )
    .await;
    assert_eq!(income_status, axum::http::StatusCode::CREATED);

    let period_text = period.format("%Y-%m").to_string();
    let (budget_status, budget) = json_response(
        send(
            pool.clone(),
            workspace,
            "GET",
            &format!("/api/budgets/{period_text}"),
            None,
            None,
        )
        .await,
    )
    .await;
    assert_eq!(budget_status, axum::http::StatusCode::OK);
    assert_eq!(budget["funding"]["amount"], "11000");
    let valuation: rust_decimal::Decimal = sqlx::query_scalar(
        "SELECT amount FROM budget_funding_effects \
         WHERE workspace_id=$1 AND source_kind='valuation' AND wallet_id=$2",
    )
    .bind(workspace)
    .bind(Uuid::parse_str(wallet_id).unwrap())
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(valuation.to_string(), "1000");
}

#[sqlx::test]
async fn correcting_opening_recomputes_existing_same_rate_valuation_net_amount(pool: PgPool) {
    let workspace = bootstrap(&pool).await.unwrap();
    let period = current_period(&pool).await;
    let as_of: NaiveDate =
        sqlx::query_scalar("SELECT (now() AT TIME ZONE 'America/Santiago')::date")
            .fetch_one(&pool)
            .await
            .unwrap();
    let account_body = json!({
        "name": "Corrected foreign opening",
        "account_type": "investment",
        "base_currency": "ZZZ",
        "budget_eligible": true,
        "wallets": [{
            "currency": "ZZZ",
            "opening_amount": "100",
            "opening_date": period.format("%Y-%m-%d").to_string()
        }]
    })
    .to_string();
    let (create_status, account_receipt) = json_response(
        send(
            pool.clone(),
            workspace,
            "POST",
            "/api/accounts",
            Some(Uuid::new_v4()),
            Some(account_body),
        )
        .await,
    )
    .await;
    assert_eq!(create_status, axum::http::StatusCode::CREATED);
    let account_id = account_receipt["resource_id"].as_str().unwrap();
    let (_status, account) = json_response(
        send(
            pool.clone(),
            workspace,
            "GET",
            &format!("/api/accounts/{account_id}"),
            None,
            None,
        )
        .await,
    )
    .await;
    let wallet_id = account["wallets"][0]["id"].as_str().unwrap();
    let opening_id = account["wallets"][0]["openings"][0]["id"].as_str().unwrap();
    for (date, rate) in [(period, 100), (as_of, 110)] {
        sqlx::query(
            "INSERT INTO daily_fx_observations \
             (id, workspace_id, source, base_currency, quote_currency, effective_date, rate) \
             VALUES ($1, $2, 'Frankfurter v2', 'ZZZ', 'CLP', $3, $4)",
        )
        .bind(Uuid::new_v4())
        .bind(workspace)
        .bind(date)
        .bind(rust_decimal::Decimal::from(rate))
        .execute(&pool)
        .await
        .unwrap();
    }
    let period_text = period.format("%Y-%m").to_string();
    let (first_status, first_budget) = json_response(
        send(
            pool.clone(),
            workspace,
            "GET",
            &format!("/api/budgets/{period_text}"),
            None,
            None,
        )
        .await,
    )
    .await;
    assert_eq!(first_status, axum::http::StatusCode::OK);
    assert_eq!(first_budget["funding"]["amount"], "11000");

    let (correction_status, _) = json_response(
        send(
            pool.clone(),
            workspace,
            "PUT",
            &format!("/api/accounts/{account_id}/openings/{opening_id}"),
            Some(Uuid::new_v4()),
            Some(
                json!({
                    "amount": "200",
                    "expected_revision": 1,
                    "reason": "statement correction"
                })
                .to_string(),
            ),
        )
        .await,
    )
    .await;
    assert_eq!(correction_status, axum::http::StatusCode::OK);

    let (corrected_status, corrected_budget) = json_response(
        send(
            pool.clone(),
            workspace,
            "GET",
            &format!("/api/budgets/{period_text}"),
            None,
            None,
        )
        .await,
    )
    .await;
    assert_eq!(corrected_status, axum::http::StatusCode::OK);
    assert_eq!(corrected_budget["funding"]["amount"], "22000");
    let valuation_rows: Vec<(String, rust_decimal::Decimal)> = sqlx::query_as(
        "SELECT rate_observation_id::text, amount FROM budget_funding_effects \
         WHERE workspace_id=$1 AND source_kind='valuation' AND wallet_id=$2",
    )
    .bind(workspace)
    .bind(Uuid::parse_str(wallet_id).unwrap())
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(valuation_rows.len(), 1);
    assert_eq!(valuation_rows[0].1.to_string(), "2000");

    let (zero_correction_status, _) = json_response(
        send(
            pool.clone(),
            workspace,
            "PUT",
            &format!("/api/accounts/{account_id}/openings/{opening_id}"),
            Some(Uuid::new_v4()),
            Some(
                json!({
                    "amount": "0",
                    "expected_revision": 2,
                    "reason": "opening was fully offset"
                })
                .to_string(),
            ),
        )
        .await,
    )
    .await;
    assert_eq!(zero_correction_status, axum::http::StatusCode::OK);
    let (zero_status, zero_budget) = json_response(
        send(
            pool.clone(),
            workspace,
            "GET",
            &format!("/api/budgets/{period_text}"),
            None,
            None,
        )
        .await,
    )
    .await;
    assert_eq!(zero_status, axum::http::StatusCode::OK);
    assert_eq!(zero_budget["funding"]["amount"], "0");
    let cleared_amount: rust_decimal::Decimal = sqlx::query_scalar(
        "SELECT amount FROM budget_funding_effects \
         WHERE workspace_id=$1 AND source_kind='valuation' AND wallet_id=$2",
    )
    .bind(workspace)
    .bind(Uuid::parse_str(wallet_id).unwrap())
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(cleared_amount, rust_decimal::Decimal::ZERO);
}

#[sqlx::test]
async fn revaluation_ignores_preserve_balance_wallet_effects(pool: PgPool) {
    let workspace = bootstrap(&pool).await.unwrap();
    let period = current_period(&pool).await;
    let as_of: NaiveDate =
        sqlx::query_scalar("SELECT (now() AT TIME ZONE 'America/Santiago')::date")
            .fetch_one(&pool)
            .await
            .unwrap();
    let account_body = json!({
        "name": "Preserved foreign balance",
        "account_type": "investment",
        "base_currency": "ZZZ",
        "budget_eligible": true,
        "wallets": [{
            "currency": "ZZZ",
            "opening_amount": "100",
            "opening_date": period.format("%Y-%m-%d").to_string()
        }]
    })
    .to_string();
    let (_create_status, account_receipt) = json_response(
        send(
            pool.clone(),
            workspace,
            "POST",
            "/api/accounts",
            Some(Uuid::new_v4()),
            Some(account_body),
        )
        .await,
    )
    .await;
    let (_read_status, account) = json_response(
        send(
            pool.clone(),
            workspace,
            "GET",
            &format!(
                "/api/accounts/{}",
                account_receipt["resource_id"].as_str().unwrap()
            ),
            None,
            None,
        )
        .await,
    )
    .await;
    let wallet_id = account["wallets"][0]["id"].as_str().unwrap();
    for (date, rate) in [(period, 100), (as_of, 110)] {
        sqlx::query(
            "INSERT INTO daily_fx_observations \
             (id, workspace_id, source, base_currency, quote_currency, effective_date, rate) \
             VALUES ($1, $2, 'Frankfurter v2', 'ZZZ', 'CLP', $3, $4)",
        )
        .bind(Uuid::new_v4())
        .bind(workspace)
        .bind(date)
        .bind(rust_decimal::Decimal::from(rate))
        .execute(&pool)
        .await
        .unwrap();
    }
    let (transaction_status, transaction_receipt) = json_response(
        send(
            pool.clone(),
            workspace,
            "POST",
            "/api/transactions",
            Some(Uuid::new_v4()),
            Some(
                json!({
                    "kind": "expense",
                    "business_date": period.format("%Y-%m-%d").to_string(),
                    "wallet_id": wallet_id,
                    "amount": "50",
                    "opening_treatment": "preserve_balance"
                })
                .to_string(),
            ),
        )
        .await,
    )
    .await;
    assert_eq!(transaction_status, axum::http::StatusCode::CREATED);

    let period_text = period.format("%Y-%m").to_string();
    let (budget_status, budget) = json_response(
        send(
            pool.clone(),
            workspace,
            "GET",
            &format!("/api/budgets/{period_text}"),
            None,
            None,
        )
        .await,
    )
    .await;
    assert_eq!(budget_status, axum::http::StatusCode::OK);
    assert_eq!(budget["funding"]["amount"], "11000");
    let wallet_balance = sqlx::query_scalar::<_, rust_decimal::Decimal>(
        "SELECT COALESCE(sum(o.amount),0) + COALESCE((SELECT sum(e.amount) \
         FROM financial_wallet_effects e WHERE e.workspace_id=o.workspace_id AND e.wallet_id=o.wallet_id \
         AND e.opening_treatment IS DISTINCT FROM 'preserve_balance'),0) \
         FROM financial_openings o WHERE o.workspace_id=$1 AND o.wallet_id=$2 GROUP BY o.workspace_id,o.wallet_id",
    )
    .bind(workspace)
    .bind(Uuid::parse_str(wallet_id).unwrap())
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(wallet_balance.to_string(), "100");
    let effect_treatment: String = sqlx::query_scalar(
        "SELECT opening_treatment FROM financial_wallet_effects \
         WHERE workspace_id=$1 AND transaction_id=$2",
    )
    .bind(workspace)
    .bind(Uuid::parse_str(transaction_receipt["resource_id"].as_str().unwrap()).unwrap())
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(effect_treatment, "preserve_balance");
}

#[sqlx::test]
async fn revaluation_preserves_negative_foreign_opening_clp_basis(pool: PgPool) {
    let workspace = bootstrap(&pool).await.unwrap();
    let period = current_period(&pool).await;
    let flow_date = period.succ_opt().unwrap();
    let as_of: NaiveDate =
        sqlx::query_scalar("SELECT (now() AT TIME ZONE 'America/Santiago')::date")
            .fetch_one(&pool)
            .await
            .unwrap();
    let (account_status, receipt) = json_response(
        send(
            pool.clone(),
            workspace,
            "POST",
            "/api/accounts",
            Some(Uuid::new_v4()),
            Some(
                json!({
                    "name": "Foreign liability that became an asset",
                    "account_type": "card",
                    "base_currency": "ZZZ",
                    "budget_eligible": true,
                    "wallets": [{
                        "currency": "ZZZ",
                        "opening_amount": "-100",
                        "opening_date": period.format("%Y-%m-%d").to_string(),
                        "liability_clp_basis": "-9000"
                    }]
                })
                .to_string(),
            ),
        )
        .await,
    )
    .await;
    assert_eq!(account_status, axum::http::StatusCode::CREATED);
    let (_read_status, account) = json_response(
        send(
            pool.clone(),
            workspace,
            "GET",
            &format!("/api/accounts/{}", receipt["resource_id"].as_str().unwrap()),
            None,
            None,
        )
        .await,
    )
    .await;
    let wallet_id = account["wallets"][0]["id"].as_str().unwrap();

    for (date, rate) in [(period, 100), (flow_date, 100), (as_of, 110)] {
        sqlx::query(
            "INSERT INTO daily_fx_observations \
             (id, workspace_id, source, base_currency, quote_currency, effective_date, rate) \
             VALUES ($1, $2, 'Frankfurter v2', 'ZZZ', 'CLP', $3, $4)",
        )
        .bind(Uuid::new_v4())
        .bind(workspace)
        .bind(date)
        .bind(rust_decimal::Decimal::from(rate))
        .execute(&pool)
        .await
        .unwrap();
    }
    let (income_status, _) = json_response(
        send(
            pool.clone(),
            workspace,
            "POST",
            "/api/transactions",
            Some(Uuid::new_v4()),
            Some(
                json!({
                    "kind": "income",
                    "business_date": flow_date.format("%Y-%m-%d").to_string(),
                    "wallet_id": wallet_id,
                    "amount": "200"
                })
                .to_string(),
            ),
        )
        .await,
    )
    .await;
    assert_eq!(income_status, axum::http::StatusCode::CREATED);

    let period_text = period.format("%Y-%m").to_string();
    let (status, budget) = json_response(
        send(
            pool.clone(),
            workspace,
            "GET",
            &format!("/api/budgets/{period_text}"),
            None,
            None,
        )
        .await,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(budget["funding"]["amount"], "11000");
    let valuation_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM budget_funding_effects \
         WHERE workspace_id=$1 AND source_kind='valuation' AND wallet_id=$2",
    )
    .bind(workspace)
    .bind(Uuid::parse_str(wallet_id).unwrap())
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(valuation_count, 0);
}

#[sqlx::test]
async fn period_fingerprint_detects_transaction_edits_and_other_category_assignments(pool: PgPool) {
    let workspace = bootstrap(&pool).await.unwrap();
    let period = current_period(&pool).await;
    let wallet = create_account(&pool, workspace, period).await;
    let day = period.succ_opt().unwrap();
    let (_status, bootstrap_view) =
        json_response(send(pool.clone(), workspace, "GET", "/api/bootstrap", None, None).await)
            .await;
    let categories = bootstrap_view["categories"].as_array().unwrap();
    let food: Uuid = categories
        .iter()
        .find(|category| category["slug"] == "alimentacion")
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    let transport: Uuid = categories
        .iter()
        .find(|category| category["slug"] == "transporte")
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    let (create_status, transaction_receipt) = json_response(
        send(
            pool.clone(),
            workspace,
            "POST",
            "/api/transactions",
            Some(Uuid::new_v4()),
            Some(
                json!({
                    "kind": "expense",
                    "business_date": day.format("%Y-%m-%d").to_string(),
                    "wallet_id": wallet,
                    "amount": "1000",
                    "category_id": food
                })
                .to_string(),
            ),
        )
        .await,
    )
    .await;
    assert_eq!(create_status, axum::http::StatusCode::CREATED);

    let period_text = period.format("%Y-%m").to_string();
    let (_status, before_edit) = json_response(
        send(
            pool.clone(),
            workspace,
            "GET",
            &format!("/api/budgets/{period_text}"),
            None,
            None,
        )
        .await,
    )
    .await;
    let old_revision = before_edit["revision"].as_str().unwrap().to_owned();
    let (edit_status, edit_response) = json_response(
        send(
            pool.clone(),
            workspace,
            "PUT",
            &format!(
                "/api/transactions/{}",
                transaction_receipt["resource_id"].as_str().unwrap()
            ),
            Some(Uuid::new_v4()),
            Some(
                json!({
                    "kind": "expense",
                    "business_date": day.format("%Y-%m-%d").to_string(),
                    "wallet_id": wallet,
                    "amount": "1000",
                    "category_id": food,
                    "note": "corrected note",
                    "expected_revision": 1,
                    "reason": "Corrected transaction note"
                })
                .to_string(),
            ),
        )
        .await,
    )
    .await;
    assert_eq!(
        edit_status,
        axum::http::StatusCode::OK,
        "transaction edit failed: {edit_response}"
    );

    let stale_assignment = |category: Uuid, key: Uuid, expected_period_revision: String| {
        let pool = pool.clone();
        let uri = format!("/api/budgets/{period_text}/assignments/{category}");
        let body = json!({
            "amount": "1000",
            "expected_period_revision": expected_period_revision
        })
        .to_string();
        async move { send(pool, workspace, "PUT", &uri, Some(key), Some(body)).await }
    };
    let (stale_edit_status, _) =
        json_response(stale_assignment(food, Uuid::new_v4(), old_revision.clone()).await).await;
    assert_eq!(stale_edit_status, axum::http::StatusCode::CONFLICT);

    let (_status, after_edit) = json_response(
        send(
            pool.clone(),
            workspace,
            "GET",
            &format!("/api/budgets/{period_text}"),
            None,
            None,
        )
        .await,
    )
    .await;
    let revision_before_assignments = after_edit["revision"].as_str().unwrap().to_owned();
    let (first_status, first_assignment) =
        json_response(stale_assignment(food, Uuid::new_v4(), revision_before_assignments).await)
            .await;
    assert_eq!(first_status, axum::http::StatusCode::OK);
    let (second_status, second_assignment) = json_response(
        stale_assignment(
            transport,
            Uuid::new_v4(),
            first_assignment["period_revision"]
                .as_str()
                .unwrap()
                .to_owned(),
        )
        .await,
    )
    .await;
    assert_eq!(second_status, axum::http::StatusCode::OK);
    assert!(second_assignment["period_revision"].as_str().is_some());
    let (stale_assignment_status, _) = json_response(
        send(
            pool.clone(),
            workspace,
            "PUT",
            &format!("/api/budgets/{period_text}/assignments/{food}"),
            Some(Uuid::new_v4()),
            Some(
                json!({
                    "amount": "2000",
                    "expected_period_revision": first_assignment["period_revision"],
                    "expected_revision": 1
                })
                .to_string(),
            ),
        )
        .await,
    )
    .await;
    assert_eq!(stale_assignment_status, axum::http::StatusCode::CONFLICT);
}
