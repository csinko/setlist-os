// tests/import_flow.rs
//! Full round-trip: album folder → Import worker → tracks/files rows & FP jobs.

use e2e::harness::prelude::*;
use reqwest::Client;
use std::{
    env,
    fs,
    path::PathBuf,
    time::{Duration, Instant},
};
use tokio::process::Command;

#[tokio::test]
async fn import_creates_tracks_and_fp_jobs() -> Result<()> {
    let t0 = Instant::now();
    let infra = Infra::spin_up()?;

    /*──  launch API + Import worker  ────────────────────────────────────*/
    let api_bin    = env::var("API_BIN").context("API_BIN not set")?;
    let import_bin = env::var("IMPORT_BIN")
        .unwrap_or_else(|_| "../target/debug/worker-import".into());

    let (_api,   _api_log)   = spawn_with_logs(
        "API",
        &api_bin,
        &[("DATABASE_URL", &infra.db_url), ("AMQP_URL", &infra.amqp_url)],
        34,
    )?;
    let (_imp,   _imp_log)   = spawn_with_logs(
        "IMPORT",
        &import_bin,
        &[("DATABASE_URL", &infra.db_url), ("AMQP_URL", &infra.amqp_url)],
        35,
    )?;

    wait_for_http_ok("http://127.0.0.1:8080/internal/health", Duration::from_secs(10)).await?;

    /*──  prepare a temp album dir with two tiny FLACs  ──────────────────*/
    let tmp_root   = tempfile::tempdir()?;
    let album_dir  = tmp_root.path().join(Uuid::new_v4().to_string());
    fs::create_dir(&album_dir)?;
    for n in 1..=2 {
        let file = album_dir.join(format!("{n:02}.flac"));
        Command::new("ffmpeg")
            .args([
                "-f","lavfi","-i","anullsrc=r=44100:cl=stereo",
                "-t","1","-c:a","flac",
                file.to_str().unwrap(),
                "-y","-loglevel","error",
            ])
            .status().await?;
    }

    /*──  create album in DB & update source.path  ───────────────────────*/
    let client   = Client::new();
    let album_id: Uuid = client
        .post("http://127.0.0.1:8080/albums")
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    sqlx::query(
        "UPDATE albums
            SET source = jsonb_build_object('type','upload','path',$1)
          WHERE id = $2",
    )
    .bind(album_dir.to_str().unwrap())
    .bind(album_id)
    .execute(&sqlx::PgPool::connect(&infra.db_url).await?)
    .await?;

    /*──  kick the Import stage  ─────────────────────────────────────────*/
    client
        .put(format!("http://127.0.0.1:8080/albums/{album_id}/complete"))
        .send()
        .await?
        .error_for_status()?;

    /*──  give the worker a moment  ──────────────────────────────────────*/
    tokio::time::sleep(Duration::from_secs(3)).await;

    /*──  assertions  ────────────────────────────────────────────────────*/
    let pool = sqlx::PgPool::connect(&infra.db_url).await?;

    let (tracks,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM tracks WHERE album_id=$1",
    )
    .bind(album_id)
    .fetch_one(&pool)
    .await?;
    assert_eq!(tracks, 2, "2 tracks expected");

    let (files,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM files f
          JOIN tracks t ON t.id = f.track_id
         WHERE t.album_id=$1",
    )
    .bind(album_id)
    .fetch_one(&pool)
    .await?;
    assert_eq!(files, 2, "2 files expected");

    let (fp_jobs,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM jobs
          WHERE stage='fingerprint' AND payload->>'album_id' = $1",
    )
    .bind(album_id.to_string())
    .fetch_one(&pool)
    .await?;
    assert_eq!(fp_jobs, 2, "2 fingerprint jobs queued");

    println!("✔ import flow OK in {:.1?}", t0.elapsed());
    Ok(())
}

