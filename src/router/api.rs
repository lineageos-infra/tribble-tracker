use crate::AppState;
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;

pub fn api_router() -> Router<AppState> {
    Router::new()
        .route("/stats", get(list_stats).post(create_stat))
        .route("/stats/{thing}/{name}", get(filtered_stats))
}

#[derive(Serialize)]
struct StatsResponse {
    model: HashMap<String, usize>,
    country: HashMap<String, usize>,
    version: HashMap<String, usize>,
    total: usize,
}

async fn list_stats(
    State(state): State<AppState>,
) -> Result<Json<StatsResponse>, super::RouterError> {
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

    let total_fut = sqlx::query_scalar!(r#"SELECT COUNT(*) FROM stats"#).fetch_one(&state.pool);

    let (models, countries, versions, total) =
        tokio::try_join!(models_fut, countries_fut, versions_fut, total_fut)?;

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
        total: total as usize,
    }))
}

async fn filtered_stats(
    State(state): State<AppState>,
    Path((thing, name)): Path<(String, String)>,
) -> Result<Json<StatsResponse>, super::RouterError> {
    macro_rules! queries_by {
        ($col:literal) => {
            [
                concat!(
                    "SELECT model, COUNT(*) FROM stats ",
                    "WHERE model IS NOT NULL AND ",
                    $col,
                    " = ? ",
                    "GROUP BY model ORDER BY 2 DESC LIMIT 250",
                ),
                concat!(
                    "SELECT country, COUNT(*) FROM stats ",
                    "WHERE country IS NOT NULL AND ",
                    $col,
                    " = ? ",
                    "GROUP BY country ORDER BY 2 DESC LIMIT 250",
                ),
                concat!(
                    "SELECT version, COUNT(*) FROM stats ",
                    "WHERE version IS NOT NULL AND ",
                    $col,
                    " = ? ",
                    "GROUP BY version ORDER BY 2 DESC LIMIT 250",
                ),
                concat!("SELECT COUNT(*) FROM stats WHERE ", $col, " = ?"),
            ]
        };
    }

    let [model_sql, country_sql, version_sql, total_sql]: [&'static str; 4] = match thing.as_str() {
        "model" => queries_by!("model"),
        "country" => queries_by!("country"),
        "version" => queries_by!("version"),
        "carrier" => queries_by!("carrier"),
        _ => return Err(super::RouterError::BadRequest("invalid filter column")),
    };

    let models_fut = sqlx::query_as::<_, (String, i64)>(model_sql)
        .bind(&name)
        .fetch_all(&state.pool);
    let countries_fut = sqlx::query_as::<_, (String, i64)>(country_sql)
        .bind(&name)
        .fetch_all(&state.pool);
    let versions_fut = sqlx::query_as::<_, (String, i64)>(version_sql)
        .bind(&name)
        .fetch_all(&state.pool);
    let total_fut = sqlx::query_scalar::<_, i64>(total_sql)
        .bind(&name)
        .fetch_one(&state.pool);

    let (models, countries, versions, total) =
        tokio::try_join!(models_fut, countries_fut, versions_fut, total_fut)?;

    Ok(Json(StatsResponse {
        model: models.into_iter().map(|(k, c)| (k, c as usize)).collect(),
        country: countries
            .into_iter()
            .map(|(k, c)| (k, c as usize))
            .collect(),
        version: versions.into_iter().map(|(k, c)| (k, c as usize)).collect(),
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
    State(state): State<AppState>,
    Json(mut input): Json<StatInput>,
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
