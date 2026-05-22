use axum::{Router, routing::get};
use std::env;

use crate::router::api_router;
use crate::router::internal_router;
mod models;
mod router;

#[derive(Clone)]
pub struct AppState {
    pub db: toasty::Db,
}

#[tokio::main]
async fn main() -> toasty::Result<()> {
    // Set-up DB connection
    let database_path = env::var("DATABASE_PATH").unwrap_or("dev.db".to_string());
    let db = toasty::Db::builder()
        .models(toasty::models!(crate::*))
        .connect(&format!("sqlite:{}", database_path))
        .await?;

    // Set-up global Axum state
    let state = AppState { db };

    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .nest("/api/v1", api_router())
        .nest("/internal", internal_router())
        .with_state(state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
