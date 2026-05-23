// SPDX-FileCopyrightText: 2026 The LineageOS Project
//
// SPDX-License-Identifier: Apache-2.0

use axum::Router;
use sqlx::SqlitePool;
use sqlx::sqlite::SqliteConnectOptions;
use std::env;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use tokio::signal;
use tower_http::services::{ServeDir, ServeFile};

pub mod router;
use crate::router::api::api_router;
use crate::router::internal::internal_router;
pub mod tasks;

#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::SqlitePool,
    pub banned: tasks::BannedCache,
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let database_url = env::var("DATABASE_URL").unwrap_or("dev.db".to_string());
    let options = SqliteConnectOptions::new()
        .filename(database_url)
        .create_if_missing(true);
    let pool = SqlitePool::connect_with(options).await?;
    sqlx::migrate!().run(&pool).await?;

    let banned_cache: tasks::BannedCache = Arc::new(RwLock::new(tasks::Banned::default()));

    let state = AppState {
        pool,
        banned: banned_cache,
    };

    // Start tasks
    tasks::spawn_stats_cleanup(state.pool.clone());
    tasks::spawn_banned_refresh(state.pool.clone(), state.banned.clone());

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
