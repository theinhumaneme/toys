use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::limiter::RateLimiter;

pub async fn middleware_limiter(
    State(limiter): State<Arc<dyn RateLimiter>>,
    req: Request,
    next: Next,
) -> Response {
    let client_id = req
        .headers()
        .get("X-Client-Id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("anonymous");
    if limiter.allow(client_id) {
        next.run(req).await
    } else {
        StatusCode::TOO_MANY_REQUESTS.into_response()
    }
}
