//! ────────────────────────────────────────────────────────────────────────────
//!  WORKER:  TAG  (queue.tag)
//! ────────────────────────────────────────────────────────────────────────────
//! Responsibility
//! --------------
//! • Embed final metadata & artwork into the audio file (ID3/FLAC tags).
//!
//! Inputs
//! ------
//!   routing_key="tag"
//!   JobEnvelope { file_id, stage=Tag }
//!
//! Steps
//! -----
//! 1. Fetch definitive metadata (joins albums ⋈ tracks ⋈ matches).
//! 2. Use `beet`/`mutagen`/`audion`/`metaflac` to write tags in-place.
//! 3. UPDATE files.status='TAG_DONE', tagged_at=NOW()
//! 4. Publish Index job:
//!        routing_key="index"
//!        JobEnvelope { file_id, stage=Index }
//!
//! Considerations
//! --------------
//! • Artwork download/caching directory.
//! • Atomic file replacement (write to tmp, rename).
//!


use anyhow::Result;
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("worker_tag=debug".parse()?))
        .init();

    tracing::info!("TAG worker stub online – TODO: implement");
    std::future::pending::<()>().await;
    Ok(())
}

