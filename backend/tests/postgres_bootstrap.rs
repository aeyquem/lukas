use axum::{body::Body, http::Request};
use chrono::NaiveDate;
use crc::{CRC_32_ISO_HDLC, Crc};
use sqlx::{PgPool, Row, postgres::PgPoolOptions};
use std::time::Duration;
use tower::ServiceExt;
use uuid::Uuid;

use lukas_api::{
    config::Config,
    database::{
        StartupDeadlineExceeded, rotate_installation_generation, run_migrations,
        with_startup_deadline,
    },
    health::router,
    persistence::{
        fetch_accounts, fetch_assignments, fetch_categories, fetch_close_category_snapshots,
        fetch_close_snapshot, fetch_corrections, fetch_funding_effects, fetch_fx_observations,
        fetch_liability_basis, fetch_openings, fetch_transactions_page, fetch_wallet_effects,
        fetch_wallets, find_idempotency_receipt,
    },
    workspace::bootstrap,
};

#[sqlx::test(migrations = "./migrations")]
async fn restore_generation_rotation_invalidates_the_previous_token(pool: PgPool) {
    let workspace_id = bootstrap(&pool).await.unwrap();
    let original: Uuid =
        sqlx::query_scalar("SELECT installation_generation FROM workspaces WHERE id=$1")
            .bind(workspace_id)
            .fetch_one(&pool)
            .await
            .unwrap();

    let rotated = rotate_installation_generation(&pool, workspace_id)
        .await
        .unwrap();
    assert_ne!(rotated, original);
    let stored: Uuid =
        sqlx::query_scalar("SELECT installation_generation FROM workspaces WHERE id=$1")
            .bind(workspace_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(stored, rotated);
}

#[sqlx::test(migrations = "./migrations")]
async fn workspace_bootstrap_is_repeatable_and_preserves_owner_edits(pool: PgPool) {
    let workspace_id = bootstrap(&pool).await.unwrap();
    let second_workspace_id = bootstrap(&pool).await.unwrap();

    assert_eq!(workspace_id, second_workspace_id);

    sqlx::query("UPDATE expense_categories SET name = 'Casa' WHERE slug = 'vivienda'")
        .execute(&pool)
        .await
        .unwrap();
    bootstrap(&pool).await.unwrap();

    let edited_name: String = sqlx::query_scalar(
        "SELECT name FROM expense_categories WHERE workspace_id = $1 AND slug = 'vivienda'",
    )
    .bind(workspace_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(edited_name, "Casa");

    let workspace_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM workspaces")
        .fetch_one(&pool)
        .await
        .unwrap();
    let category_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM expense_categories")
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(workspace_count, 1);
    assert_eq!(category_count, 15);
}

#[sqlx::test(migrations = "./migrations")]
async fn concurrent_workspace_bootstrap_keeps_one_workspace_and_category_set(pool: PgPool) {
    let (first, second) = tokio::join!(bootstrap(&pool), bootstrap(&pool));
    let first = first.unwrap();
    let second = second.unwrap();

    assert_eq!(first, second);

    let workspace_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM workspaces")
        .fetch_one(&pool)
        .await
        .unwrap();
    let category_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM expense_categories")
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(workspace_count, 1);
    assert_eq!(category_count, 15);
}

#[sqlx::test(migrations = false)]
async fn migration_runner_applies_in_order_and_is_safe_to_rerun(pool: PgPool) {
    let migration_table_before: bool = sqlx::query_scalar(
        "SELECT EXISTS (\
            SELECT 1 FROM information_schema.tables \
            WHERE table_schema = 'public' AND table_name = '_sqlx_migrations'\
         )",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(!migration_table_before);

    run_migrations(&pool).await.unwrap();
    run_migrations(&pool).await.unwrap();

    let migration_rows = sqlx::query("SELECT version FROM _sqlx_migrations ORDER BY version")
        .fetch_all(&pool)
        .await
        .unwrap();
    let versions: Vec<i64> = migration_rows
        .iter()
        .map(|row| row.get("version"))
        .collect();

    assert_eq!(versions.len(), 4);
    assert!(versions.windows(2).all(|pair| pair[0] < pair[1]));

    let categories_table: bool = sqlx::query_scalar(
        "SELECT EXISTS (\
            SELECT 1 FROM information_schema.tables \
            WHERE table_schema = 'public' AND table_name = 'expense_categories'\
         )",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(categories_table);
}

#[sqlx::test(migrations = "./tests/fixtures/migrations_v2")]
async fn database_with_workspace_and_categories_receives_financial_migration(pool: PgPool) {
    let categories_before: bool = sqlx::query_scalar(
        "SELECT EXISTS (\
            SELECT 1 FROM information_schema.tables \
            WHERE table_schema = 'public' AND table_name = 'expense_categories'\
         )",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(categories_before);

    let workspace_id = Uuid::new_v4();
    let category_id = Uuid::new_v4();
    sqlx::query("INSERT INTO workspaces (id, singleton_key) VALUES ($1, 'owner')")
        .bind(workspace_id)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO expense_categories (id, workspace_id, slug, name, sort_order) \
         VALUES ($1, $2, 'vivienda', 'Vivienda editada', 0)",
    )
    .bind(category_id)
    .bind(workspace_id)
    .execute(&pool)
    .await
    .unwrap();

    run_migrations(&pool).await.unwrap();

    let categories_after: bool = sqlx::query_scalar(
        "SELECT EXISTS (\
            SELECT 1 FROM information_schema.tables \
            WHERE table_schema = 'public' AND table_name = 'expense_categories'\
         )",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(categories_after);

    let preserved_category_name: String = sqlx::query_scalar(
        "SELECT name FROM expense_categories WHERE workspace_id = $1 AND id = $2",
    )
    .bind(workspace_id)
    .bind(category_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(preserved_category_name, "Vivienda editada");

    let migration_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(migration_count, 4);

    let financial_tables: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM information_schema.tables \
         WHERE table_schema = 'public' AND table_name = ANY($1)",
    )
    .bind(vec![
        "financial_accounts",
        "financial_wallets",
        "financial_openings",
        "foreign_liability_bases",
        "financial_transactions",
        "financial_wallet_effects",
        "financial_idempotency_receipts",
        "budget_assignments",
        "daily_fx_observations",
        "financial_corrections",
        "budget_funding_effects",
        "budget_close_snapshots",
        "budget_close_category_snapshots",
    ])
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(financial_tables, 13);

    let decimal_boundary_checks: (bool, bool, bool) = sqlx::query_as(
        "SELECT \
            lukas_decimal_in_envelope('79228162514264337593543950335'::NUMERIC), \
            lukas_decimal_in_envelope('0.0000000000000000000000000001'::NUMERIC), \
            lukas_decimal_in_envelope('0.00000000000000000000000000001'::NUMERIC)",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(decimal_boundary_checks, (true, true, false));
}

#[sqlx::test]
async fn financial_persistence_queries_map_workspace_records(pool: PgPool) {
    let workspace_id = bootstrap(&pool).await.unwrap();
    let account_id = Uuid::new_v4();
    let wallet_id = Uuid::new_v4();
    let opening_id = Uuid::new_v4();
    let transaction_id = Uuid::new_v4();
    let income_id = Uuid::new_v4();
    let idempotency_key = Uuid::new_v4();
    let assignment_id = Uuid::new_v4();
    let snapshot_id = Uuid::new_v4();
    let correction_id = Uuid::new_v4();
    let today = NaiveDate::from_ymd_opt(2026, 11, 5).unwrap();
    let period = NaiveDate::from_ymd_opt(2026, 11, 1).unwrap();
    let category_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM expense_categories WHERE workspace_id = $1 AND slug = 'alimentacion'",
    )
    .bind(workspace_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO financial_accounts (id, workspace_id, name, account_type, base_currency, budget_eligible) \
         VALUES ($1, $2, 'Cuenta diaria', 'bank', 'CLP', TRUE)",
    )
    .bind(account_id)
    .bind(workspace_id)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO financial_wallets (id, workspace_id, account_id, currency) VALUES ($1, $2, $3, 'USD')",
    )
    .bind(wallet_id)
    .bind(workspace_id)
    .bind(account_id)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO financial_openings (id, workspace_id, wallet_id, amount, business_date) \
         VALUES ($1, $2, $3, '-12.50', $4)",
    )
    .bind(opening_id)
    .bind(workspace_id)
    .bind(wallet_id)
    .bind(today)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO foreign_liability_bases (id, workspace_id, opening_id, clp_amount, provenance) \
         VALUES ($1, $2, $3, '-11500', 'owner-confirmed')",
    )
    .bind(Uuid::new_v4())
    .bind(workspace_id)
    .bind(opening_id)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO financial_transactions \
         (id, workspace_id, transaction_kind, business_date, category_id, confirmed_clp_amount) \
         VALUES ($1, $2, 'expense', $3, $4, '1000')",
    )
    .bind(transaction_id)
    .bind(workspace_id)
    .bind(today)
    .bind(category_id)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO financial_wallet_effects \
         (id, workspace_id, transaction_id, wallet_id, effect_role, amount) \
         VALUES ($1, $2, $3, $4, 'single', '-1000')",
    )
    .bind(Uuid::new_v4())
    .bind(workspace_id)
    .bind(transaction_id)
    .bind(wallet_id)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO financial_transactions (id, workspace_id, transaction_kind, business_date) \
         VALUES ($1, $2, 'income', $3)",
    )
    .bind(income_id)
    .bind(workspace_id)
    .bind(today)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO financial_idempotency_receipts \
         (workspace_id, idempotency_key, http_method, route, request_hash, response_status, resource_id, resource_revision) \
         VALUES ($1, $2, 'POST', '/api/transactions', decode(repeat('ab', 32), 'hex'), 201, $3, 1)",
    )
    .bind(workspace_id)
    .bind(idempotency_key)
    .bind(income_id)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO budget_assignments (id, workspace_id, period_start, category_id, amount) \
         VALUES ($1, $2, $3, $4, '5000')",
    )
    .bind(assignment_id)
    .bind(workspace_id)
    .bind(period)
    .bind(category_id)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO daily_fx_observations \
         (id, workspace_id, source, base_currency, quote_currency, effective_date, rate) \
         VALUES ($1, $2, 'Frankfurter v2', 'USD', 'CLP', $3, '900.25')",
    )
    .bind(Uuid::new_v4())
    .bind(workspace_id)
    .bind(today)
    .execute(&pool)
    .await
    .unwrap();
    let rate_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM daily_fx_observations WHERE workspace_id = $1 AND quote_currency = 'CLP'",
    )
    .bind(workspace_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO financial_corrections \
         (id, workspace_id, record_kind, transaction_id, prior_revision, new_revision, reason) \
         VALUES ($1, $2, 'transaction', $3, 1, 2, 'category correction')",
    )
    .bind(correction_id)
    .bind(workspace_id)
    .bind(transaction_id)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO budget_funding_effects \
         (id, workspace_id, period_start, source_kind, transaction_id, amount, currency) \
         VALUES ($1, $2, $3, 'income', $4, '750', 'CLP')",
    )
    .bind(Uuid::new_v4())
    .bind(workspace_id)
    .bind(period)
    .bind(income_id)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO budget_close_snapshots (id, workspace_id, period_start, revision, funding, ready_to_assign) \
         VALUES ($1, $2, $3, 1, '750', '250')",
    )
    .bind(snapshot_id)
    .bind(workspace_id)
    .bind(period)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO budget_close_category_snapshots \
         (workspace_id, snapshot_id, category_id, assigned, activity, remaining, disposition) \
         VALUES ($1, $2, $3, '5000', '1000', '4000', 'carry')",
    )
    .bind(workspace_id)
    .bind(snapshot_id)
    .bind(category_id)
    .execute(&pool)
    .await
    .unwrap();

    assert_eq!(fetch_accounts(&pool, workspace_id).await.unwrap().len(), 1);
    assert_eq!(
        fetch_wallets(&pool, workspace_id, account_id)
            .await
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        fetch_openings(&pool, workspace_id, wallet_id)
            .await
            .unwrap()[0]
            .amount
            .to_string(),
        "-12.50"
    );
    assert!(
        fetch_liability_basis(&pool, workspace_id, opening_id)
            .await
            .unwrap()
            .is_some()
    );
    assert_eq!(
        fetch_categories(&pool, workspace_id).await.unwrap().len(),
        15
    );

    let first_page = fetch_transactions_page(&pool, workspace_id, 1, None)
        .await
        .unwrap();
    assert_eq!(first_page.items.len(), 1);
    assert!(first_page.has_more);
    let cursor = (&first_page.items[0].business_date, first_page.items[0].id);
    let second_page = fetch_transactions_page(&pool, workspace_id, 1, Some((*cursor.0, cursor.1)))
        .await
        .unwrap();
    assert_eq!(second_page.items.len(), 1);
    assert!(!second_page.has_more);
    assert_eq!(
        fetch_wallet_effects(&pool, workspace_id, transaction_id)
            .await
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        fetch_assignments(&pool, workspace_id, period)
            .await
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        fetch_funding_effects(&pool, workspace_id, period)
            .await
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        fetch_fx_observations(&pool, workspace_id, "USD", "CLP")
            .await
            .unwrap()[0]
            .id,
        rate_id
    );
    assert!(
        find_idempotency_receipt(&pool, workspace_id, idempotency_key)
            .await
            .unwrap()
            .is_some()
    );
    assert_eq!(
        fetch_corrections(&pool, workspace_id, "transaction", transaction_id)
            .await
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        fetch_close_snapshot(&pool, workspace_id, period)
            .await
            .unwrap()
            .unwrap()
            .id,
        snapshot_id
    );
    assert_eq!(
        fetch_close_category_snapshots(&pool, workspace_id, snapshot_id)
            .await
            .unwrap()
            .len(),
        1
    );
}

