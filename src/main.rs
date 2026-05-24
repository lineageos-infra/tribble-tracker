// SPDX-FileCopyrightText: 2026 The LineageOS Project
//
// SPDX-License-Identifier: Apache-2.0

use axum::Router;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use tower_http::services::{ServeDir, ServeFile};

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

    let app = Router::new()
        .nest("/api/v1", router::api::api_router())
        .nest("/internal", router::internal::internal_router())
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
