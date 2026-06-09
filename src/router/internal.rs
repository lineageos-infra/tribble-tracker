// SPDX-FileCopyrightText: 2026 The LineageOS Project
//
// SPDX-License-Identifier: Apache-2.0

use crate::AppState;
use crate::database::BannedItem;
use crate::database::TotalInstallationsItem;
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
        .route("/ban/models", delete(unban_model))
        .route("/ban/models", post(ban_models))
        .route("/ban/versions", delete(unban_version))
        .route("/ban/versions", post(ban_versions))
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
    models: Vec<String>,
    #[serde(default)]
    note: Option<String>,
}

async fn ban_models(
    state: State<AppState>,
    input: Json<BanModelInput>,
) -> Result<&'static str, super::RouterError> {
    if input.models.is_empty() {
        return Err(super::RouterError::BadRequest("models are required"));
    }
    if input.models.iter().any(|x| x.is_empty()) {
        return Err(super::RouterError::BadRequest(
            "models cannot contain empty strings",
        ));
    }
    state
        .db
        .upsert_banned_models(&input.models, input.note.as_deref())
        .await?;
    Ok("OK")
}

async fn unban_model(
    state: State<AppState>,
    input: Json<BanModelInput>,
) -> Result<&'static str, super::RouterError> {
    if input.models.is_empty() {
        return Err(super::RouterError::BadRequest("models are required"));
    }
    if input.models.iter().any(|x| x.is_empty()) {
        return Err(super::RouterError::BadRequest(
            "models cannot contain empty strings",
        ));
    }
    state.db.remove_banned_models(&input.models).await?;
    Ok("OK")
}

#[derive(Deserialize)]
struct BanVersionInput {
    versions: Vec<String>,
    #[serde(default)]
    note: Option<String>,
}

async fn ban_versions(
    state: State<AppState>,
    input: Json<BanVersionInput>,
) -> Result<&'static str, super::RouterError> {
    if input.versions.is_empty() {
        return Err(super::RouterError::BadRequest("versions are required"));
    }
    if input.versions.iter().any(|x| x.is_empty()) {
        return Err(super::RouterError::BadRequest(
            "versions cannot contain empty strings",
        ));
    }
    state
        .db
        .upsert_banned_versions(&input.versions, input.note.as_deref())
        .await?;
    Ok("OK")
}

async fn unban_version(
    state: State<AppState>,
    input: Json<BanVersionInput>,
) -> Result<&'static str, super::RouterError> {
    if input.versions.is_empty() {
        return Err(super::RouterError::BadRequest("versions are required"));
    }
    if input.versions.iter().any(|x| x.is_empty()) {
        return Err(super::RouterError::BadRequest(
            "versions cannot contain empty strings",
        ));
    }
    state.db.remove_banned_versions(&input.versions).await?;
    Ok("OK")
}

async fn installations(
    State(state): State<AppState>,
    Query(query): Query<FilterQuery>,
) -> Result<Json<Vec<TotalInstallationsItem>>, super::RouterError> {
    let filters = query.to_filters();
    let items = state.db.fetch_total_installations(&filters).await?;
    Ok(Json(items))
}
