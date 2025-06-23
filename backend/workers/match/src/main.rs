//! ────────────────────────────────────────────────────────────────────────────
//!  WORKER:  MATCH  (queue.match)
//! ────────────────────────────────────────────────────────────────────────────
//! Responsibility
//! --------------
//! • Use AcoustID / MusicBrainz (or internal ML) to match fingerprints to
//!   canonical recordings, retrieving metadata candidates.
//!
//! Trigger
//! -------
//!   routing_key="match"
//!   payload: JobEnvelope { file_id, stage=Match }
//
//! Steps
//! -----
//! 1. Get fingerprint (needs to be stored or recomputed quickly).
//! 2. Call external service(s), collect candidate Recording MBIDs & scores.
//! 3. INSERT INTO matches(file_id, mbid, score, raw_json)
//! 4. Decide:
//!      – High-confidence → emit Tag job.
//!      – Low confidence  → mark for manual review (future).
//!
//! DB additions (future)
//! --------------------
//!   TABLE matches (file_id, mbid, score, raw, chosen BOOL)
//!
//! Output
//! ------
//!   • Tag job on success:
//!        routing_key="tag"
//!        JobEnvelope { file_id, stage=Tag }
//!


use anyhow::Result;
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("worker_match=debug".parse()?))
        .init();

    tracing::info!("MATCH worker stub online – TODO: implement");
    std::future::pending::<()>().await;
    Ok(())
}

