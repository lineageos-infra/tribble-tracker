// SPDX-FileCopyrightText: 2026 The LineageOS Project
//
// SPDX-License-Identifier: Apache-2.0

use crate::AppState;
use crate::database::{DbError, FilterClause, GroupCol, GroupedCount, NewStat};
use axum::{
    Json, Router,
    extract::{Query, State, rejection::JsonRejection},
    routing::get,
};
use axum_extra::{TypedHeader, headers::UserAgent};
use cached::macros::cached;
use indexmap::IndexMap;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;

pub fn api_router() -> Router<AppState> {
    Router::new()
        .route("/stats", get(filtered_stats).post(create_stat))
        .route("/stats/filter", get(filtered_stats))
}

#[derive(Deserialize, Clone, Hash, PartialEq, Eq)]
pub struct FilterQuery {
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    country: Option<String>,
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    carrier: Option<String>,
}

impl FilterQuery {
    pub fn iter(&self) -> impl Iterator<Item = (GroupCol, &str)> {
        [
            self.model.as_deref().map(|v| (GroupCol::Model, v)),
            self.country.as_deref().map(|v| (GroupCol::Country, v)),
            self.version.as_deref().map(|v| (GroupCol::Version, v)),
            self.carrier.as_deref().map(|v| (GroupCol::Carrier, v)),
        ]
        .into_iter()
        .flatten()
    }

    #[must_use]
    pub fn to_filters(&self) -> Vec<FilterClause<'_>> {
        self.iter()
            .map(|(column, value)| FilterClause { column, value })
            .collect()
    }

    #[must_use]
    pub fn to_map(&self) -> HashMap<GroupCol, &str> {
        self.iter().collect()
    }
}

#[derive(Serialize, Clone)]
struct StatsResponse {
    model: IndexMap<String, i64>,
    country: IndexMap<String, i64>,
    version: IndexMap<String, i64>,
    carrier: IndexMap<String, i64>,
    total: i64,
    official: i64,
}

async fn filtered_stats(
    State(state): State<AppState>,
    Query(query): Query<FilterQuery>,
) -> Result<Json<StatsResponse>, super::RouterError> {
    filtered_stats_inner(state, query).await
}

async fn fetch_group(
    state: &AppState,
    group: GroupCol,
    filters: &[FilterClause<'_>],
    pinned: &HashMap<GroupCol, &str>,
) -> Result<Option<Vec<GroupedCount>>, DbError> {
    if pinned.contains_key(&group) {
        Ok(None)
    } else {
        state
            .db
            .fetch_grouped_counts(group, filters)
            .await
            .map(Some)
    }
}

#[cached(
    result = true,
    size = 1000,
    ttl = 3600,
    key = "FilterQuery",
    convert = r#"{ query.clone() }"#
)]
async fn filtered_stats_inner(
    state: AppState,
    query: FilterQuery,
) -> Result<Json<StatsResponse>, super::RouterError> {
    let filters = query.to_filters();
    let pinned = query.to_map();

    let (models, countries, versions, carriers, total, official) = tokio::try_join!(
        fetch_group(&state, GroupCol::Model, &filters, &pinned),
        fetch_group(&state, GroupCol::Country, &filters, &pinned),
        fetch_group(&state, GroupCol::Version, &filters, &pinned),
        fetch_group(&state, GroupCol::Carrier, &filters, &pinned),
        state.db.fetch_total(&filters),
        state.db.fetch_official_total(&filters),
    )?;

    let resolve = |rows: Option<Vec<GroupedCount>>, col: GroupCol| -> IndexMap<String, i64> {
        match rows {
            Some(rows) => rows.into_iter().map(|row| (row.name, row.count)).collect(),
            None => IndexMap::from([(pinned[&col].to_string(), total)]),
        }
    };

    Ok(Json(StatsResponse {
        model: resolve(models, GroupCol::Model),
        country: resolve(countries, GroupCol::Country),
        version: resolve(versions, GroupCol::Version),
        carrier: resolve(carriers, GroupCol::Carrier),
        total,
        official,
    }))
}

#[derive(Deserialize)]
struct StatInput {
    #[serde(rename = "device_hash")]
    device_id: String,
    #[serde(rename = "device_name")]
    name: String,
    #[serde(rename = "device_version")]
    version: String,
    #[serde(rename = "device_country")]
    country: String,
    #[serde(rename = "device_carrier")]
    carrier: String,
    #[serde(rename = "device_carrier_id")]
    carrier_id: String,
}

static VERSION_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\d\d\.\d)-").unwrap());
static OFFICIAL_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\d\d\.\d-\d{8}-NIGHTLY-.*").unwrap());
static DEVICE_ID_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[0-9A-F]{64}$").unwrap());

async fn create_stat(
    state: State<AppState>,
    user_agent: Option<TypedHeader<UserAgent>>,
    input: Result<Json<StatInput>, JsonRejection>,
) -> Result<&'static str, super::RouterError> {
    let Ok(Json(mut input)) = input else {
        return Ok("neat");
    };

    let is_dalvik = user_agent
        .as_ref()
        .is_some_and(|x| x.as_str().starts_with("Dalvik/"));

    if !is_dalvik {
        return Ok("neat");
    }

    {
        let banned = state.banned.read().await;
        if banned.versions.contains(&input.version) || banned.models.contains(&input.name) {
            return Ok("neat");
        }
    }

    if !DEVICE_ID_REGEX.is_match(&input.device_id) {
        return Ok("neat");
    }

    if input.name != "x86_64" && !input.version.ends_with(&input.name) {
        return Ok("neat");
    }

    if input.country.len() != 2 && input.country != "Unknown" {
        return Ok("neat");
    }

    let version = match VERSION_REGEX.captures(&input.version) {
        Some(x) => &x[1].to_string(),
        None => return Ok("neat"),
    };
    let official = OFFICIAL_REGEX.is_match(&input.version);

    if input.country != "Unknown" {
        input.country.make_ascii_uppercase();
    }

    state
        .db
        .upsert_stat(NewStat {
            device_id: &input.device_id,
            carrier: &input.carrier,
            carrier_id: &input.carrier_id,
            country: &input.country,
            model: &input.name,
            official,
            version,
            version_raw: &input.version,
        })
        .await?;

    Ok("neat")
}
