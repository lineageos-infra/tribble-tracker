// SPDX-FileCopyrightText: 2026 The LineageOS Project
//
// SPDX-License-Identifier: Apache-2.0

use axum::Router;
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

    let public_router = Router::new()
        .nest("/api/v1", router::api::api_router())
        .fallback_service(client)
        .with_state(state.clone());
    let internal_router = Router::new()
        .nest("/internal", router::internal::internal_router())
        .with_state(state.clone());

    let public_listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    let internal_listener = tokio::net::TcpListener::bind("127.0.0.1:8081").await?;

    println!("listening on {}", public_listener.local_addr().unwrap());
    tokio::try_join!(
        axum::serve(public_listener, public_router.into_make_service())
            .with_graceful_shutdown(shutdown_signal()),
        axum::serve(internal_listener, internal_router.into_make_service())
            .with_graceful_shutdown(shutdown_signal()),
    )?;

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
