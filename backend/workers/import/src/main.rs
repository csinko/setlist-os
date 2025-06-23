//! ────────────────────────────────────────────────────────────────────────────
//!  WORKER:  IMPORT  (queue.import)
//! ────────────────────────────────────────────────────────────────────────────
//! Primary responsibility
//! ----------------------
//! • Convert a *finished* album upload (or remote download) into a structured
//!   DB representation: albums → tracks → files.
//!
//! Trigger / Routing-Key
//! ---------------------
//!   exchange=jobs   routing_key="import"
//!   payload: JobEnvelope { album_id,  … , stage=Import }
//!
//! Input invariants
//! ----------------
//! • `albums(id)` row already exists and `source` informs WHERE the files live:
//!     { "type":"upload", "path":"/inbox/<uuid>" }   ← local temp dir
//!     { "type":"remote", "url":"https://..."   }    ← fetched by fetch-worker
//!
//! Steps (happy path)
//! ------------------
//! 1. Locate album directory (fail if missing).
//! 2. Enumerate audio files (sort stable; folders ⇒ disc number).
//! 3. For each file create:
//!        tracks(id, album_id, disc, index, title, duration_sec)
//!      •  files(id, track_id, path, codec, status='NEW')
//! 4. For each *file* emit a Fingerprint job:
//!        routing_key="fingerprint"
//!        JobEnvelope { file_id, stage=Fingerprint, … }
//!
//! Edge-cases / errors
//! -------------------
//! • Non-audio files → ignore (log at debug).
//! • Duplicate track index → fallback to filename sort order.
//! • On partial failure: mark album `source.status = "error"` and requeue?
//!
//! DB writes must be in a single transaction to keep album state coherent.
//!


use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    shared::tracing_init::init("worker-import");
    tracing::info!("IMPORT worker stub online – TODO: implement");
    std::future::pending::<()>().await;
    Ok(())
}

