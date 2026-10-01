//! Bearer token authentication for the MCP HTTP endpoint.

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse as _, Response},
};

#[derive(Clone)]
pub(super) struct AuthState {
    pub token: std::string::String,
}

fn extract_bearer(request: &Request<Body>) -> Option<&str> {
    request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
}

/// Constant-time comparison so a timing side channel cannot leak the token byte by byte.
fn tokens_match(provided: &str, expected: &str) -> bool {
    let (provided, expected) = (provided.as_bytes(), expected.as_bytes());
    if provided.len() != expected.len() {
        return false;
    }
    provided.iter().zip(expected).fold(0u8, |acc, (a, b)| acc | (a ^ b)) == 0
}

pub(super) async fn require_bearer_token(
    State(state): State<AuthState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    match extract_bearer(&request) {
        Some(token) if tokens_match(token, &state.token) => next.run(request).await,
        _ => StatusCode::UNAUTHORIZED.into_response(),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::{AuthState, require_bearer_token, tokens_match};
    use axum::{Router, body::Body, http::StatusCode, routing::get};
    use tower::ServiceExt as _;

    #[test]
    fn tokens_match_requires_exact_equality() {
        assert!(tokens_match("secret", "secret"));
        assert!(!tokens_match("secret", "secre0"));
        assert!(!tokens_match("secret", "secrets"));
        assert!(!tokens_match("", "secret"));
    }

    fn guarded_router() -> Router {
        let state = AuthState {
            token: "expected".to_owned(),
        };
        Router::new()
            .route("/mcp", get(|| async { "ok" }))
            .route_layer(axum::middleware::from_fn_with_state(state, require_bearer_token))
    }

    #[tokio::test]
    async fn rejects_request_with_no_authorization_header() {
        let response = guarded_router()
            .oneshot(axum::http::Request::builder().uri("/mcp").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn rejects_request_with_wrong_token() {
        let request = axum::http::Request::builder()
            .uri("/mcp")
            .header("authorization", "Bearer wrong")
            .body(Body::empty())
            .unwrap();
        let response = guarded_router().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn accepts_request_with_correct_token() {
        let request = axum::http::Request::builder()
            .uri("/mcp")
            .header("authorization", "Bearer expected")
            .body(Body::empty())
            .unwrap();
        let response = guarded_router().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
