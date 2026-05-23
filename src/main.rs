use axum::Router;
use std::env;
use std::net::SocketAddr;
use tokio::signal;
use tower_http::services::{ServeDir, ServeFile};

pub mod router;
use crate::router::api::api_router;
use crate::router::internal::internal_router;

#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::SqlitePool,
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let database_url = env::var("DATABASE_URL").unwrap_or("sqlite:dev.db".to_string());
    let pool = sqlx::SqlitePool::connect(&database_url).await?;
    sqlx::migrate!().run(&pool).await?;

    let state = AppState { pool };

    // Production Path, use vite directly in development
    let client = ServeDir::new("client").fallback(ServeFile::new("client/index.html"));

    let app = Router::new()
        .nest("/api/v1", api_router())
        .nest("/internal", internal_router())
        .fallback_service(client)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
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
