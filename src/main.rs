// SPDX-FileCopyrightText: 2026 The LineageOS Project
//
// SPDX-License-Identifier: Apache-2.0

use arc_swap::ArcSwapOption;
use axum::Router;
use axum_client_ip::ClientIpSource;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::services::{ServeDir, ServeFile};

pub mod asn;
pub mod database;
pub mod router;
use crate::database::Database;
use crate::tasks::Banned;
pub mod tasks;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub banned: tasks::BannedCache,
    pub asn_db: asn::AsnDb,
}

impl AppState {
    #[must_use]
    pub fn new(db: Database) -> Self {
        Self {
            db,
            banned: Arc::new(RwLock::new(Banned::default())),
            asn_db: Arc::new(ArcSwapOption::empty()),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::new().await?;
    let state = AppState::new(db);

    // Start tasks
    tasks::spawn_stats_cleanup(state.db.clone());
    tasks::spawn_banned_refresh(state.db.clone(), state.banned.clone()).await;
    asn::spawn_asn_refresh(state.asn_db.clone());

    // Production Path, use vite directly in development
    let client = ServeDir::new("client").fallback(ServeFile::new("client/index.html"));

    // Defaults to Cloudflare; Use CLIENT_IP_SOURCE=ConnectInfo locally
    let ip_source = match std::env::var("CLIENT_IP_SOURCE") {
        Ok(s) => s.parse()?,
        Err(_) => ClientIpSource::CfConnectingIp,
    };

    let app = Router::new()
        .nest("/api/v1", router::api::api_router())
        .nest("/internal", router::internal::internal_router())
        .fallback_service(client)
        .layer(ip_source.into_extension())
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
        () = ctrl_c => {},
        () = terminate => {},
    }
}
