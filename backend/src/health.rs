use std::time::Duration;

use axum::{
    Json, Router, extract::State, http::StatusCode, middleware, response::IntoResponse,
    routing::get,
};
use serde::Serialize;
use sqlx::PgPool;
use tokio::time::timeout;
use uuid::Uuid;

use crate::{config::Config, origin::enforce_origin};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub canonical_origin: String,
    pub readiness_timeout: Duration,
    // Future workspace-scoped routes receive this ID from shared state.
    #[allow(dead_code)]
    pub workspace_id: Uuid,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

pub fn router(pool: PgPool, workspace_id: Uuid, config: Config) -> Router {
    let state = AppState {
        pool,
        canonical_origin: config.canonical_origin,
        readiness_timeout: config.acquire_timeout,
        workspace_id,
    };

    Router::new()
        .route("/health/live", get(live))
        .route("/health/ready", get(ready))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            enforce_origin,
        ))
        .with_state(state)
}

async fn live() -> impl IntoResponse {
    (StatusCode::OK, Json(HealthResponse { status: "alive" }))
}

async fn ready(State(state): State<AppState>) -> impl IntoResponse {
    let database_check = timeout(
        state.readiness_timeout,
        sqlx::query("SELECT 1").execute(&state.pool),
    )
    .await;

    match database_check {
        Ok(Ok(_)) => (StatusCode::OK, Json(HealthResponse { status: "ready" })),
        _ => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(HealthResponse {
                status: "not_ready",
            }),
        ),
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use axum::{extract::State, response::IntoResponse};
    use sqlx::postgres::PgPoolOptions;
    use uuid::Uuid;

    use super::{AppState, live, ready};

    #[tokio::test]
    async fn liveness_is_independent_and_readiness_fails_when_database_is_unavailable() {
        let liveness = live().await.into_response();
        assert_eq!(liveness.status(), axum::http::StatusCode::OK);

        let pool = PgPoolOptions::new()
            .acquire_timeout(Duration::from_millis(100))
            .connect_lazy("postgres://lukas:lukas@127.0.0.1:1/lukas")
            .unwrap();
        let state = AppState {
            pool,
            canonical_origin: "http://localhost:4173".to_owned(),
            readiness_timeout: Duration::from_millis(100),
            workspace_id: Uuid::nil(),
        };

        let readiness = ready(State(state)).await.into_response();
        assert_eq!(
            readiness.status(),
            axum::http::StatusCode::SERVICE_UNAVAILABLE
        );
    }
}
