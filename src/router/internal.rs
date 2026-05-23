use crate::AppState;
use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

pub fn internal_router() -> Router<AppState> {
    Router::new()
        .route("/ban/list", get(list_bans))
        .route("/ban/model", post(ban_model))
        .route("/ban/version", post(ban_version))
}

#[derive(Serialize)]
struct BannedItem {
    version: Option<String>,
    model: Option<String>,
    note: Option<String>,
}

async fn list_bans(
    State(state): State<AppState>,
) -> Result<Json<Vec<BannedItem>>, super::RouterError> {
    let items = sqlx::query_as!(BannedItem, r#"SELECT version, model, note FROM banned"#)
        .fetch_all(&state.pool)
        .await?;

    Ok(Json(items))
}

#[derive(Deserialize)]
struct BanModelInput {
    model: String,
    #[serde(default)]
    note: Option<String>,
}

async fn ban_model(
    State(state): State<AppState>,
    Json(input): Json<BanModelInput>,
) -> Result<&'static str, super::RouterError> {
    if input.model.is_empty() {
        return Err(super::RouterError::BadRequest("model is required"));
    }
    sqlx::query!(
        "INSERT INTO banned (model, note) VALUES (?, ?)
         ON CONFLICT (model) DO UPDATE SET note = excluded.note",
        input.model,
        input.note,
    )
    .execute(&state.pool)
    .await?;
    Ok("OK")
}

#[derive(Deserialize)]
struct BanVersionInput {
    version: String,
    #[serde(default)]
    note: Option<String>,
}

async fn ban_version(
    State(state): State<AppState>,
    Json(input): Json<BanVersionInput>,
) -> Result<&'static str, super::RouterError> {
    if input.version.is_empty() {
        return Err(super::RouterError::BadRequest("version is required"));
    }
    sqlx::query!(
        "INSERT INTO banned (version, note) VALUES (?, ?)
         ON CONFLICT (version) DO UPDATE SET note = excluded.note",
        input.version,
        input.note,
    )
    .execute(&state.pool)
    .await?;
    Ok("OK")
}
