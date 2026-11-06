use std::time::Duration;

use axum::{body::Body, http::Request};
use chrono::Datelike;
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

use lukas_api::{config::Config, health::router, workspace::bootstrap};

fn config() -> Config {
    Config {
        database_url: "postgres://lukas:lukas@localhost/lukas".to_owned(),
        canonical_origin: "http://localhost:4173".to_owned(),
        bind_address: "127.0.0.1:8080".parse().unwrap(),
        max_connections: 5,
        connect_timeout: Duration::from_secs(5),
        acquire_timeout: Duration::from_secs(1),
        database_startup_timeout: Duration::from_secs(30),
        shutdown_timeout: Duration::from_secs(10),
    }
}

async fn response_json(response: axum::response::Response) -> (axum::http::StatusCode, Value) {
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    (status, serde_json::from_slice(&bytes).unwrap())
}

async fn installation_generation(pool: &PgPool, workspace_id: Uuid) -> String {
    sqlx::query_scalar("SELECT installation_generation::text FROM workspaces WHERE id=$1")
        .bind(workspace_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn post_account(
    pool: PgPool,
    workspace_id: Uuid,
    key: Uuid,
    body: String,
) -> axum::response::Response {
    let generation = installation_generation(&pool, workspace_id).await;
    router(pool, workspace_id, config())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/accounts")
                .header("Origin", "http://localhost:4173")
                .header("Idempotency-Key", key.to_string())
                .header("Installation-Generation", generation)
                .header("Content-Type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap()
}

async fn send_json(
    pool: PgPool,
    workspace_id: Uuid,
    method: &str,
    uri: &str,
    body: String,
) -> axum::response::Response {
    let generation = installation_generation(&pool, workspace_id).await;
    router(pool, workspace_id, config())
        .oneshot(
            Request::builder()
                .method(method)
                .uri(uri)
                .header("Origin", "http://localhost:4173")
                .header("Idempotency-Key", Uuid::new_v4().to_string())
                .header("Installation-Generation", generation)
                .header("Content-Type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap()
}

#[sqlx::test]
async fn account_creation_is_atomic_exactly_retriable_and_reads_wallet_balances(pool: PgPool) {
    let workspace_id = bootstrap(&pool).await.unwrap();
    let bootstrap_response = router(pool.clone(), workspace_id, config())
        .oneshot(
            Request::builder()
                .uri("/api/bootstrap")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let (status, bootstrap) = response_json(bootstrap_response).await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(bootstrap["workspace_id"], workspace_id.to_string());
    assert!(
        !bootstrap["installation_generation"]
            .as_str()
            .unwrap()
            .is_empty()
    );
    assert_eq!(bootstrap["categories"].as_array().unwrap().len(), 15);

    let key = Uuid::new_v4();
    let body = json!({
        "name": "Cuenta principal",
        "account_type": "card",
        "base_currency": "CLP",
        "budget_eligible": true,
        "wallets": [
            {"currency": "CLP", "opening_amount": "125000", "opening_date": "2026-11-05"},
            {"currency": "USD", "opening_amount": "-10.50", "opening_date": "2026-11-05", "liability_clp_basis": "-9500"}
        ]
    })
    .to_string();

    let (status, receipt) =
        response_json(post_account(pool.clone(), workspace_id, key, body.clone()).await).await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    let account_id = receipt["resource_id"].as_str().unwrap();
    assert_eq!(receipt["revision"], 1);

    let replay = post_account(pool.clone(), workspace_id, key, body.clone()).await;
    let (replay_status, replay_body) = response_json(replay).await;
    assert_eq!(replay_status, axum::http::StatusCode::CREATED);
    assert_eq!(replay_body, receipt);

    let changed_body = body.replace("125000", "125001");
    let (changed_status, changed) =
        response_json(post_account(pool.clone(), workspace_id, key, changed_body).await).await;
    assert_eq!(changed_status, axum::http::StatusCode::CONFLICT);
    assert_eq!(changed["error"]["code"], "idempotency_key_reused");

    let view = router(pool.clone(), workspace_id, config())
        .oneshot(
            Request::builder()
                .uri(format!("/api/accounts/{account_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let (view_status, view) = response_json(view).await;
    assert_eq!(view_status, axum::http::StatusCode::OK);
    assert_eq!(view["wallets"].as_array().unwrap().len(), 2);
    assert_eq!(view["wallets"][0]["balance"], "125000");
    assert_eq!(view["wallets"][1]["balance"], "-10.50");
    assert_eq!(view["wallets"][0]["openings"][0]["amount"], "125000");
    assert_eq!(
        view["wallets"][0]["openings"][0]["business_date"],
        "2026-11-05"
    );
    assert_eq!(
        view["wallets"][1]["openings"][0]["liability_clp_basis"],
        "-9500"
    );
    assert_eq!(
        view["wallets"][1]["openings"][0]["liability_basis_provenance"],
        "owner-confirmed"
    );

    let list = router(pool.clone(), workspace_id, config())
        .oneshot(
            Request::builder()
                .uri("/api/accounts")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let (status, list) = response_json(list).await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(list.as_array().unwrap().len(), 1);
    assert_eq!(list[0]["wallets"].as_array().unwrap().len(), 2);

    let accounts: i64 =
        sqlx::query_scalar("SELECT count(*) FROM financial_accounts WHERE workspace_id = $1")
            .bind(workspace_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    let openings: i64 =
        sqlx::query_scalar("SELECT count(*) FROM financial_openings WHERE workspace_id = $1")
            .bind(workspace_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    let bases: i64 =
        sqlx::query_scalar("SELECT count(*) FROM foreign_liability_bases WHERE workspace_id = $1")
            .bind(workspace_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!((accounts, openings, bases), (1, 2, 1));
}

#[sqlx::test]
async fn stale_installation_generation_is_rejected_before_account_write(pool: PgPool) {
    let workspace_id = bootstrap(&pool).await.unwrap();
    let body = json!({
        "name": "Must not be created",
        "account_type": "cash",
        "base_currency": "CLP",
        "budget_eligible": true,
        "wallets": [{"currency": "CLP", "opening_amount": "0", "opening_date": "2026-11-05"}]
    })
    .to_string();
    let response = router(pool.clone(), workspace_id, config())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/accounts")
                .header("Origin", "http://localhost:4173")
                .header("Idempotency-Key", Uuid::new_v4().to_string())
                .header(
                    "Installation-Generation",
                    "d42f0933-8af2-41f1-96c5-e21459fbb0a2",
                )
                .header("Content-Type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    let (status, error) = response_json(response).await;
    assert_eq!(status, axum::http::StatusCode::CONFLICT);
    assert_eq!(error["error"]["code"], "installation_generation_mismatch");
    let account_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM financial_accounts WHERE workspace_id=$1")
            .bind(workspace_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(account_count, 0);
}

#[sqlx::test]
async fn invalid_account_input_rolls_back_and_investments_default_to_tracking_only(pool: PgPool) {
    let workspace_id = bootstrap(&pool).await.unwrap();
    let invalid = json!({
        "name": "Invalid debt",
        "account_type": "card",
        "base_currency": "CLP",
        "budget_eligible": true,
        "wallets": [
            {"currency": "USD", "opening_amount": "-10", "opening_date": "2026-11-05"}
        ]
    })
    .to_string();
    let (status, error) =
        response_json(post_account(pool.clone(), workspace_id, Uuid::new_v4(), invalid).await)
            .await;
    assert_eq!(status, axum::http::StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        error["error"]["fields"][0]["code"],
        "required_for_foreign_liability"
    );

    let count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM financial_accounts WHERE workspace_id = $1")
            .bind(workspace_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(count, 0);

    let wrong_sign = json!({
        "name": "Wrong basis sign",
        "account_type": "card",
        "base_currency": "CLP",
        "budget_eligible": true,
        "wallets": [
            {"currency": "USD", "opening_amount": "-10", "opening_date": "2026-11-05", "liability_clp_basis": "9000"}
        ]
    })
    .to_string();
    let (status, error) =
        response_json(post_account(pool.clone(), workspace_id, Uuid::new_v4(), wrong_sign).await)
            .await;
    assert_eq!(status, axum::http::StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        error["error"]["fields"][0]["code"],
        "must_match_liability_sign"
    );

    let investment = json!({
        "name": "Inversiones",
        "account_type": "investment",
        "base_currency": "CLP",
        "wallets": [
            {"currency": "CLP", "opening_amount": "0", "opening_date": "2026-11-05"}
        ]
    })
    .to_string();
    let (status, receipt) =
        response_json(post_account(pool.clone(), workspace_id, Uuid::new_v4(), investment).await)
            .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);

    let account = sqlx::query_as::<_, (bool,)>(
        "SELECT budget_eligible FROM financial_accounts WHERE workspace_id = $1 AND id = $2",
    )
    .bind(workspace_id)
    .bind(Uuid::parse_str(receipt["resource_id"].as_str().unwrap()).unwrap())
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(!account.0);
}

#[sqlx::test]
async fn only_banks_savings_and_investments_are_single_currency(pool: PgPool) {
    let workspace_id = bootstrap(&pool).await.unwrap();
    let account_count_before: i64 =
        sqlx::query_scalar("SELECT count(*) FROM financial_accounts WHERE workspace_id = $1")
            .bind(workspace_id)
            .fetch_one(&pool)
            .await
            .unwrap();

    for account_type in ["bank", "savings", "investment"] {
        let request = json!({
            "name": format!("Invalid {account_type}"),
            "account_type": account_type,
            "base_currency": "CLP",
            "budget_eligible": true,
            "wallets": [
                {"currency": "CLP", "opening_amount": "0", "opening_date": "2026-11-05"},
                {"currency": "USD", "opening_amount": "0", "opening_date": "2026-11-05"}
            ]
        })
        .to_string();
        let (status, error) =
            response_json(post_account(pool.clone(), workspace_id, Uuid::new_v4(), request).await)
                .await;
        assert_eq!(status, axum::http::StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(error["error"]["fields"][0]["field"], "wallets");
        assert_eq!(
            error["error"]["fields"][0]["code"],
            "multiple_currencies_unsupported_for_account_type"
        );
    }

    let account_count_after_invalid: i64 =
        sqlx::query_scalar("SELECT count(*) FROM financial_accounts WHERE workspace_id = $1")
            .bind(workspace_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(account_count_after_invalid, account_count_before);

    let cash_multi = json!({
        "name": "Efectivo multimoneda",
        "account_type": "cash",
        "base_currency": "CLP",
        "budget_eligible": true,
        "wallets": [
            {"currency": "CLP", "opening_amount": "10000", "opening_date": "2026-11-05"},
            {"currency": "USD", "opening_amount": "20", "opening_date": "2026-11-05"}
        ]
    })
    .to_string();
    let (cash_status, cash_receipt) =
        response_json(post_account(pool.clone(), workspace_id, Uuid::new_v4(), cash_multi).await)
            .await;
    assert_eq!(cash_status, axum::http::StatusCode::CREATED);
    let cash_id = cash_receipt["resource_id"].as_str().unwrap();
    let cash_response = router(pool.clone(), workspace_id, config())
        .oneshot(
            Request::builder()
                .uri(format!("/api/accounts/{cash_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let (cash_read_status, cash_account) = response_json(cash_response).await;
    assert_eq!(cash_read_status, axum::http::StatusCode::OK);
    assert_eq!(cash_account["wallets"].as_array().unwrap().len(), 2);
    assert_eq!(cash_account["wallets"][0]["currency"], "CLP");
    assert_eq!(cash_account["wallets"][1]["currency"], "USD");

    let mut cash_account_ids = Vec::new();
    for currency in ["CLP", "USD"] {
        let request = json!({
            "name": "Efectivo",
            "account_type": "cash",
            "base_currency": currency,
            "budget_eligible": true,
            "wallets": [{
                "currency": currency,
                "opening_amount": "0",
                "opening_date": "2026-11-05"
            }]
        })
        .to_string();
        let (status, receipt) =
            response_json(post_account(pool.clone(), workspace_id, Uuid::new_v4(), request).await)
                .await;
        assert_eq!(status, axum::http::StatusCode::CREATED);
        cash_account_ids.push(receipt["resource_id"].as_str().unwrap().to_owned());
    }

    for (id, expected_currency) in cash_account_ids.iter().zip(["CLP", "USD"]) {
        let response = router(pool.clone(), workspace_id, config())
            .oneshot(
                Request::builder()
                    .uri(format!("/api/accounts/{id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let (status, account) = response_json(response).await;
        assert_eq!(status, axum::http::StatusCode::OK);
        assert_eq!(account["name"], "Efectivo");
        assert_eq!(account["wallets"].as_array().unwrap().len(), 1);
        assert_eq!(account["wallets"][0]["currency"], expected_currency);
    }
}

#[sqlx::test]
async fn account_edit_uses_revision_and_idempotency_receipt(pool: PgPool) {
    let workspace_id = bootstrap(&pool).await.unwrap();
    let body = json!({
        "name": "Cuenta",
        "account_type": "bank",
        "base_currency": "CLP",
        "budget_eligible": true,
        "wallets": [
            {"currency": "CLP", "opening_amount": "100", "opening_date": "2026-11-05"}
        ]
    })
    .to_string();
    let (_, receipt) =
        response_json(post_account(pool.clone(), workspace_id, Uuid::new_v4(), body).await).await;
    let account_id = receipt["resource_id"].as_str().unwrap();
    let edit_key = Uuid::new_v4();
    let generation = installation_generation(&pool, workspace_id).await;
    let edit_body = json!({"name":"Cuenta renombrada","expected_revision":1}).to_string();
    let edit_request = || {
        Request::builder()
            .method("PUT")
            .uri(format!("/api/accounts/{account_id}"))
            .header("Origin", "http://localhost:4173")
            .header("Idempotency-Key", edit_key.to_string())
            .header("Installation-Generation", generation.clone())
            .header("Content-Type", "application/json")
            .body(Body::from(edit_body.clone()))
            .unwrap()
    };
    let edit = router(pool.clone(), workspace_id, config())
        .oneshot(edit_request())
        .await
        .unwrap();
    let (status, edit_receipt) = response_json(edit).await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(edit_receipt["revision"], 2);

    let replay = router(pool.clone(), workspace_id, config())
        .oneshot(edit_request())
        .await
        .unwrap();
    let (status, replay_receipt) = response_json(replay).await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(replay_receipt, edit_receipt);

    let stale = router(pool.clone(), workspace_id, config())
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/accounts/{account_id}"))
                .header("Origin", "http://localhost:4173")
                .header("Idempotency-Key", Uuid::new_v4().to_string())
                .header("Installation-Generation", generation.clone())
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({"name":"Stale","expected_revision":1}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let (status, stale) = response_json(stale).await;
    assert_eq!(status, axum::http::StatusCode::CONFLICT);
    assert_eq!(stale["error"]["code"], "stale_revision");
}

#[sqlx::test]
async fn foreign_opening_correction_requires_and_updates_signed_basis_atomically(pool: PgPool) {
    let workspace_id = bootstrap(&pool).await.unwrap();
    let generation = installation_generation(&pool, workspace_id).await;
    let body = json!({
        "name": "Tarjeta en USD",
        "account_type": "card",
        "base_currency": "CLP",
        "budget_eligible": true,
        "wallets": [
            {"currency": "USD", "opening_amount": "-10", "opening_date": "2026-11-05", "liability_clp_basis": "-9000"}
        ]
    })
    .to_string();
    let (_, receipt) =
        response_json(post_account(pool.clone(), workspace_id, Uuid::new_v4(), body).await).await;
    let account_id = Uuid::parse_str(receipt["resource_id"].as_str().unwrap()).unwrap();
    let opening_id: Uuid =
        sqlx::query_scalar("SELECT id FROM financial_openings WHERE workspace_id = $1")
            .bind(workspace_id)
            .fetch_one(&pool)
            .await
            .unwrap();

    let incomplete = json!({
        "amount": "-12",
        "expected_revision": 1,
        "reason": "statement correction"
    })
    .to_string();
    let incomplete_response = router(pool.clone(), workspace_id, config())
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/accounts/{account_id}/openings/{opening_id}"))
                .header("Origin", "http://localhost:4173")
                .header("Idempotency-Key", Uuid::new_v4().to_string())
                .header("Installation-Generation", generation.clone())
                .body(Body::from(incomplete))
                .unwrap(),
        )
        .await
        .unwrap();
    let (status, error) = response_json(incomplete_response).await;
    assert_eq!(status, axum::http::StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        error["error"]["fields"][0]["code"],
        "required_for_principal_correction"
    );

    let correction = json!({
        "amount": "-12",
        "expected_revision": 1,
        "expected_basis_revision": 1,
        "liability_clp_basis": "-10800",
        "reason": "statement correction"
    })
    .to_string();
    let correction_response = router(pool.clone(), workspace_id, config())
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/accounts/{account_id}/openings/{opening_id}"))
                .header("Origin", "http://localhost:4173")
                .header("Idempotency-Key", Uuid::new_v4().to_string())
                .header("Installation-Generation", generation.clone())
                .body(Body::from(correction))
                .unwrap(),
        )
        .await
        .unwrap();
    let (status, response) = response_json(correction_response).await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(response["revision"], 2);

    let view = router(pool.clone(), workspace_id, config())
        .oneshot(
            Request::builder()
                .uri(format!("/api/accounts/{account_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let (status, view) = response_json(view).await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(view["wallets"][0]["openings"][0]["amount"], "-12");
    assert_eq!(
        view["wallets"][0]["openings"][0]["liability_clp_basis"],
        "-10800"
    );
    assert_eq!(view["wallets"][0]["openings"][0]["revision"], 2);
    assert_eq!(
        view["wallets"][0]["openings"][0]["liability_basis_revision"],
        2
    );

    let corrections: i64 =
        sqlx::query_scalar("SELECT count(*) FROM financial_corrections WHERE workspace_id = $1")
            .bind(workspace_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(corrections, 2);
}

#[sqlx::test]
async fn closed_foreign_opening_correction_uses_frozen_close_rate_baseline(pool: PgPool) {
    let workspace_id = bootstrap(&pool).await.unwrap();
    let opening_date = chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let create = json!({
        "name": "USD savings",
        "account_type": "bank",
        "base_currency": "USD",
        "budget_eligible": true,
        "wallets": [{
            "currency": "USD",
            "opening_amount": "100",
            "opening_date": opening_date.format("%Y-%m-%d").to_string()
        }]
    })
    .to_string();
    let (create_status, receipt) =
        response_json(post_account(pool.clone(), workspace_id, Uuid::new_v4(), create).await).await;
    assert_eq!(create_status, axum::http::StatusCode::CREATED);
    let account_id = Uuid::parse_str(receipt["resource_id"].as_str().unwrap()).unwrap();
    let opening_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM financial_openings WHERE workspace_id=$1 ORDER BY id LIMIT 1",
    )
    .bind(workspace_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO daily_fx_observations(id,workspace_id,source,base_currency,quote_currency,effective_date,rate) \
         VALUES($1,$2,'Frankfurter v2','USD','CLP',$3,900)",
    )
    .bind(Uuid::new_v4())
    .bind(workspace_id)
    .bind(opening_date)
    .execute(&pool)
    .await
    .unwrap();

    let (preview_status, preview) = response_json(
        send_json(
            pool.clone(),
            workspace_id,
            "GET",
            "/api/budgets/2026-01/close-preview",
            "{}".to_owned(),
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
        .map(|category| {
            json!({
                "category_id": category["category_id"],
                "disposition": "carry"
            })
        })
        .collect::<Vec<_>>();
    let close_body = json!({
        "expected_period_revision": preview["close"]["expected_period_revision"],
        "categories": categories
    })
    .to_string();
    let (close_status, _) = response_json(
        send_json(
            pool.clone(),
            workspace_id,
            "POST",
            "/api/budgets/2026-01/close",
            close_body,
        )
        .await,
    )
    .await;
    assert_eq!(close_status, axum::http::StatusCode::CREATED);
    let frozen_baseline: rust_decimal::Decimal = sqlx::query_scalar(
        "SELECT funding_amount FROM budget_close_opening_snapshots WHERE workspace_id=$1 AND opening_id=$2",
    )
    .bind(workspace_id)
    .bind(opening_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(frozen_baseline, rust_decimal::Decimal::from(90_000));

    // A corrected observation for the same historical date must not rewrite
    // the closed-period baseline used to calculate this later correction.
    sqlx::query(
        "INSERT INTO daily_fx_observations(id,workspace_id,source,base_currency,quote_currency,effective_date,rate) \
         VALUES($1,$2,'Frankfurter v2','USD','CLP',$3,1000)",
    )
    .bind(Uuid::new_v4())
    .bind(workspace_id)
    .bind(opening_date)
    .execute(&pool)
    .await
    .unwrap();
    let corrected_opening = json!({
        "amount": "200",
        "expected_revision": 1,
        "reason": "Correct opening statement"
    })
    .to_string();
    let (correction_status, _) = response_json(
        send_json(
            pool.clone(),
            workspace_id,
            "PUT",
            &format!("/api/accounts/{account_id}/openings/{opening_id}"),
            corrected_opening,
        )
        .await,
    )
    .await;
    assert_eq!(correction_status, axum::http::StatusCode::OK);
    let correction_effect: rust_decimal::Decimal = sqlx::query_scalar(
        "SELECT amount FROM budget_funding_effects WHERE workspace_id=$1 AND source_kind='correction' AND correction_id=( \
           SELECT id FROM financial_corrections WHERE workspace_id=$1 AND opening_id=$2 ORDER BY new_revision DESC LIMIT 1)",
    )
    .bind(workspace_id)
    .bind(opening_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(correction_effect, rust_decimal::Decimal::from(110_000));

    sqlx::query(
        "INSERT INTO daily_fx_observations(id,workspace_id,source,base_currency,quote_currency,effective_date,rate,observed_at) \
         VALUES($1,$2,'Frankfurter v2','USD','CLP',$3,1100,clock_timestamp()+interval '1 second')",
    )
    .bind(Uuid::new_v4())
    .bind(workspace_id)
    .bind(opening_date)
    .execute(&pool)
    .await
    .unwrap();
    let current_period: chrono::NaiveDate = sqlx::query_scalar(
        "SELECT date_trunc('month',now() AT TIME ZONE 'America/Santiago')::date",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    let (budget_status, _) = response_json(
        send_json(
            pool.clone(),
            workspace_id,
            "GET",
            &format!("/api/budgets/{}", current_period.format("%Y-%m")),
            "{}".to_owned(),
        )
        .await,
    )
    .await;
    assert_eq!(budget_status, axum::http::StatusCode::OK);
    let refreshed_effect: rust_decimal::Decimal = sqlx::query_scalar(
        "SELECT amount FROM budget_funding_effects WHERE workspace_id=$1 AND source_kind='correction' AND correction_id=( \
           SELECT id FROM financial_corrections WHERE workspace_id=$1 AND opening_id=$2 ORDER BY new_revision DESC LIMIT 1)",
    )
    .bind(workspace_id)
    .bind(opening_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(refreshed_effect, rust_decimal::Decimal::from(130_000));

    sqlx::query(
        "INSERT INTO budget_close_snapshots \
         (id,workspace_id,period_start,revision,funding,ready_to_assign,activity,uncategorized_activity) \
         VALUES($1,$2,$3,1,0,0,0,0)",
    )
    .bind(Uuid::new_v4())
    .bind(workspace_id)
    .bind(current_period)
    .execute(&pool)
    .await
    .unwrap();
    let next_opening_correction = json!({
        "amount": "250",
        "expected_revision": 2,
        "reason": "Correct opening after correction month close"
    })
    .to_string();
    let (next_status, _) = response_json(
        send_json(
            pool.clone(),
            workspace_id,
            "PUT",
            &format!("/api/accounts/{account_id}/openings/{opening_id}"),
            next_opening_correction,
        )
        .await,
    )
    .await;
    assert_eq!(next_status, axum::http::StatusCode::OK);
    let next_period = if current_period.month() == 12 {
        chrono::NaiveDate::from_ymd_opt(current_period.year() + 1, 1, 1).unwrap()
    } else {
        chrono::NaiveDate::from_ymd_opt(current_period.year(), current_period.month() + 1, 1)
            .unwrap()
    };
    sqlx::query(
        "INSERT INTO daily_fx_observations(id,workspace_id,source,base_currency,quote_currency,effective_date,rate,observed_at) \
         VALUES($1,$2,'Frankfurter v2','USD','CLP',$3,1200,clock_timestamp()+interval '2 seconds')",
    )
    .bind(Uuid::new_v4())
    .bind(workspace_id)
    .bind(opening_date)
    .execute(&pool)
    .await
    .unwrap();
    let (next_budget_status, _) = response_json(
        send_json(
            pool.clone(),
            workspace_id,
            "GET",
            &format!("/api/budgets/{}", next_period.format("%Y-%m")),
            "{}".to_owned(),
        )
        .await,
    )
    .await;
    assert_eq!(next_budget_status, axum::http::StatusCode::OK);
    let next_effect: rust_decimal::Decimal = sqlx::query_scalar(
        "SELECT amount FROM budget_funding_effects WHERE workspace_id=$1 AND source_kind='correction' AND correction_id=( \
           SELECT id FROM financial_corrections WHERE workspace_id=$1 AND opening_id=$2 ORDER BY new_revision DESC LIMIT 1)",
    )
    .bind(workspace_id)
    .bind(opening_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(next_effect, rust_decimal::Decimal::from(80_000));
}
