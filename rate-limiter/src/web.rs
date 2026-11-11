use std::sync::Arc;

use axum::{Json, Router, http::StatusCode, middleware::from_fn_with_state, routing::get};
use serde::Serialize;

use crate::{
    algos::token_bucket::TokenBucket, limiter::RateLimiter, middleware::middleware_limiter,
};

pub fn router() -> Router {
    let limiter: Arc<dyn RateLimiter> = Arc::new(TokenBucket::new(10.0, 5.0));
    Router::new()
        .route("/ping", get(ping))
        .layer(from_fn_with_state(limiter, middleware_limiter))
}

#[derive(Serialize)]
struct PingResponse {
    response: &'static str,
}

async fn ping() -> Result<Json<PingResponse>, StatusCode> {
    Ok(Json(PingResponse {
        response: "Hello world",
    }))
}
