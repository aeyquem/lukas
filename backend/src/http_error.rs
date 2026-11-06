use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;
use std::{
    future::Future,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize)]
pub struct FieldError {
    pub field: String,
    pub code: String,
}

#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    code: &'static str,
    message: &'static str,
    fields: Vec<FieldError>,
    request_id: Uuid,
    retryable_transaction_abort: bool,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: ErrorDetail,
    request_id: Uuid,
}

#[derive(Serialize)]
struct ErrorDetail {
    code: &'static str,
    message: &'static str,
    fields: Vec<FieldError>,
}

impl ApiError {
    pub fn new(status: StatusCode, code: &'static str, message: &'static str) -> Self {
        Self {
            status,
            code,
            message,
            fields: Vec::new(),
            request_id: Uuid::new_v4(),
            retryable_transaction_abort: false,
        }
    }

    pub fn field(
        status: StatusCode,
        code: &'static str,
        message: &'static str,
        field: impl Into<String>,
        field_code: impl Into<String>,
    ) -> Self {
        let mut error = Self::new(status, code, message);
        error.fields.push(FieldError {
            field: field.into(),
            code: field_code.into(),
        });
        error
    }

    pub fn bad_request(code: &'static str, message: &'static str) -> Self {
        Self::new(StatusCode::BAD_REQUEST, code, message)
    }

    pub fn invalid_request_parameter() -> Self {
        Self::bad_request(
            "invalid_request_parameter",
            "A path or query parameter is malformed.",
        )
    }

    pub fn invalid(field: impl Into<String>, field_code: impl Into<String>) -> Self {
        Self::field(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            "Review the highlighted fields.",
            field,
            field_code,
        )
    }

    pub fn not_found() -> Self {
        Self::new(
            StatusCode::NOT_FOUND,
            "not_found",
            "The requested item was not found.",
        )
    }

    pub fn stale_revision() -> Self {
        Self::new(
            StatusCode::CONFLICT,
            "stale_revision",
            "Review the latest value before saving again.",
        )
    }

    pub fn idempotency_conflict() -> Self {
        Self::new(
            StatusCode::CONFLICT,
            "idempotency_key_reused",
            "This idempotency key is already tied to a different request.",
        )
    }

    pub fn database(error: sqlx::Error) -> Self {
        let transaction_abort = error
            .as_database_error()
            .and_then(|database| database.code())
            .is_some_and(|code| code == "40001" || code == "40P01");
        let temporary = matches!(
            error,
            sqlx::Error::PoolTimedOut
                | sqlx::Error::PoolClosed
                | sqlx::Error::Io(_)
                | sqlx::Error::Tls(_)
                | sqlx::Error::WorkerCrashed
        ) || transaction_abort
            || error
                .as_database_error()
                .and_then(|database| database.code())
                .is_some_and(|code| code.starts_with("08"));

        if temporary {
            let mut api_error = Self::new(
                StatusCode::SERVICE_UNAVAILABLE,
                if transaction_abort {
                    "storage_busy"
                } else {
                    "storage_unavailable"
                },
                if transaction_abort {
                    "Storage could not serialize this write. Retry with the same request key."
                } else {
                    "Storage is temporarily unavailable. Try again with the same request key."
                },
            );
            api_error.retryable_transaction_abort = transaction_abort;
            api_error
        } else {
            tracing::error!(%error, "financial API database operation failed");
            Self::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "The request could not be completed.",
            )
        }
    }
}

/// Run one write command at most three times, retrying only a confirmed
/// PostgreSQL serialization or deadlock abort. Each call must start a new
/// transaction and reuse the same request bytes and idempotency key.
pub async fn retry_transaction<T, F, Fut>(mut operation: F) -> Result<T, ApiError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, ApiError>>,
{
    const MAX_ATTEMPTS: usize = 3;

    for attempt in 0..MAX_ATTEMPTS {
        match operation().await {
            Err(error) if error.retryable_transaction_abort && attempt + 1 < MAX_ATTEMPTS => {
                tokio::time::sleep(retry_delay(attempt)).await;
            }
            result => return result,
        }
    }

    unreachable!("the retry loop always returns on its final attempt")
}

fn retry_delay(attempt: usize) -> Duration {
    let base_millis = 10_u64.saturating_mul(1 << attempt.min(4));
    let jitter_millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos() as u64
        % 21;
    Duration::from_millis(base_millis + jitter_millis)
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let body = ErrorResponse {
            error: ErrorDetail {
                code: self.code,
                message: self.message,
                fields: self.fields,
            },
            request_id: self.request_id,
        };
        (self.status, Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn serialization_abort() -> ApiError {
        let mut error = ApiError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "storage_busy",
            "Storage is busy.",
        );
        error.retryable_transaction_abort = true;
        error
    }

    #[tokio::test]
    async fn confirmed_aborts_retry_twice_then_succeed() {
        let attempts = AtomicUsize::new(0);
        let result = retry_transaction(|| async {
            let count = attempts.fetch_add(1, Ordering::SeqCst) + 1;
            if count < 3 {
                Err(serialization_abort())
            } else {
                Ok(count)
            }
        })
        .await;

        assert_eq!(result.unwrap(), 3);
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn non_retryable_error_is_returned_without_another_attempt() {
        let attempts = AtomicUsize::new(0);
        let result: Result<(), ApiError> = retry_transaction(|| async {
            attempts.fetch_add(1, Ordering::SeqCst);
            Err(ApiError::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "storage_unavailable",
                "Storage is unavailable.",
            ))
        })
        .await;

        assert_eq!(result.unwrap_err().code, "storage_unavailable");
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn connection_failure_maps_to_retryable_client_response_without_server_retry() {
        let error = ApiError::database(sqlx::Error::Io(std::io::Error::new(
            std::io::ErrorKind::ConnectionReset,
            "connection reset",
        )));

        assert_eq!(error.status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(error.code, "storage_unavailable");
        assert!(!error.retryable_transaction_abort);
    }

    #[tokio::test]
    async fn exhausted_confirmed_aborts_stop_after_three_attempts() {
        let attempts = AtomicUsize::new(0);
        let result: Result<(), ApiError> = retry_transaction(|| async {
            attempts.fetch_add(1, Ordering::SeqCst);
            Err(serialization_abort())
        })
        .await;

        assert!(result.unwrap_err().retryable_transaction_abort);
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }
}
