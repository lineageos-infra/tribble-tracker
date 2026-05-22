use crate::AppState;
use crate::models::{Banned, Stat};
use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;

static VERSION_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\d+\.\d+").unwrap());
static OFFICIAL_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"-(?:UNOFFICIAL|unofficial)").unwrap());

pub fn api_router() -> Router<AppState> {
    Router::new().route("/stats", get(list_stats).post(create_stat))
}

enum ApiError {
    Db(toasty::Error),
    BadRequest(&'static str),
}

impl From<toasty::Error> for ApiError {
    fn from(e: toasty::Error) -> Self {
        ApiError::Db(e)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::Db(e) => {
                eprintln!("database error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "sql error").into_response()
            }
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg).into_response(),
        }
    }
}

#[derive(Serialize)]
struct StatsResponse {
    model: HashMap<String, usize>,
    country: HashMap<String, usize>,
    version: HashMap<String, usize>,
    total: usize,
}

async fn list_stats(State(state): State<AppState>) -> Result<Json<StatsResponse>, ApiError> {
    let mut db = state.db.clone();
    let stats = Stat::all()
        .select((
            Stat::fields().model(),
            Stat::fields().country(),
            Stat::fields().version(),
        ))
        .exec(&mut db)
        .await?;

    let mut model: HashMap<String, usize> = HashMap::new();
    let mut country: HashMap<String, usize> = HashMap::new();
    let mut version: HashMap<String, usize> = HashMap::new();

    for s in &stats {
        if let Some(v) = &s.0 {
            *model.entry(v.clone()).or_insert(0) += 1;
        }
        if let Some(v) = &s.1 {
            *country.entry(v.clone()).or_insert(0) += 1;
        }
        if let Some(v) = &s.2 {
            *version.entry(v.clone()).or_insert(0) += 1;
        }
    }

    Ok(Json(StatsResponse {
        model: top_n(model, 250),
        country: top_n(country, 250),
        version: top_n(version, 250),
        total: stats.len(),
    }))
}

fn top_n(map: HashMap<String, usize>, n: usize) -> HashMap<String, usize> {
    let mut v: Vec<_> = map.into_iter().collect();
    v.sort_by(|a, b| b.1.cmp(&a.1));
    v.into_iter().take(n).collect()
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

async fn create_stat(
    State(state): State<AppState>,
    Json(mut input): Json<StatInput>,
) -> Result<&'static str, ApiError> {
    let mut db = state.db.clone();

    // banned version?
    let banned_version = Banned::filter(Banned::fields().version().eq(input.version.clone()))
        .first()
        .exec(&mut db)
        .await?;
    if banned_version.is_some() {
        return Ok("neat");
    }

    // banned model?
    let banned_model = Banned::filter(Banned::fields().model().eq(input.name.clone()))
        .first()
        .exec(&mut db)
        .await?;
    if banned_model.is_some() {
        return Ok("neat");
    }

    if input.name != "x86_64" && !input.version.ends_with(&input.name) {
        return Err(ApiError::BadRequest("version string must end with -model"));
    }

    if input.country.len() != 2 && input.country != "Unknown" {
        return Err(ApiError::BadRequest(
            "country must be a two letter iso code",
        ));
    }

    let version = VERSION_REGEX
        .find(&input.version)
        .ok_or(ApiError::BadRequest(
            "version must start with version code (ie, 22.1)",
        ))?
        .as_str()
        .to_string();

    let official = !OFFICIAL_REGEX.is_match(&input.version);

    if input.country != "Unknown" {
        input.country = input.country.to_uppercase();
    }

    toasty::create!(Stat {
        device_id: input.device_id,
        carrier: input.carrier,
        carrier_id: input.carrier_id,
        country: Some(input.country),
        model: Some(input.name),
        official: Some(official),
        submit_time: Some(jiff::Zoned::now().datetime()),
        version: Some(version),
        version_raw: Some(input.version),
    })
    .exec(&mut db)
    .await?;

    Ok("neat")
}

pub fn internal_router() -> Router<AppState> {
    Router::new().route("/ban/list", get(list_bans))
}

#[derive(Serialize)]
struct BannedItem {
    version: String,
    model: String,
    note: Option<String>,
}

async fn list_bans(State(state): State<AppState>) -> Result<Json<Vec<BannedItem>>, ApiError> {
    let mut db = state.db.clone();
    let banned = Banned::all().exec(&mut db).await?;

    let items = banned
        .into_iter()
        .map(|b| BannedItem {
            version: b.version,
            model: b.model,
            note: b.note,
        })
        .collect();

    Ok(Json(items))
}
