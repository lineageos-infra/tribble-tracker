// SPDX-FileCopyrightText: 2026 The LineageOS Project
//
// SPDX-License-Identifier: Apache-2.0

use crate::AppState;
use crate::database::BannedItem;
use crate::database::VersionRawTotalItem;
use crate::router::api::FilterQuery;
use axum::{
    Json, Router,
    extract::{Query, State},
    routing::{delete, get, post},
};
use serde::Deserialize;

pub fn internal_router() -> Router<AppState> {
    Router::new()
        .route("/ban/list", get(list_bans))
        .route("/ban/model", delete(unban_model))
        .route("/ban/model", post(ban_model))
        .route("/ban/version", delete(unban_version))
        .route("/ban/version", post(ban_version))
        .route("/installations", get(installations))
}

async fn list_bans(
    State(state): State<AppState>,
) -> Result<Json<Vec<BannedItem>>, super::RouterError> {
    let items = state.db.list_bans().await?;
    Ok(Json(items))
}

#[derive(Deserialize)]
struct BanModelInput {
    model: String,
    #[serde(default)]
    note: Option<String>,
}

async fn ban_model(
    state: State<AppState>,
    input: Json<BanModelInput>,
) -> Result<&'static str, super::RouterError> {
    if input.model.is_empty() {
        return Err(super::RouterError::BadRequest("model is required"));
    }
    state
        .db
        .upsert_banned_model(&input.model, input.note.as_deref())
        .await?;
    Ok("OK")
}

async fn unban_model(
    state: State<AppState>,
    input: Json<BanModelInput>,
) -> Result<&'static str, super::RouterError> {
    if input.model.is_empty() {
        return Err(super::RouterError::BadRequest("model is required"));
    }
    state.db.remove_banned_model(&input.model).await?;
    Ok("OK")
}

#[derive(Deserialize)]
struct BanVersionInput {
    version: String,
    #[serde(default)]
    note: Option<String>,
}

async fn ban_version(
    state: State<AppState>,
    input: Json<BanVersionInput>,
) -> Result<&'static str, super::RouterError> {
    if input.version.is_empty() {
        return Err(super::RouterError::BadRequest("version is required"));
    }
    state
        .db
        .upsert_banned_version(&input.version, input.note.as_deref())
        .await?;
    Ok("OK")
}

async fn unban_version(
    state: State<AppState>,
    input: Json<BanVersionInput>,
) -> Result<&'static str, super::RouterError> {
    if input.version.is_empty() {
        return Err(super::RouterError::BadRequest("version is required"));
    }
    state.db.remove_banned_version(&input.version).await?;
    Ok("OK")
}

async fn installations(
    State(state): State<AppState>,
    Query(query): Query<FilterQuery>,
) -> Result<Json<Vec<VersionRawTotalItem>>, super::RouterError> {
    let filters = query.to_filters();
    let items = state.db.fetch_version_raw_total(&filters).await?;
    Ok(Json(items))
}
