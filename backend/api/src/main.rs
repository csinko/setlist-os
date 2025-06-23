//! Minimal album-centric façade (v0).

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post, put},
    Json, Router,
};
use shared::pipeline::{JobEnvelope, Stage};
use sqlx::{PgPool, migrate::Migrator};
use uuid::Uuid;
use tracing::{info, instrument};
use anyhow::Result;

#[derive(Clone)]
struct AppState { db: PgPool }

static MIGRATOR: Migrator = sqlx::migrate!("../migrations");

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let db = PgPool::connect(&std::env::var("DATABASE_URL")?).await?;
    MIGRATOR.run(&db).await?;
    let state = AppState { db };

    let app = Router::new()
        .route("/internal/health", get(|| async { "ok" }))
        .route("/albums",               post(create_album))
        .route("/albums/:id",           get(get_album))
        .route("/albums/:id/complete",  put(complete_album))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, app).await?;
    Ok(())
}

/*──────── handlers ───────────────────────────────────────────────────────*/
#[instrument(skip_all)]
async fn create_album(State(app): State<AppState>) -> Result<Json<Uuid>, (StatusCode, String)> {
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO albums(id, source) VALUES ($1, '{\"type\":\"upload\"}')")
        .bind(id)
        .execute(&app.db)
        .await
        .map_err(internal)?;
    Ok(Json(id))
}

async fn get_album(Path(id): Path<Uuid>, State(app): State<AppState>)
    -> Result<String, (StatusCode, String)>
{
    // TODO: materialized view / join with jobs for fancy status
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM jobs WHERE payload->>'album_id' = $1")
        .bind(id.to_string())
        .fetch_one(&app.db)
        .await
        .map_err(internal)?;
    Ok(format!("{} jobs for album", row.0))
}

#[instrument(skip_all)]
async fn complete_album(
    Path(id): Path<Uuid>,
    State(app): State<AppState>,
) -> Result<(), (StatusCode, String)> {
    let env = JobEnvelope { album_id: Some(id), track_id: None, file_id: None, stage: Stage::Import };
    sqlx::query("INSERT INTO jobs(stage, payload) VALUES ($1, $2)")
        .bind(Stage::Import.as_str())
        .bind(serde_json::to_value(&env).unwrap())
        .execute(&app.db)
        .await
        .map_err(internal)?;
    info!("queued import job");
    Ok(())
}

fn internal<E: std::fmt::Display>(e: E) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}

