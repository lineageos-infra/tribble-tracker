// SPDX-FileCopyrightText: 2026 The LineageOS Project
//
// SPDX-License-Identifier: Apache-2.0

use crate::AppState;
use crate::database::{FilterClause, GroupCol, NewStat};
use axum::{
    Json, Router,
    extract::{Query, State},
    routing::get,
};
use axum_extra::{TypedHeader, headers::UserAgent};
use cached::macros::cached;
use indexmap::IndexMap;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

pub fn api_router() -> Router<AppState> {
    Router::new()
        .route("/stats", get(filtered_stats).post(create_stat))
        .route("/stats/filter", get(filtered_stats))
}

#[derive(Deserialize, Clone, Hash, PartialEq, Eq)]
struct FilterQuery {
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
    fn to_filters(&self) -> Vec<FilterClause<'_>> {
        let mut filters = Vec::new();

        if let Some(name) = &self.model {
            filters.push(FilterClause {
                column: "model",
                value: name,
            });
        }
        if let Some(name) = &self.country {
            filters.push(FilterClause {
                column: "country",
                value: name,
            });
        }
        if let Some(name) = &self.version {
            filters.push(FilterClause {
                column: "version",
                value: name,
            });
        }
        if let Some(name) = &self.carrier {
            filters.push(FilterClause {
                column: "carrier",
                value: name,
            });
        }

        filters
    }
}

#[derive(Serialize, Clone)]
struct StatsResponse {
    model: IndexMap<String, usize>,
    country: IndexMap<String, usize>,
    version: IndexMap<String, usize>,
    carrier: IndexMap<String, usize>,
    total: usize,
}

async fn filtered_stats(
    State(state): State<AppState>,
    Query(query): Query<FilterQuery>,
) -> Result<Json<StatsResponse>, super::RouterError> {
    filtered_stats_inner(state, query).await
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

    let (models, countries, versions, carriers, total) = tokio::try_join!(
        state.db.fetch_grouped_counts(GroupCol::Model, &filters),
        state.db.fetch_grouped_counts(GroupCol::Country, &filters),
        state.db.fetch_grouped_counts(GroupCol::Version, &filters),
        state.db.fetch_grouped_counts(GroupCol::Carrier, &filters),
        state.db.fetch_total(&filters),
    )?;

    Ok(Json(StatsResponse {
        model: models.into_iter().map(|(k, c)| (k, c as usize)).collect(),
        country: countries
            .into_iter()
            .map(|(k, c)| (k, c as usize))
            .collect(),
        version: versions.into_iter().map(|(k, c)| (k, c as usize)).collect(),
        carrier: carriers.into_iter().map(|(k, c)| (k, c as usize)).collect(),
        total: total as usize,
    }))
}

#[derive(Deserialize)]
struct StatInput {
    device_id: String,
    name: String,
    version: String,
    country: String,
    carrier: Option<String>,
    carrier_id: Option<String>,
}

static VERSION_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\d+\.\d+").unwrap());
static OFFICIAL_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\d\d\.\d-\d{8}-NIGHTLY-.*").unwrap());
static DEVICE_ID_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[0-9a-f]{64}$").unwrap());

async fn create_stat(
    state: State<AppState>,
    user_agent: Option<TypedHeader<UserAgent>>,
    mut input: Json<StatInput>,
) -> Result<&'static str, super::RouterError> {
    let is_dalvik = user_agent
        .as_ref()
        .map(|x| x.as_str().starts_with("Dalvik/"))
        .unwrap_or(false);

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
        return Err(super::RouterError::BadRequest("device_id is not valid"));
    }

    if input.name != "x86_64" && !input.version.ends_with(&input.name) {
        return Err(super::RouterError::BadRequest(
            "version string must end with -model",
        ));
    }

    if input.country.len() != 2 && input.country != "Unknown" {
        return Err(super::RouterError::BadRequest(
            "country must be a two letter iso code",
        ));
    }

    let version = VERSION_REGEX
        .find(&input.version)
        .ok_or(super::RouterError::BadRequest(
            "version must start with version code (ie, 22.1)",
        ))?
        .as_str()
        .to_string();

    let official = OFFICIAL_REGEX.is_match(&input.version);

    if input.country != "Unknown" {
        input.country = input.country.to_uppercase();
    }

    state
        .db
        .upsert_stat(NewStat {
            device_id: &input.device_id,
            carrier: input.carrier.as_deref(),
            carrier_id: input.carrier_id.as_deref(),
            country: &input.country,
            model: &input.name,
            official,
            version: &version,
            version_raw: &input.version,
        })
        .await?;

    Ok("neat")
}
