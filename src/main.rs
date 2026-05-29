// SPDX-FileCopyrightText: 2026 The LineageOS Project
//
// SPDX-License-Identifier: Apache-2.0

use axum::Router;
use log::info;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

pub mod database;
pub mod router;
use crate::database::Database;
use crate::tasks::Banned;
pub mod tasks;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub banned: tasks::BannedCache,
}

impl AppState {
    pub fn new(db: Database) -> Self {
        Self {
            db,
            banned: Arc::new(RwLock::new(Banned::default())),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::new().await?;
    let state = AppState::new(db);

    // Start tasks
    tasks::spawn_stats_cleanup(state.db.clone());
    tasks::spawn_banned_refresh(state.db.clone(), state.banned.clone());

    // Production Path, use vite directly in development
    let client = ServeDir::new("client").fallback(ServeFile::new("client/index.html"));

    // Tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .or_else(|_| EnvFilter::try_new("tribble_tracker=info,tower_http=trace"))?,
        )
        .without_time()
        .init();

    let app = Router::new()
        .nest("/api/v1", router::api::api_router())
        .nest("/internal", router::internal::internal_router())
        .fallback_service(client)
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    info!("listening on {}", listener.local_addr()?);
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
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
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
