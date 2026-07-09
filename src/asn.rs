// SPDX-FileCopyrightText: 2026 The LineageOS Project
//
// SPDX-License-Identifier: Apache-2.0

use std::env;
use std::fmt;
use std::io::BufReader;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use arc_swap::ArcSwapOption;
use asn_db2::IpEntry;

pub type AsnDb = Arc<ArcSwapOption<asn_db2::Database>>;

const ASN_DB_URL: &str = "https://iptoasn.com/data/ip2asn-combined.tsv.gz";
const MAX_AGE: Duration = Duration::from_hours(24);
const MAX_DOWNLOAD_SIZE: u64 = 256 * 1024 * 1024;

#[derive(Debug)]
pub enum AsnError {
    Io(std::io::Error),
    Http(Box<ureq::Error>),
    Parse(asn_db2::Error),
    NotGzip,
}

impl From<std::io::Error> for AsnError {
    fn from(e: std::io::Error) -> Self {
        AsnError::Io(e)
    }
}

impl From<ureq::Error> for AsnError {
    fn from(e: ureq::Error) -> Self {
        AsnError::Http(Box::new(e))
    }
}

impl From<asn_db2::Error> for AsnError {
    fn from(e: asn_db2::Error) -> Self {
        AsnError::Parse(e)
    }
}

impl fmt::Display for AsnError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("asn database error")
    }
}

impl std::error::Error for AsnError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AsnError::Io(e) => Some(e),
            AsnError::Http(e) => Some(e),
            AsnError::Parse(e) => Some(e),
            AsnError::NotGzip => None,
        }
    }
}

/// Returns `None` when the database is not loaded (yet), the address has no
/// match, or the match is unrouted (AS0).
#[must_use]
pub fn lookup(db: &AsnDb, ip: IpAddr) -> Option<i64> {
    let guard = db.load();
    let asn = match guard.as_ref()?.lookup(ip)? {
        IpEntry::V4(e) => e.as_number,
        IpEntry::V6(e) => e.as_number,
    };
    (asn != 0).then_some(i64::from(asn))
}

/// Returns `None` when the database is not loaded (yet), the address has no
/// match, or the match is unrouted (AS0).
#[must_use]
pub fn lookup_asn_owner(db: &AsnDb, asn: u32) -> Option<String> {
    let guard = db.load();
    match guard.as_ref()?.lookup_as_number(asn) {
        Some(IpEntry::V4(e)) => Some(e.owner.to_string()),
        Some(IpEntry::V6(e)) => Some(e.owner.to_string()),
        None => None,
    }
}

fn db_path() -> PathBuf {
    env::var("ASN_DB_PATH")
        .unwrap_or("ip2asn-combined.tsv.gz".to_string())
        .into()
}

fn is_stale(path: &Path) -> bool {
    let Ok(modified) = std::fs::metadata(path).and_then(|m| m.modified()) else {
        return true;
    };
    SystemTime::now()
        .duration_since(modified)
        .is_ok_and(|age| age > MAX_AGE)
}

fn load_from_disk(path: &Path) -> Result<asn_db2::Database, AsnError> {
    let file = std::fs::File::open(path)?;
    let gz = flate2::read::GzDecoder::new(BufReader::new(file));
    Ok(asn_db2::Database::from_reader(BufReader::new(gz))?)
}

async fn reload(db: &AsnDb, path: &Path) {
    let path = path.to_owned();
    match tokio::task::spawn_blocking(move || load_from_disk(&path)).await {
        Ok(Ok(new_db)) => {
            db.store(Some(Arc::new(new_db)));
            println!("asn db loaded");
        }
        Ok(Err(e)) => eprintln!("asn db load failed: {e:?}"),
        Err(e) => eprintln!("asn db load panicked: {e:?}"),
    }
}

fn download(path: &Path) -> Result<(), AsnError> {
    let agent = ureq::Agent::new_with_config(
        ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_mins(5)))
            .build(),
    );
    let bytes = agent
        .get(ASN_DB_URL)
        .call()?
        .body_mut()
        .with_config()
        .limit(MAX_DOWNLOAD_SIZE)
        .read_to_vec()?;
    if !bytes.starts_with(&[0x1f, 0x8b]) {
        return Err(AsnError::NotGzip);
    }
    // atomic replace so a failed download never clobbers a good cache
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, &bytes)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

pub fn spawn_asn_refresh(db: AsnDb) {
    tokio::spawn(async move {
        let path = db_path();

        // Serve whatever is cached on disk right away, even if stale.
        if path.exists() {
            reload(&db, &path).await;
        }

        let mut ticker = tokio::time::interval(MAX_AGE);
        loop {
            ticker.tick().await;
            if is_stale(&path) || db.load().is_none() {
                let dl_path = path.clone();
                match tokio::task::spawn_blocking(move || download(&dl_path)).await {
                    Ok(Ok(())) => reload(&db, &path).await,
                    // keep serving the previous data on failure
                    Ok(Err(e)) => eprintln!("asn db download failed: {e:?}"),
                    Err(e) => eprintln!("asn db download panicked: {e:?}"),
                }
            }
        }
    });
}
