//! INDEX worker – still a stub.

use anyhow::Result;
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("worker_index=debug".parse()?))
        .init();

    tracing::info!("INDEX worker stub online – TODO: implement");
    std::future::pending::<()>().await;
    Ok(())
}

