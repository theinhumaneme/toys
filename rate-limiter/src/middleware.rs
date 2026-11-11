use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use tokio::time::sleep;

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
    match limiter.check(client_id) {
        Some(wait) => {
            if !wait.is_zero() {
                sleep(wait).await;
            }
            next.run(req).await
        }
        None => StatusCode::TOO_MANY_REQUESTS.into_response(),
    }
}
