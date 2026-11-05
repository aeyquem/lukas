use axum::{
    body::Body,
    http::{Method, Request, StatusCode, header::ORIGIN},
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::health::AppState;

pub fn request_origin_is_allowed(
    method: &Method,
    origin: Option<&str>,
    canonical_origin: &str,
) -> bool {
    if !is_state_changing(method) {
        return true;
    }

    origin == Some(canonical_origin)
}

fn is_state_changing(method: &Method) -> bool {
    !matches!(*method, Method::GET | Method::HEAD | Method::OPTIONS)
}

pub async fn enforce_origin(
    axum::extract::State(state): axum::extract::State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let origin = request
        .headers()
        .get(ORIGIN)
        .and_then(|value| value.to_str().ok());

    if !request_origin_is_allowed(request.method(), origin, &state.canonical_origin) {
        return StatusCode::FORBIDDEN.into_response();
    }

    next.run(request).await
}

#[cfg(test)]
mod tests {
    use super::*;

    const CANONICAL_ORIGIN: &str = "http://localhost:4173";

    #[test]
    fn state_changing_request_accepts_only_exact_origin() {
        assert!(request_origin_is_allowed(
            &Method::POST,
            Some(CANONICAL_ORIGIN),
            CANONICAL_ORIGIN
        ));
        assert!(!request_origin_is_allowed(
            &Method::POST,
            None,
            CANONICAL_ORIGIN
        ));
        assert!(!request_origin_is_allowed(
            &Method::POST,
            Some("http://evil.example"),
            CANONICAL_ORIGIN
        ));
    }

    #[test]
    fn forwarded_origin_does_not_replace_origin_header() {
        // The middleware reads only the Origin header. A forwarded header by
        // itself therefore cannot satisfy a state-changing request.
        assert!(!request_origin_is_allowed(
            &Method::PUT,
            None,
            CANONICAL_ORIGIN
        ));
    }

    #[test]
    fn health_get_is_not_blocked_for_missing_origin() {
        assert!(request_origin_is_allowed(
            &Method::GET,
            None,
            CANONICAL_ORIGIN
        ));
        assert!(request_origin_is_allowed(
            &Method::HEAD,
            None,
            CANONICAL_ORIGIN
        ));
        assert!(request_origin_is_allowed(
            &Method::OPTIONS,
            None,
            CANONICAL_ORIGIN
        ));
    }
}