#[sqlx::test]
async fn migration_lock_wait_is_bounded_by_startup_deadline(pool: PgPool) {
    let lock_pool = PgPoolOptions::new()
        .max_connections(1)
        .connect_with(pool.connect_options().as_ref().clone())
        .await
        .unwrap();
    let mut lock_connection = lock_pool.acquire().await.unwrap();

    let database_name: String = sqlx::query_scalar("SELECT current_database()")
        .fetch_one(&mut *lock_connection)
        .await
        .unwrap();
    let crc = Crc::<u32>::new(&CRC_32_ISO_HDLC);
    let lock_id = 0x3d32ad9e_i64 * (crc.checksum(database_name.as_bytes()) as i64);

    sqlx::query("SELECT pg_advisory_lock($1)")
        .bind(lock_id)
        .execute(&mut *lock_connection)
        .await
        .unwrap();

    let result = with_startup_deadline(
        &pool,
        std::time::Duration::from_millis(100),
        run_migrations(&pool),
    )
    .await;

    assert!(matches!(result, Err(StartupDeadlineExceeded)));
    drop(lock_connection);
    lock_pool.close().await;
}

#[sqlx::test]
async fn readiness_succeeds_when_database_is_available(pool: PgPool) {
    let config = Config {
        database_url: "postgres://lukas:lukas@localhost/lukas".to_owned(),
        canonical_origin: "http://localhost:4173".to_owned(),
        bind_address: "127.0.0.1:8080".parse().unwrap(),
        max_connections: 5,
        connect_timeout: Duration::from_secs(5),
        acquire_timeout: Duration::from_secs(1),
        database_startup_timeout: Duration::from_secs(30),
        shutdown_timeout: Duration::from_secs(10),
    };
    let app = router(pool, Uuid::nil(), config);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health/ready")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::OK);
}

