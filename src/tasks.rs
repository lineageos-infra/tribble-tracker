// SPDX-FileCopyrightText: 2026 The LineageOS Project
//
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashSet;
use std::sync::{Arc, RwLock};

async fn drop_old_stats(pool: &sqlx::SqlitePool) -> Result<u64, sqlx::Error> {
    let res = sqlx::query!("DELETE FROM stats WHERE submit_time < datetime('now', '-90 days')")
        .execute(pool)
        .await?;
    Ok(res.rows_affected())
}

pub fn spawn_stats_cleanup(pool: sqlx::SqlitePool) {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(std::time::Duration::from_secs(24 * 60 * 60));
        loop {
            ticker.tick().await;
            match drop_old_stats(&pool).await {
                Ok(n) => println!("dropped {n} old stats rows"),
                Err(e) => eprintln!("drop_old_stats failed: {e:?}"),
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

pub async fn refresh_banned(
    pool: &sqlx::SqlitePool,
    cache: &BannedCache,
) -> Result<(), sqlx::Error> {
    let rows = sqlx::query!("SELECT version, model FROM banned")
        .fetch_all(pool)
        .await?;
    let mut next = Banned::default();
    for r in rows {
        if let Some(v) = r.version {
            next.versions.insert(v);
        }
        if let Some(m) = r.model {
            next.models.insert(m);
        }
    }
    *cache.write().unwrap() = next;
    Ok(())
}

pub fn spawn_banned_refresh(pool: sqlx::SqlitePool, banned: BannedCache) {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(std::time::Duration::from_secs(60));
        ticker.tick().await; // skip the immediate tick — we just loaded
        loop {
            ticker.tick().await;
            if let Err(e) = refresh_banned(&pool, &banned).await {
                eprintln!("failed to refresh banned list: {e:?}");
            }
        }
    });
}
