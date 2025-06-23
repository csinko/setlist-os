//! Fingerprint worker
//! Consumes queue.fingerprint → runs fpcalc → marks FP_DONE
//! then enqueues metadata job.

use anyhow::Result;
use futures_util::StreamExt;
use lapin::{
    options::{
        BasicAckOptions, BasicConsumeOptions, BasicPublishOptions, BasicQosOptions,
        ExchangeDeclareOptions, QueueBindOptions, QueueDeclareOptions,
    },
    types::FieldTable,
    BasicProperties, Channel, Connection, ConnectionProperties,
};
use shared::pipeline::{JobEnvelope, Stage};
use sqlx::PgPool;
use std::process::Command;
use tracing::{debug, error, info, instrument, span, Level, Span, Instrument};
use tracing_subscriber::{fmt, EnvFilter};
use std::default::Default;

/*────────────────────────────────────────────────────────────────────────────*/

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("worker=debug".parse()?))
        .init();

    /*── DB pool ───────────────────────────────────────────────────────────*/
    let db_url = std::env::var("DATABASE_URL")?;
    debug!(%db_url, "connecting to Postgres");
    let db = PgPool::connect(&db_url).await?;
    info!("Postgres connection ready");

    /*── AMQP setup ────────────────────────────────────────────────────────*/
    let amqp_url = std::env::var("AMQP_URL")?;
    debug!(%amqp_url, "connecting to RabbitMQ");
    let conn = Connection::connect(&amqp_url, ConnectionProperties::default()).await?;
    let channel = conn.create_channel().await?;
    info!("AMQP channel open");

    channel.basic_qos(4, BasicQosOptions::default()).await?;
    debug!("QoS set to prefetch=4");

    // exchange + queues
    channel
        .exchange_declare(
            "jobs",
            lapin::ExchangeKind::Direct,
            ExchangeDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;
    debug!("exchange=jobs declared");
    declare_queue(&channel, "queue.fingerprint", "fingerprint").await?;
    declare_queue(&channel, "queue.metadata", "metadata").await?;

    /*── start consumer loop ───────────────────────────────────────────────*/
    let mut consumer = channel
        .basic_consume(
            "queue.fingerprint",
            "worker-fp",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;
    info!("fingerprint worker online – waiting for jobs…");

    while let Some(delivery) = consumer.next().await {
        let delivery = delivery?;
        let span = span!(Level::INFO, "job", delivery_tag = delivery.delivery_tag);
        if let Err(e) = handle_job(&channel, &db, &delivery.data).instrument(span).await {
            error!("job failed: {e:#}");
        }
        delivery.ack(BasicAckOptions::default()).await?;
    }
    Ok(())
}

/*────────────────────────────────────────────────────────────────────────────*/

/// Handle a single fingerprint job
#[instrument(skip_all, level = "debug")]
async fn handle_job(channel: &Channel, db: &PgPool, payload: &[u8]) -> Result<()> {
    let env: JobEnvelope = serde_json::from_slice(payload)?;
    if let Some(fid) = env.file_id {
        Span::current().record("file_id", &tracing::field::display(fid));
    }
    info!(stage = ?env.stage, "received job");

    /*── fetch file path ───────────────────────────────────────────────────*/
    let (path,): (String,) =
        sqlx::query_as("SELECT path FROM files WHERE id=$1")
            .bind(env.file_id.expect("file_id required"))
            .fetch_one(db)
            .await?;
    debug!(%path, "file path resolved");

    /*── run fpcalc ────────────────────────────────────────────────────────*/
    let fp = run_fpcalc(&path)?;
    debug!(dur = fp.duration, "fpcalc OK");

    /*── update DB ─────────────────────────────────────────────────────────*/
    sqlx::query(
        r#"
          UPDATE files
             SET status='FP_DONE',
                 duration_sec=$1,
                 updated_at=now()
           WHERE id=$2
        "#,
    )
    .bind(fp.duration)
    .bind(env.file_id.expect("file_id required"))
    .execute(db)
    .await?;
    info!("DB updated to FP_DONE");

    /*── enqueue next stage ───────────────────────────────────────────────*/
    let next = JobEnvelope {
        album_id: None,
        track_id: None,
        file_id: env.file_id,
        stage: Stage::Match,
    };
    channel
        .basic_publish(
            "jobs",
            "metadata",
            BasicPublishOptions::default(),
            &serde_json::to_vec(&next)?,
            BasicProperties::default(),
        )
        .await?
        .await?;
    info!("metadata job published");

    Ok(())
}

/*────────────────────────────────────────────────────────────────────────────*/

async fn declare_queue(channel: &Channel, name: &str, rk: &str) -> Result<()> {
    channel
        .queue_declare(name, QueueDeclareOptions{ durable: true, .. Default::default()}, FieldTable::default())
        .await?;
    channel
        .queue_bind(
            name,
            "jobs",
            rk,
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await?;
    debug!(queue = name, routing_key = rk, "queue declared & bound");
    Ok(())
}

/*────────────────────────────────────────────────────────────────────────────*/
// Light fpcalc wrapper; refine later
#[derive(Debug)]
struct FingerPrint {
    duration: i32,
    fingerprint: String,
}

#[instrument(level = "debug", ret)]
fn run_fpcalc(path: &str) -> Result<FingerPrint> {
    debug!(%path, "invoking fpcalc");
    let out = Command::new("fpcalc").arg("-json").arg(path).output()?;
    if !out.status.success() {
        anyhow::bail!("fpcalc failed: {}", out.status);
    }

    #[derive(serde::Deserialize)]
    struct Raw {
        duration: f32,
        fingerprint: String,
    }
    let raw: Raw = serde_json::from_slice(&out.stdout)?;
    Ok(FingerPrint {
        duration: raw.duration.round() as i32,
        fingerprint: raw.fingerprint,
    })
}