#[sqlx::test]
async fn api_origin_middleware_checks_the_actual_request_headers(pool: PgPool) {
    let config = Config {
        database_url: "postgres://lukas:lukas@localhost/lukas".to_owned(),
        canonical_origin: "http://localhost:4173".to_owned(),
        bind_address: "127.0.0.1:8080".parse().unwrap(),
        max_connections: 5,
        connect_timeout: Duration::from_secs(5),
        acquire_timeout: Duration::from_secs(1),
        database_startup_timeout: Duration::from_secs(30),
        shutdown_timeout: Duration::from_secs(10),
    };
    let app = router(pool, Uuid::nil(), config);

    let canonical = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/not-a-route")
                .header("Origin", "http://localhost:4173")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(canonical.status(), axum::http::StatusCode::NOT_FOUND);

    let mismatched = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/not-a-route")
                .header("Origin", "http://evil.example")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(mismatched.status(), axum::http::StatusCode::FORBIDDEN);

    let forwarded_only = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/not-a-route")
                .header("X-Forwarded-Origin", "http://localhost:4173")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(forwarded_only.status(), axum::http::StatusCode::FORBIDDEN);

    let health_without_origin = app
        .oneshot(
            Request::builder()
                .uri("/health/live")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(health_without_origin.status(), axum::http::StatusCode::OK);
}
