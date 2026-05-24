// SPDX-FileCopyrightText: 2026 The LineageOS Project
//
// SPDX-License-Identifier: Apache-2.0

use std::env;
use std::fmt;

use serde::Serialize;
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

#[derive(Debug)]
pub enum DbError {
    Sqlx(sqlx::Error),
    Migrate(sqlx::migrate::MigrateError),
}

impl From<sqlx::Error> for DbError {
    fn from(e: sqlx::Error) -> Self {
        DbError::Sqlx(e)
    }
}

impl From<sqlx::migrate::MigrateError> for DbError {
    fn from(e: sqlx::migrate::MigrateError) -> Self {
        DbError::Migrate(e)
    }
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("database error")
    }
}

impl std::error::Error for DbError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DbError::Sqlx(e) => Some(e),
            DbError::Migrate(e) => Some(e),
        }
    }
}

#[derive(Serialize)]
pub struct BannedItem {
    pub version: Option<String>,
    pub model: Option<String>,
    pub note: Option<String>,
}

pub struct NewStat<'a> {
    pub device_id: &'a str,
    pub carrier: Option<&'a str>,
    pub carrier_id: Option<&'a str>,
    pub country: &'a str,
    pub model: &'a str,
    pub official: bool,
    pub version: &'a str,
    pub version_raw: &'a str,
}

#[derive(Clone, Copy)]
pub enum GroupCol {
    Model,
    Country,
    Version,
    Carrier,
}

impl GroupCol {
    fn as_str(self) -> &'static str {
        match self {
            Self::Model => "model",
            Self::Country => "country",
            Self::Version => "version",
            Self::Carrier => "carrier",
        }
    }
}

pub struct FilterClause<'a> {
    pub column: &'static str,
    pub value: &'a str,
}

impl Database {
    pub async fn new() -> Result<Self, DbError> {
        let database_url = env::var("DATABASE_URL").unwrap_or("sqlite:dev.db".to_string());
        let pool = SqlitePool::connect(&database_url).await?;
        sqlx::migrate!().run(&pool).await?;

        Ok(Self { pool })
    }

    pub async fn delete_old_stats(&self) -> Result<u64, DbError> {
        let res = sqlx::query!("DELETE FROM stats WHERE submit_time < datetime('now', '-90 days')")
            .execute(&self.pool)
            .await?;
        Ok(res.rows_affected())
    }

    pub async fn upsert_stat(&self, stat: NewStat<'_>) -> Result<(), DbError> {
        sqlx::query!(
            r#"
            INSERT INTO stats (device_id, carrier, carrier_id, country, model, official, version, version_raw)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(device_id) DO UPDATE SET
                carrier = excluded.carrier,
                carrier_id = excluded.carrier_id,
                country = excluded.country,
                model = excluded.model,
                official = excluded.official,
                version = excluded.version,
                version_raw = excluded.version_raw
            "#,
            stat.device_id,
            stat.carrier,
            stat.carrier_id,
            stat.country,
            stat.model,
            stat.official,
            stat.version,
            stat.version_raw,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn fetch_grouped_counts(
        &self,
        group: GroupCol,
        filters: &[FilterClause<'_>],
    ) -> Result<Vec<(String, i64)>, DbError> {
        let col = group.as_str();
        let mut qb = sqlx::QueryBuilder::new(format!(
            "SELECT {col}, COUNT(*) FROM stats WHERE {col} IS NOT NULL AND {col} != ''"
        ));
        for filter in filters {
            qb.push(" AND ")
                .push(filter.column)
                .push(" = ")
                .push_bind(filter.value);
        }
        qb.push(format!(" GROUP BY {col} ORDER BY 2 DESC LIMIT 250"));
        let rows = qb
            .build_query_as::<(String, i64)>()
            .fetch_all(&self.pool)
            .await?;
        Ok(rows)
    }

    pub async fn fetch_total(&self, filters: &[FilterClause<'_>]) -> Result<i64, DbError> {
        let mut qb = sqlx::QueryBuilder::new("SELECT COUNT(*) FROM stats");

        if !filters.is_empty() {
            qb.push(" WHERE ");
            let mut separated = qb.separated(" AND ");
            for filter in filters {
                separated
                    .push(filter.column)
                    .push_unseparated(" = ")
                    .push_bind_unseparated(filter.value);
            }
        }

        let total = qb.build_query_scalar::<i64>().fetch_one(&self.pool).await?;
        Ok(total)
    }

    pub async fn list_bans(&self) -> Result<Vec<BannedItem>, DbError> {
        let items = sqlx::query_as!(BannedItem, r#"SELECT version, model, note FROM banned"#)
            .fetch_all(&self.pool)
            .await?;
        Ok(items)
    }

    pub async fn upsert_banned_model(
        &self,
        model: &str,
        note: Option<&str>,
    ) -> Result<(), DbError> {
        sqlx::query!(
            "INSERT INTO banned (model, note) VALUES (?, ?)
             ON CONFLICT (model) DO UPDATE SET note = excluded.note",
            model,
            note,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn upsert_banned_version(
        &self,
        version: &str,
        note: Option<&str>,
    ) -> Result<(), DbError> {
        sqlx::query!(
            "INSERT INTO banned (version, note) VALUES (?, ?)
             ON CONFLICT (version) DO UPDATE SET note = excluded.note",
            version,
            note,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
