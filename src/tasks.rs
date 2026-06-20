// SPDX-FileCopyrightText: 2026 The LineageOS Project
//
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashSet;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::database::{Database, DbError};

pub fn spawn_stats_cleanup(db: Database) {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(std::time::Duration::from_hours(24));
        loop {
            ticker.tick().await;
            match db.delete_old_stats().await {
                Ok(n) => println!("dropped {n} old stats rows"),
                Err(e) => eprintln!("delete_old_stats failed: {e:?}"),
            }
        }
    });
}

#[derive(Default)]
pub struct Banned {
    pub versions: HashSet<String>,
    pub models: HashSet<String>,
}

pub type BannedCache = Arc<RwLock<Banned>>;

/// # Errors
///
/// Returns a [`DbError`] if fetching bans from the database fails.
pub async fn refresh_banned(db: &Database, cache: &BannedCache) -> Result<(), DbError> {
    let rows = db.list_bans().await?;
    let mut next = Banned::default();
    for r in rows {
        if let Some(v) = r.version {
            next.versions.insert(v);
        }
        if let Some(m) = r.model {
            next.models.insert(m);
        }
    }
    *cache.write().await = next;
    Ok(())
}

pub async fn spawn_banned_refresh(db: Database, banned: BannedCache) {
    if let Err(e) = refresh_banned(&db, &banned).await {
        eprintln!("failed to refresh banned list: {e:?}");
    }

    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(std::time::Duration::from_mins(1));
        loop {
            ticker.tick().await;
            if let Err(e) = refresh_banned(&db, &banned).await {
                eprintln!("failed to refresh banned list: {e:?}");
            }
        }
    });
}
