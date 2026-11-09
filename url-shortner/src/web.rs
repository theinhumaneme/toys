use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::Redirect,
    routing::{get, post},
};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::db;

#[derive(Deserialize)]
struct ShortenRequest {
    long_url: String,
}

#[derive(Serialize)]
struct ShortenResponse {
    short_url: String,
}

type AppState = Arc<Mutex<Connection>>;

pub fn router(conn: Connection) -> Router {
    let state: AppState = Arc::new(Mutex::new(conn));
    Router::new()
        .route("/shorten", post(shorten))
        .route("/{code}", get(redirect))
        .with_state(state)
}

async fn shorten(
    State(state): State<AppState>,
    Json(req): Json<ShortenRequest>,
) -> Result<Json<ShortenResponse>, StatusCode> {
    let conn = state.lock().await;
    let code = db::shorten(&conn, &req.long_url).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(ShortenResponse {
        short_url: format!("http://127.0.0.1:3000/{code}"),
    }))
}

async fn redirect(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> Result<Redirect, StatusCode> {
    let conn = state.lock().await;
    match db::resolve(&conn, &code) {
        Ok(Some(long_url)) => Ok(Redirect::temporary(&long_url)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
