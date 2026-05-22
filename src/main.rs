use axum::Router;
use std::env;

use crate::router::api_router;
use crate::router::internal_router;
mod router;

#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::SqlitePool,
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let database_url = env::var("DATABASE_URL").unwrap_or("sqlite:dev.db".to_string());
    let pool = sqlx::SqlitePool::connect(&database_url).await?;

    let state = AppState { pool };

    let app = Router::new()
        .nest("/api/v1", api_router())
        .nest("/internal", internal_router())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
