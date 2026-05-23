use axum::Router;
use std::env;
use tokio::signal;

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
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
