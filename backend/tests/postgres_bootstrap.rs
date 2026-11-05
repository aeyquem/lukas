use axum::{body::Body, http::Request};
use crc::{CRC_32_ISO_HDLC, Crc};
use sqlx::{PgPool, Row, postgres::PgPoolOptions};
use std::time::Duration;
use tower::ServiceExt;
use uuid::Uuid;

use lukas_api::{
    config::Config,
    database::{StartupDeadlineExceeded, run_migrations, with_startup_deadline},
    health::router,
    workspace::bootstrap,
};

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

#[sqlx::test]
async fn migration_runner_applies_in_order_and_is_safe_to_rerun(pool: PgPool) {
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

    assert_eq!(versions.len(), 2);
    assert!(versions[0] < versions[1]);

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

#[sqlx::test(migrations = "./tests/fixtures/migrations_v1")]
async fn database_at_previous_version_receives_current_category_migration(pool: PgPool) {
    let categories_before: bool = sqlx::query_scalar(
        "SELECT EXISTS (\
            SELECT 1 FROM information_schema.tables \
            WHERE table_schema = 'public' AND table_name = 'expense_categories'\
         )",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(!categories_before);

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

    let migration_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(migration_count, 2);
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
