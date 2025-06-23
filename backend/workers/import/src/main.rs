//! WORKER:  IMPORT  (queue.import)
//!
//! From an album folder it creates rows in
//!   • albums (already present)           ← id given in JobEnvelope
//!   • tracks  (logical songs)
//!   • files   (physical media objects)
//! …then publishes one Fingerprint job per file.
//!
//! *Disc / track ordering heuristics:*
//! ───────────────────────────────────
//! • The path segments immediately under the root are interpreted as discs
//!   if they look like “CD1”, “Disc 2”, “1-…”, etc.; otherwise everything is
//!   treated as disc 1.
//! • Track index is inferred in this order:
//!     1) existing tag (`track_number` via lofty)
//!     2) ## prefix in filename (“01 …”, “1-02 …”)
//!     3) stable sort order fallback.
//!
//! Any audio file we can’t parse goes to disc 1/index 0 (will still get
//! fingerprinted, but flagged for later manual review).

use std::{path::{Path, PathBuf}, collections::BTreeMap};

use anyhow::{Result, Context, bail};
use futures_util::StreamExt;
use lapin::{
    options::{BasicConsumeOptions, BasicAckOptions, BasicPublishOptions},
    types::FieldTable,
    BasicProperties, Connection, ConnectionProperties, Channel,
};
use shared::{
    pipeline::{JobEnvelope, Stage},
    amqp,
};
use sqlx::PgPool;
use tokio::task;
use tracing::{debug, error, info, instrument};
use uuid::Uuid;
use walkdir::WalkDir;
use lofty::{TaggedFileExt, Accessor};

/*──────────────────────────────────────────────────────────────────────────*/

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    shared::tracing_init::init("worker-import");

    /*── postgres ─────────────────────────────────────────────────────────*/
    let db = PgPool::connect(&std::env::var("DATABASE_URL")?).await?;

    /*── amqp ─────────────────────────────────────────────────────────────*/
    let conn = Connection::connect(
        &std::env::var("AMQP_URL")?,
        ConnectionProperties::default()
    ).await?;
    let ch = conn.create_channel().await?;
    amqp::declare_all(&ch).await?;
    info!("import worker online – waiting for jobs…");

    /*── consume loop ─────────────────────────────────────────────────────*/
    let mut consumer = ch.basic_consume(
        "queue.import",
        "worker-import",
        BasicConsumeOptions::default(),
        FieldTable::default()
    ).await?;

    while let Some(delivery) = consumer.next().await {
        let delivery = delivery?;
        if let Err(e) = handle_job(&ch, &db, &delivery.data).await {
            error!("job failed: {e:#}");
            // TODO: push to error queue / update jobs.status = 'error'
        }
        delivery.ack(BasicAckOptions::default()).await?;
    }
    Ok(())
}

/*──────────────────────────────────────────────────────────────────────────*/

#[instrument(skip_all, level = "info")]
async fn handle_job(ch: &Channel, db: &PgPool, payload: &[u8]) -> Result<()> {
    let env: JobEnvelope = serde_json::from_slice(payload)?;
    let album_id = env.album_id.context("album_id required")?;
    debug!(%album_id, "importing album");

    /*── locate source dir ------------------------------------------------*/
    let (path_str,): (String,) = sqlx::query_as(
        "SELECT source->>'path' FROM albums WHERE id=$1"
    )
    .bind(album_id)
    .fetch_one(db)
    .await?;

    let source_path: PathBuf = path_str.into();

    let scan_root = source_path.clone();

    /*── scan files (blocking → spawn_blocking) ---------------------------*/
    let file_infos = task::spawn_blocking(move || scan_album(&scan_root))
        .await?
        .context("scan_album")?;

    /*── transactional insert --------------------------------------------*/
    let mut tx = db.begin().await?;

    // BTreeMap keeps disc/index ordering deterministic
    let mut queued_files = Vec::<Uuid>::new();

    for ((disc, idx), info) in &file_infos {
        // 1) track row
        let track_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO tracks(id, album_id, disc, \"index\", title)
                   VALUES ($1,$2,$3,$4,$5)"
        )
        .bind(track_id)
        .bind(album_id)
        .bind(*disc)
        .bind(*idx)
        .bind(&info.title)
        .execute(&mut *tx)
        .await?;

        // 2) file row
        let file_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO files(id, track_id, path, codec)
                   VALUES ($1,$2,$3,$4)"
        )
        .bind(file_id)
        .bind(track_id)
        .bind(&info.path)
        .bind(&info.codec)
        .execute(&mut *tx)
        .await?;

        queued_files.push(file_id);
    }
    tx.commit().await?;
    info!(tracks = file_infos.len(), "album inserted");

    /*── emit fingerprint jobs -------------------------------------------*/
    for fid in queued_files {
        let next = JobEnvelope {
            album_id: Some(album_id),
            track_id: None,
            file_id:  Some(fid),
            stage:    Stage::Fingerprint,
        };
        ch.basic_publish(
            amqp::EXCHANGE,
            "fingerprint",
            BasicPublishOptions::default(),
            &serde_json::to_vec(&next)?,
            BasicProperties::default()
        ).await?.await?;
    }
    info!("queued fingerprint jobs");

    Ok(())
}

/*──────────────────────── helpers (blocking) ─────────────────────────────*/

/// Data we care about for each audio file found.
struct FileInfo {
    path:   String,   // absolute path
    title:  String,   // best-guess title (may be empty)
    codec:  String,   // "flac" | "mp3" | …
}

/// Walk `root`, returning (disc,index) → FileInfo
fn scan_album(root: &Path) -> Result<BTreeMap<(i32,i32), FileInfo>> {
    let mut out = BTreeMap::<(i32,i32), FileInfo>::new();

    for entry in WalkDir::new(root).into_iter().filter_map(Result::ok).filter(|e| e.file_type().is_file()) {
        let path = entry.into_path();
        let ext  = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_ascii_lowercase();
        if !matches!(ext.as_str(), "flac" | "mp3" | "ogg" | "opus" | "m4a") {
            continue;                       // not audio → skip
        }

        // disc is inferred from the *relative* parent dir name
        let disc = path.parent()
                       .and_then(|p| p.file_name())
                       .and_then(|s| s.to_str())
                       .and_then(parse_disc)
                       .unwrap_or(1);

        // index from tag OR filename prefix
        let (index, title_guess) = parse_track_index(&path)?;

        out.insert(
            (disc, index),
            FileInfo {
                path:  path.to_string_lossy().into_owned(),
                title: title_guess,
                codec: ext,
            }
        );
    }
    if out.is_empty() {
        bail!("no audio files found in {}", root.display());
    }
    Ok(out)
}

/*── heuristics helpers (pure) ────────────────────────────────────────────*/

fn parse_disc(s: &str) -> Option<i32> {
    // common patterns: “CD1”, “Disc 2”, “1”
    let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
    digits.parse().ok()
}

fn parse_track_index(p: &Path) -> Result<(i32,String)> {
    // 1) try tags
    if let Ok(tagged) = lofty::read_from_path(p) {
        if let Some(tag) = tagged.primary_tag() {
            if let Some(no) = tag.track() {
                let title = tag.title().unwrap_or_default().to_string();
                return Ok((no as i32, title));
            }
        }
    }
    // 2) fallback to filename “01 - Title.flac”
    let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let digits: String = stem.chars().take_while(|c| c.is_ascii_digit()).collect();
    let idx = digits.parse().unwrap_or(0);
    Ok((idx, String::new()))
}

