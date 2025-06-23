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

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    shared::tracing_init::init("worker-fetch");
    tracing::info!("FETCH worker stub online – TODO: implement");
    std::future::pending::<()>().await;
    Ok(())
}

