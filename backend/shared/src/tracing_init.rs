use tracing_subscriber::{fmt, EnvFilter};

pub fn init(service: &str) {
    // Default = info; allow `RUST_LOG` override
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));
    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_thread_ids(true)
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
        .json()                      // ← machine-readable out-of-the-box
        .init();
    tracing::info!(service, "tracing initialised");
}

