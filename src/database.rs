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
    pub affected_installations: Option<i64>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct TotalInstallationsItem {
    pub model: String,
    pub version_raw: String,
    pub asn: i64,
    pub installations: i64,
}

#[derive(Serialize)]
pub struct TopAsnItem {
    pub asn: i64,
    pub asn_owner: String,
    pub devices: i64,
    pub top_model: String,
    pub top_model_count: i64,
}

pub struct NewStat<'a> {
    pub device_id: &'a str,
    pub carrier: &'a str,
    pub carrier_id: &'a str,
    pub country: &'a str,
    pub model: &'a str,
    pub official: bool,
    pub version: &'a str,
    pub version_raw: &'a str,
    pub asn: i64,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum GroupCol {
    Model,
    Country,
    Version,
    Carrier,
    Asn,
}

impl GroupCol {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Model => "model",
            Self::Country => "country",
            Self::Version => "version",
            Self::Carrier => "carrier",
            Self::Asn => "asn",
        }
    }
}

impl fmt::Display for GroupCol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(sqlx::FromRow)]
pub struct GroupedCount {
    pub name: String,
    pub count: i64,
}

pub struct FilterClause<'a> {
    pub column: GroupCol,
    pub value: &'a str,
}

impl Database {
    /// # Errors
    ///
    /// Returns a [`DbError`] if connecting to the database or running migrations fails.
    pub async fn new() -> Result<Self, DbError> {
        let database_url = env::var("DATABASE_URL").unwrap_or("sqlite:dev.db".to_string());
        let pool = SqlitePool::connect(&database_url).await?;
        sqlx::migrate!().run(&pool).await?;

        Ok(Self { pool })
    }

    /// # Errors
    ///
    /// Returns a [`DbError`] if the delete query fails.
    pub async fn delete_old_stats(&self) -> Result<u64, DbError> {
        let res = sqlx::query!("DELETE FROM stats WHERE submit_time < datetime('now', '-90 days')")
            .execute(&self.pool)
            .await?;
        Ok(res.rows_affected())
    }

