//! MATCH worker – still a stub.

use anyhow::Result;
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("worker_match=debug".parse()?))
        .init();

    tracing::info!("MATCH worker stub online – TODO: implement");
    std::future::pending::<()>().await;
    Ok(())
}

