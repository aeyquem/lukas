//! Database-generation validation and workspace-first write locking.

use axum::http::HeaderMap;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::http_error::ApiError;

/// Parse the generation binding required on every financial write.
pub fn installation_generation(headers: &HeaderMap) -> Result<Uuid, ApiError> {
    headers
        .get("Installation-Generation")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| Uuid::parse_str(value).ok())
        .ok_or_else(|| ApiError::invalid("Installation-Generation", "required_uuid"))
}

/// Lock the workspace row first and reject commands bound to a different
/// database generation before looking up idempotency receipts or writing data.
pub async fn lock_workspace(
    tx: &mut Transaction<'_, Postgres>,
    workspace_id: Uuid,
    expected_generation: Uuid,
) -> Result<(), ApiError> {
    let generation: Option<Uuid> =
        sqlx::query_scalar("SELECT installation_generation FROM workspaces WHERE id=$1 FOR UPDATE")
            .bind(workspace_id)
            .fetch_optional(&mut **tx)
            .await
            .map_err(ApiError::database)?;
    let generation = generation.ok_or_else(ApiError::not_found)?;
    if generation != expected_generation {
        return Err(ApiError::new(
            axum::http::StatusCode::CONFLICT,
            "installation_generation_mismatch",
            "This command belongs to a different database generation and was not applied.",
        ));
    }
    Ok(())
}
