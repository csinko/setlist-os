//! Simple poll-loop helper: wait until `GET <url>` returns 2xx or timeout.

use anyhow::Result;
use reqwest::Client;
use std::time::{Duration, Instant};
use tokio::time::sleep;

pub async fn wait_for_http_ok(url: &str, timeout: Duration) -> Result<()> {
    let client  = Client::new();
    let start   = Instant::now();
    loop {
        match client.get(url).send().await {
            Ok(r) if r.status().is_success() => return Ok(()),
            _ => {
                if start.elapsed() > timeout {
                    anyhow::bail!("timed-out waiting for {url}");
                }
                sleep(Duration::from_millis(150)).await;
            }
        }
    }
}

