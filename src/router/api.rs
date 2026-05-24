// SPDX-FileCopyrightText: 2026 The LineageOS Project
//
// SPDX-License-Identifier: Apache-2.0

use crate::AppState;
use axum::{
    Json, Router,
    extract::{Query, State},
    routing::get,
};
use cached::macros::cached;
use indexmap::IndexMap;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

pub fn api_router() -> Router<AppState> {
    Router::new()
        .route("/stats", get(list_stats).post(create_stat))
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

#[derive(Clone)]
struct FilterClause {
    column: &'static str,
    name: String,
}

impl FilterQuery {
    fn into_filters(self) -> Vec<FilterClause> {
        let mut filters = Vec::new();

        if let Some(name) = self.model {
            filters.push(FilterClause {
                column: "model",
                name,
            });
        }
        if let Some(name) = self.country {
            filters.push(FilterClause {
                column: "country",
                name,
            });
        }
        if let Some(name) = self.version {
            filters.push(FilterClause {
                column: "version",
                name,
            });
        }
        if let Some(name) = self.carrier {
            filters.push(FilterClause {
                column: "carrier",
                name,
            });
        }

        filters
    }
}

#[derive(Clone, Copy)]
enum GroupCol {
    Model,
    Country,
    Version,
    Carrier,
}

impl GroupCol {
    fn as_str(self) -> &'static str {
        match self {
            Self::Model => "model",
            Self::Country => "country",
            Self::Version => "version",
            Self::Carrier => "carrier",
        }
    }
}

async fn fetch_filtered_counts(
    state: &AppState,
    group_col: GroupCol,
    filters: &[FilterClause],
) -> Result<Vec<(String, i64)>, sqlx::Error> {
    let col = group_col.as_str();
    let mut qb = sqlx::QueryBuilder::new(format!(
        "SELECT {col}, COUNT(*) FROM stats WHERE {col} IS NOT NULL AND {col} != ''"
    ));
    for filter in filters {
        qb.push(" AND ")
            .push(filter.column)
            .push(" = ")
            .push_bind(&filter.name);
    }
    qb.push(format!(" GROUP BY {col} ORDER BY 2 DESC LIMIT 250"));
    qb.build_query_as::<(String, i64)>()
        .fetch_all(&state.pool)
        .await
}

async fn fetch_filtered_total(
    state: &AppState,
    filters: &[FilterClause],
) -> Result<i64, sqlx::Error> {
    let mut qb = sqlx::QueryBuilder::new("SELECT COUNT(*) FROM stats");

    if !filters.is_empty() {
        qb.push(" WHERE ");
        let mut separated = qb.separated(" AND ");
        for filter in filters {
            separated
                .push(filter.column)
                .push_unseparated(" = ")
                .push_bind_unseparated(&filter.name);
        }
    }

    qb.build_query_scalar::<i64>().fetch_one(&state.pool).await
}

#[derive(Serialize, Clone)]
struct StatsResponse {
    model: IndexMap<String, usize>,
    country: IndexMap<String, usize>,
    version: IndexMap<String, usize>,
    carrier: IndexMap<String, usize>,
    total: usize,
}

async fn list_stats(
    State(state): State<AppState>,
) -> Result<Json<StatsResponse>, super::RouterError> {
    list_stats_inner(state).await
}

#[cached(result = true, ttl = 3600, key = "()", convert = r#"{ () }"#)]
async fn list_stats_inner(state: AppState) -> Result<Json<StatsResponse>, super::RouterError> {
    let models_fut = sqlx::query!(
        r#"SELECT model AS "model!: String", COUNT(*) AS count
           FROM stats WHERE model IS NOT NULL
           GROUP BY model ORDER BY count DESC LIMIT 250"#
    )
    .fetch_all(&state.pool);

    let countries_fut = sqlx::query!(
        r#"SELECT country AS "country!: String", COUNT(*) AS count
           FROM stats WHERE country IS NOT NULL
           GROUP BY country ORDER BY count DESC LIMIT 250"#
    )
    .fetch_all(&state.pool);

    let versions_fut = sqlx::query!(
        r#"SELECT version AS "version!: String", COUNT(*) AS count
           FROM stats WHERE version IS NOT NULL
           GROUP BY version ORDER BY count DESC LIMIT 250"#
    )
    .fetch_all(&state.pool);

    let carriers_fut = sqlx::query!(
        r#"SELECT carrier AS "carrier!: String", COUNT(*) AS count
           FROM stats WHERE carrier IS NOT NULL AND carrier != ''
           GROUP BY carrier ORDER BY count DESC LIMIT 250"#
    )
    .fetch_all(&state.pool);

    let total_fut = sqlx::query_scalar!(r#"SELECT COUNT(*) FROM stats"#).fetch_one(&state.pool);

    let (models, countries, versions, carriers, total) = tokio::try_join!(
        models_fut,
        countries_fut,
        versions_fut,
        carriers_fut,
        total_fut
    )?;

    Ok(Json(StatsResponse {
        model: models
            .into_iter()
            .map(|r| (r.model, r.count as usize))
            .collect(),
        country: countries
            .into_iter()
            .map(|r| (r.country, r.count as usize))
            .collect(),
        version: versions
            .into_iter()
            .map(|r| (r.version, r.count as usize))
            .collect(),
        carrier: carriers
            .into_iter()
            .map(|r| (r.carrier, r.count as usize))
            .collect(),
        total: total as usize,
    }))
}

async fn filtered_stats(
    State(state): State<AppState>,
    Query(query): Query<FilterQuery>,
) -> Result<Json<StatsResponse>, super::RouterError> {
    filtered_stats_inner(state, query).await
}

#[cached(
    result = true,
    ttl = 3600,
    key = "FilterQuery",
    convert = r#"{ query.clone() }"#
)]
async fn filtered_stats_inner(
    state: AppState,
    query: FilterQuery,
) -> Result<Json<StatsResponse>, super::RouterError> {
    let filters = query.into_filters();

    if filters.is_empty() {
        return list_stats(State(state)).await;
    }

    let (models, countries, versions, carriers, total) = tokio::try_join!(
        fetch_filtered_counts(&state, GroupCol::Model, &filters),
        fetch_filtered_counts(&state, GroupCol::Country, &filters),
        fetch_filtered_counts(&state, GroupCol::Version, &filters),
        fetch_filtered_counts(&state, GroupCol::Carrier, &filters),
        fetch_filtered_total(&state, &filters),
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
    LazyLock::new(|| Regex::new(r"-(?:UNOFFICIAL|unofficial)").unwrap());

async fn create_stat(
    state: State<AppState>,
    mut input: Json<StatInput>,
) -> Result<&'static str, super::RouterError> {
    {
        let banned = state.banned.read().unwrap();
        if banned.versions.contains(&input.version) || banned.models.contains(&input.name) {
            return Ok("neat");
        }
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

    let official = !OFFICIAL_REGEX.is_match(&input.version);

    if input.country != "Unknown" {
        input.country = input.country.to_uppercase();
    }

    sqlx::query!(
        r#"
        INSERT INTO stats (device_id, carrier, carrier_id, country, model, official, version, version_raw)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(device_id) DO UPDATE SET
            carrier = excluded.carrier,
            carrier_id = excluded.carrier_id,
            country = excluded.country,
            model = excluded.model,
            official = excluded.official,
            version = excluded.version,
            version_raw = excluded.version_raw
        "#,
        input.device_id,
        input.carrier,
        input.carrier_id,
        input.country,
        input.name,
        official,
        version,
        input.version,
    )
    .execute(&state.pool)
    .await?;

    Ok("neat")
}
