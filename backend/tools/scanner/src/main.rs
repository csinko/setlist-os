//! ────────────────────────────────────────────────────────────────────────────
//!  TOOL:  SCANNER  (manual / cron)
//! ────────────────────────────────────────────────────────────────────────────
//! Responsibility
//! --------------
//! • Walk the long-term media directory (`/media`) to detect new albums that
//!   were added *outside* the pipeline (e.g. rsync, torrent).
//! • For each new folder:
//!       INSERT albums(... source={"type":"library_scan"} )
//!       queue Import job
//! • For removed folders: optionally tombstone albums / files.
//
//! Notes
//! -----
//! • This is *not* a worker; run ad-hoc or via systemd timer.
//! • Could store a per-scan manifest to detect renames/moves.
//!


use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env if present and initialise tracing at info level (override with RUST_LOG).
    dotenvy::dotenv().ok();
    shared::tracing_init::init("scanner-tool");

    tracing::info!("scanner stub – nothing to do yet");
    Ok(())
}