    /// # Errors
    ///
    /// Returns a [`DbError`] if the upsert query fails.
    pub async fn upsert_stat(&self, stat: NewStat<'_>) -> Result<(), DbError> {
        sqlx::query!(
            r#"
            INSERT INTO stats (device_id, carrier, carrier_id, country, model, official, version, version_raw, asn)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT (device_id) DO UPDATE SET
                carrier = excluded.carrier,
                carrier_id = excluded.carrier_id,
                country = excluded.country,
                model = excluded.model,
                official = excluded.official,
                version = excluded.version,
                version_raw = excluded.version_raw,
                asn = IIF(excluded.asn = 0, asn, excluded.asn)
            "#,
            stat.device_id,
            stat.carrier,
            stat.carrier_id,
            stat.country,
            stat.model,
            stat.official,
            stat.version,
            stat.version_raw,
            stat.asn,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    fn append_filters(qb: &mut sqlx::QueryBuilder<sqlx::Sqlite>, filters: &[FilterClause<'_>]) {
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
    }

    /// # Errors
    ///
    /// Returns a [`DbError`] if the query fails.
    pub async fn fetch_grouped_counts(
        &self,
        group: GroupCol,
        filters: &[FilterClause<'_>],
    ) -> Result<Vec<GroupedCount>, DbError> {
        let mut qb = sqlx::QueryBuilder::new(format!(
            "SELECT {group} as name, COUNT(*) as count FROM stats"
        ));

        Self::append_filters(&mut qb, filters);

        qb.push(format!(" GROUP BY {group} ORDER BY count DESC LIMIT 250"));
        let rows = qb
            .build_query_as::<GroupedCount>()
            .fetch_all(&self.pool)
            .await?;
        Ok(rows)
    }

    /// # Errors
    ///
    /// Returns a [`DbError`] if the query fails.
    pub async fn fetch_total(&self, filters: &[FilterClause<'_>]) -> Result<i64, DbError> {
        let mut qb = sqlx::QueryBuilder::new("SELECT COUNT(*) FROM stats");

        Self::append_filters(&mut qb, filters);

        let total = qb.build_query_scalar::<i64>().fetch_one(&self.pool).await?;
        Ok(total)
    }

    /// # Errors
    ///
    /// Returns a [`DbError`] if the query fails.
    pub async fn fetch_official_total(&self, filters: &[FilterClause<'_>]) -> Result<i64, DbError> {
        let mut qb = sqlx::QueryBuilder::new("SELECT COUNT(*) FROM stats");

        Self::append_filters(&mut qb, filters);

        qb.push(if filters.is_empty() {
            " WHERE official = 1"
        } else {
            " AND official = 1"
        });

        let total = qb.build_query_scalar::<i64>().fetch_one(&self.pool).await?;
        Ok(total)
    }

    /// # Errors
    ///
    /// Returns a [`DbError`] if the query fails.
    pub async fn fetch_total_installations(
        &self,
        filters: &[FilterClause<'_>],
    ) -> Result<Vec<TotalInstallationsItem>, DbError> {
        let mut qb = sqlx::QueryBuilder::new(
            "SELECT model, version_raw, asn, COUNT(*) AS installations FROM stats",
        );

        Self::append_filters(&mut qb, filters);

        qb.push(" GROUP BY version_raw, asn ORDER BY installations DESC");

        let items = qb
            .build_query_as::<TotalInstallationsItem>()
            .fetch_all(&self.pool)
            .await?;
        Ok(items)
    }

    /// # Errors
    ///
    /// Returns a [`DbError`] if the query fails.
    pub async fn list_bans(&self) -> Result<Vec<BannedItem>, DbError> {
        // join with stats to get affected installations count
        let items = sqlx::query_as!(
            BannedItem,
            r#"
            SELECT
                ban.version,
                ban.model,
                ban.note,
                (
                    SELECT COUNT(*)
                    FROM stats stat
                    WHERE
                        (ban.version IS NOT NULL AND stat.version_raw = ban.version)
                        OR
                        (ban.model IS NOT NULL AND stat.model = ban.model)
                ) AS affected_installations
            FROM banned ban;
            "#
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(items)
    }

    /// # Errors
    ///
    /// Returns a [`DbError`] if the query fails.
    pub async fn fetch_top_asns(&self) -> Result<Vec<TopAsnItem>, DbError> {
        // bare `model` picks the value from the MAX(devices) row (SQLite argmax)
        let items = sqlx::query_as!(
            TopAsnItem,
            r#"
            WITH by_model AS (
                SELECT asn, model, COUNT(*) AS devices
                FROM stats
                WHERE asn != 0
                GROUP BY asn, model
            )
            SELECT
                asn AS "asn!: i64",
                "" AS "asn_owner!: String",
                SUM(devices) AS "devices!: i64",
                model AS "top_model!: String",
                MAX(devices) AS "top_model_count!: i64"
            FROM by_model
            GROUP BY asn
            ORDER BY SUM(devices) DESC
            LIMIT 50
            "#
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(items)
    }

    /// # Errors
    ///
    /// Returns a [`DbError`] if the purge query fails.
    pub async fn reap_bans(&self) -> Result<u64, DbError> {
        let bans = self.list_bans().await?;
        let mut qb = sqlx::QueryBuilder::new("DELETE FROM stats WHERE ");

        qb.push("version_raw IN (");
        {
            let mut separated = qb.separated(", ");
            for version in bans.iter().filter_map(|b| b.version.as_ref()) {
                separated.push_bind(version);
            }
        }
        qb.push(") OR model IN (");
        {
            let mut separated = qb.separated(", ");
            for model in bans.iter().filter_map(|b| b.model.as_ref()) {
                separated.push_bind(model);
            }
        }
        qb.push(")");

        let result = qb.build().execute(&self.pool).await?;
        Ok(result.rows_affected())
    }

    /// # Errors
    ///
    /// Returns a [`DbError`] if the delete query fails.
    pub async fn remove_bans(&self, col: GroupCol, values: &[String]) -> Result<(), DbError> {
        let mut qb = sqlx::QueryBuilder::new(format!("DELETE FROM banned WHERE {col}"));

        let mut separated = qb.separated(", ");
        separated.push_unseparated(" IN (");
        for value in values {
            separated.push_bind(value);
        }
        separated.push_unseparated(")");

        qb.build().execute(&self.pool).await?;
        Ok(())
    }

    /// # Errors
    ///
    /// Returns a [`DbError`] if the upsert query fails.
    pub async fn upsert_bans(
        &self,
        col: GroupCol,
        values: &[String],
        note: Option<&str>,
    ) -> Result<(), DbError> {
        let mut qb = sqlx::QueryBuilder::new(format!("INSERT INTO banned ({col}, note)"));

        qb.push_values(values, |mut b, value| {
            b.push_bind(value).push_bind(note);
        });

        qb.push(format!(
            " ON CONFLICT ({col}) DO UPDATE SET note = excluded.note"
        ));

        qb.build().execute(&self.pool).await?;
        Ok(())
    }
}
