// tests/album_flow.rs
//! Smoke-test: create an album through the API → make sure an *import* job is queued.

use e2e::harness::prelude::*;
use reqwest::Client;
use std::time::{Duration, Instant};

#[tokio::test]
async fn album_upload_flow() -> Result<()> {
    let t0 = Instant::now();

    /*──  infra  ─────────────────────────────────────────────────────────*/
    let infra = Infra::spin_up()?;

    /*──  launch API  ────────────────────────────────────────────────────*/
    let api_bin = std::env::var("API_BIN")
        .context("env var API_BIN (path to `api` binary) not set")?;
    let (_api, _api_log) = spawn_with_logs(
        "API",
        &api_bin,
        &[("DATABASE_URL", &infra.db_url), ("AMQP_URL", &infra.amqp_url)],
        34,
    )?;

    /*──  wait for readiness  ────────────────────────────────────────────*/
    wait_for_http_ok("http://127.0.0.1:8080/internal/health", Duration::from_secs(10)).await?;
    println!("API up in {:.1?}", t0.elapsed());

    /*──  1️⃣  create album  ─────────────────────────────────────────────*/
    let client = Client::new();
    let album_id: Uuid = client
        .post("http://127.0.0.1:8080/albums")
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    println!("created album {album_id}");

    /*──  2️⃣  mark complete (queues Import)  ────────────────────────────*/
    client
        .put(format!("http://127.0.0.1:8080/albums/{album_id}/complete"))
        .send()
        .await?
        .error_for_status()?;

    /*──  3️⃣  assert Import job queued  ─────────────────────────────────*/
    let (cnt,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM jobs
          WHERE payload->>'album_id' = $1
            AND stage = 'import'
            AND status = 'queued'",
    )
    .bind(album_id.to_string())
    .fetch_one(&sqlx::PgPool::connect(&infra.db_url).await?)
    .await?;

    assert_eq!(cnt, 1, "exactly one import job queued");

    println!("✔ album flow OK in {:.1?}", t0.elapsed());
    Ok(())
}

