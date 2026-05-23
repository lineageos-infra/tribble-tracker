use crate::AppState;
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use cached::macros::cached;
use indexmap::IndexMap;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::{AssertSqlSafe, SqlSafeStr};
use std::sync::LazyLock;

pub fn api_router() -> Router<AppState> {
    Router::new()
        .route("/stats", get(list_stats).post(create_stat))
        .route("/stats/{column}/{name}", get(filtered_stats))
}

#[derive(Serialize, Clone)]
struct StatsResponse {
    model: IndexMap<String, usize>,
    country: IndexMap<String, usize>,
    version: IndexMap<String, usize>,
    carrier: IndexMap<String, usize>,
    total: usize,
}

#[cached(result = true, ttl = 3600, key = "()", convert = r#"{ () }"#)]
async fn list_stats(state: State<AppState>) -> Result<Json<StatsResponse>, super::RouterError> {
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

#[cached(
    result = true,
    ttl = 3600,
    key = "(String, String)",
    convert = r#"{ (path.0.0.clone(), path.0.1.clone()) }"#
)]
async fn filtered_stats(
    state: State<AppState>,
    path: Path<(String, String)>,
) -> Result<Json<StatsResponse>, super::RouterError> {
    let Path((column, name)) = &path;
    let filter_col: &'static str = match column.as_str() {
        "model" => "model",
        "country" => "country",
        "version" => "version",
        "carrier" => "carrier",
        _ => return Err(super::RouterError::BadRequest("invalid filter column")),
    };

    let group_by = |group_col: &'static str| {
        format!(
            "SELECT {group_col}, COUNT(*) FROM stats \
             WHERE {group_col} IS NOT NULL AND {group_col} != '' AND {filter_col} = ? \
             GROUP BY {group_col} ORDER BY 2 DESC LIMIT 250"
        )
    };
    let total_sql = format!("SELECT COUNT(*) FROM stats WHERE {filter_col} = ?");

    let group_fut = |sql: String| {
        sqlx::query_as::<_, (String, i64)>(AssertSqlSafe(sql).into_sql_str())
            .bind(&name)
            .fetch_all(&state.pool)
    };

    let (models, countries, versions, carriers, total) = tokio::try_join!(
        group_fut(group_by("model")),
        group_fut(group_by("country")),
        group_fut(group_by("version")),
        group_fut(group_by("carrier")),
        sqlx::query_scalar::<_, i64>(AssertSqlSafe(total_sql).into_sql_str())
            .bind(&name)
            .fetch_one(&state.pool),
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
    let banned_version: Option<i64> = sqlx::query_scalar!(
        "SELECT 1 FROM banned WHERE version = ? LIMIT 1",
        input.version
    )
    .fetch_optional(&state.pool)
    .await?;
    if banned_version.is_some() {
        return Ok("neat");
    }

    let banned_model: Option<i64> =
        sqlx::query_scalar!("SELECT 1 FROM banned WHERE model = ? LIMIT 1", input.name)
            .fetch_optional(&state.pool)
            .await?;
    if banned_model.is_some() {
        return Ok("neat");
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
        "INSERT INTO stats (device_id, carrier, carrier_id, country, model, official, version, version_raw)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
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
