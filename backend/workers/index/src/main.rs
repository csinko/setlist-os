//! ────────────────────────────────────────────────────────────────────────────
//!  WORKER:  INDEX  (queue.index)
//! ────────────────────────────────────────────────────────────────────────────
//! Responsibility
//! --------------
//! • Push (or update) vector embeddings & searchable metadata into Qdrant
//!   so that tracks become queryable (similarity search, text search, etc.).
//!
//! Inputs
//! ------
//!   routing_key="index"
//!   JobEnvelope { file_id, stage=Index }
//!
//! Steps
//! -----
//! 1. Extract embedding (e.g. Jina CLAP, wav2vec, Whisper, etc.).
//! 2. Upsert into Qdrant collection with payload {
//!        track_id, album_id, title, artist, duration, kind, year, mbid …
//!    }
//! 3. UPDATE files.status='READY'
//!
//! Failure → retry; permanent fail → files.status='ERROR_INDEX'.
//!


use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    shared::tracing_init::init("worker-index");

    tracing::info!("INDEX worker stub online – TODO: implement");
    std::future::pending::<()>().await;
    Ok(())
}

