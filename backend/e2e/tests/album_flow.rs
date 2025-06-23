//! backend/e2e/tests/album_flow.rs
//! End-to-end smoke-test for the new album workflow.

use anyhow::{Context, Result};
use reqwest::Client;
use std::{env, time::Instant};
use testcontainers::clients::Cli;
use testcontainers_modules::{postgres::Postgres, rabbitmq::RabbitMq};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::{Child, Command},
    task::JoinHandle,
};
use uuid::Uuid;
use serde_json;

/*──────────────────────── helpers ─────────────────────────────────────────*/

/// Spawn a binary, stream its stdout/stderr colour-tagged, kill on drop.
fn spawn_with_logs(
    tag: &str,
    bin: &str,
    envs: &[(&str, &str)],
    color: u8,
) -> Result<(Child, JoinHandle<()>)> {
    let mut cmd = Command::new(bin);
    cmd.stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);
    for &(k, v) in envs {
        cmd.env(k, v);
    }
    let mut child = cmd.spawn().with_context(|| format!("spawn {tag}"))?;
    println!(
        "\x1b[1;{color}m{tag} launched (pid={})\x1b[0m",
        child.id().unwrap_or(0)
    );

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    let prefix = format!("\x1b[1;{color}m{tag}\x1b[0m");

    let handle = tokio::spawn(async move {
        let mut out = BufReader::new(stdout).lines();
        let mut err = BufReader::new(stderr).lines();
        loop {
            tokio::select! {
                l = out.next_line() => match l {
                    Ok(Some(line)) => println!("{prefix} {line}"),
                    _               => break,
                },
                l = err.next_line() => match l {
                    Ok(Some(line)) => println!("{prefix} {line}"),
                    _               => break,
                },
            }
        }
    });

    Ok((child, handle))
}

/*──────────────────────── the test ───────────────────────────────────────*/

#[tokio::test]
async fn album_upload_flow() -> Result<()> {
    let t0 = Instant::now();

    /*── Docker infra ─────────────────────────────────────────────────────*/
    let docker  = Cli::default();
    let pg      = docker.run(Postgres::default());
    let mq      = docker.run(RabbitMq::default());

    let db_url  = format!(
        "postgres://postgres:postgres@localhost:{}/postgres",
        pg.get_host_port_ipv4(5432)
    );
    let amqp_url = format!(
        "amqp://guest:guest@localhost:{}/%2f",
        mq.get_host_port_ipv4(5672)
    );

    /*── Launch the API ───────────────────────────────────────────────────*/
    let api_bin = env::var("API_BIN").context("API_BIN not set (see scripts/test.sh)")?;
    let (api, api_log) = spawn_with_logs(
        "API",
        &api_bin,
        &[("DATABASE_URL", &db_url), ("AMQP_URL", &amqp_url)],
        34,
    )?;
    let _guard = (api, api_log);            // keep processes alive

    /*── Wait for /health ─────────────────────────────────────────────────*/
    let client = Client::new();
    loop {
        if let Ok(r) = client
            .get("http://127.0.0.1:8080/internal/health")
            .send()
            .await
        {
            if r.status().is_success() {
                break;
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(120)).await;
    }
    println!("API up in {:.1?}", t0.elapsed());

    /*── 1. create album ──────────────────────────────────────────────────*/
    let res  = client.post("http://127.0.0.1:8080/albums").send().await?;
    let status = res.status();
    let body = res.text().await?;
    if !status.is_success() {
        anyhow::bail!("create_album failed: {} – {}", status, body);
    }
    let album_id: Uuid = serde_json::from_str(&body)?;
    println!("created album {album_id}");

    /*── 2. mark album complete (queues Import) ───────────────────────────*/
    let r = client
        .put(format!("http://127.0.0.1:8080/albums/{album_id}/complete"))
        .send()
        .await?;
    assert!(r.status().is_success());

    /*── 3. assert jobs table has the Import row ──────────────────────────*/
    let (cnt,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM jobs
            WHERE payload->>'album_id' = $1
              AND stage = 'import'
              AND status = 'queued'"
    )
    .bind(album_id.to_string())
    .fetch_one(&sqlx::PgPool::connect(&db_url).await?)
    .await?;
    assert_eq!(cnt, 1, "exactly one import job queued");

    println!("✔ album flow OK in {:.1?}", t0.elapsed());
    Ok(())
}

