// backend/tools/scanner/src/main.rs
//! Media-library scanner (stub).
//!
//! This binary will eventually:
//!   1. Walk a configured media directory (env var `MEDIA_ROOT` or `/media`).
//!   2. Detect new albums / files.
//!   3. Publish `FileJob { stage = Stage::Fingerprint }` messages to RabbitMQ.
//!
//! For now it just boots, initialises logging, and exits successfully.

use anyhow::Result;
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env if present and initialise tracing at info level (override with RUST_LOG).
    dotenvy::dotenv().ok();
    fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    tracing::info!("scanner stub – nothing to do yet");
    Ok(())
}

