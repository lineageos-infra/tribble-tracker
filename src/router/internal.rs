// SPDX-FileCopyrightText: 2026 The LineageOS Project
//
// SPDX-License-Identifier: Apache-2.0

use crate::AppState;
use crate::database::{BannedItem, GroupCol, TotalInstallationsItem};
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
        .route("/ban/reap", post(reap_bans))
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

async fn reap_bans(State(state): State<AppState>) -> Result<String, super::RouterError> {
    let rows_affected = state.db.reap_bans().await?;
    Ok(rows_affected.to_string())
}

fn deserialize_non_empty_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v = Vec::<String>::deserialize(deserializer)?;

    if v.is_empty() {
        return Err(serde::de::Error::custom("vector cannot be empty"));
    }

    if v.iter().any(std::string::String::is_empty) {
        return Err(serde::de::Error::custom("vector contains empty string"));
    }

    Ok(v)
}

#[derive(Deserialize)]
struct BanModelInput {
    #[serde(deserialize_with = "deserialize_non_empty_vec")]
    models: Vec<String>,
    #[serde(default)]
    note: Option<String>,
}

async fn ban_models(
    state: State<AppState>,
    input: Json<BanModelInput>,
) -> Result<&'static str, super::RouterError> {
    state
        .db
        .upsert_bans(GroupCol::Model, &input.models, input.note.as_deref())
        .await?;
    Ok("OK")
}

async fn unban_model(
    state: State<AppState>,
    input: Json<BanModelInput>,
) -> Result<&'static str, super::RouterError> {
    state.db.remove_bans(GroupCol::Model, &input.models).await?;
    Ok("OK")
}

#[derive(Deserialize)]
struct BanVersionInput {
    #[serde(deserialize_with = "deserialize_non_empty_vec")]
    versions: Vec<String>,
    #[serde(default)]
    note: Option<String>,
}

async fn ban_versions(
    state: State<AppState>,
    input: Json<BanVersionInput>,
) -> Result<&'static str, super::RouterError> {
    state
        .db
        .upsert_bans(GroupCol::Version, &input.versions, input.note.as_deref())
        .await?;
    Ok("OK")
}

async fn unban_version(
    state: State<AppState>,
    input: Json<BanVersionInput>,
) -> Result<&'static str, super::RouterError> {
    state
        .db
        .remove_bans(GroupCol::Version, &input.versions)
        .await?;
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
