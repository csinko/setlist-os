//! ────────────────────────────────────────────────────────────────────────────
//!  WORKER:  FETCH  (queue.fetch)
//! ────────────────────────────────────────────────────────────────────────────
//! Responsibility
//! --------------
//! • Download remote archives (Archive.org, Bandcamp, web seed, etc.) to a
//!   transient directory and dispatch an Import job.
//!
//! Trigger
//! -------
//!   routing_key="fetch"
//!   JobEnvelope { album_id, stage=Fetch }
//
//! Steps
//! -----
//! 1. Look up albums.source -> {type:"remote", url:...}.
//! 2. Stream/zip/tar download → `/inbox/<album_id>/raw/...`.
//! 3. Validate MIME, size limits, disk quota.
//! 4. Emit Import job:
//!        routing_key="import"
//!        JobEnvelope { album_id, stage=Import }
//!
//! Cleanup strategy
//! ----------------
//! • Delete temp files if Import never starts after N hours.
//!


use anyhow::Result;
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("worker_fetch=debug".parse()?))
        .init();

    tracing::info!("FETCH worker stub online – TODO: implement");
    std::future::pending::<()>().await;
    Ok(())
}

