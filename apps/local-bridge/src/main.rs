use anyhow::{Context, Result};
use axum::{
    extract::State,
    http::{HeaderMap, Request, StatusCode},
    middleware::{self, Next},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use clap::Parser;
use cpal::traits::{DeviceTrait, HostTrait};
use serde::{Deserialize, Serialize};
use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = 9001)]
    port: u16,

    #[arg(long)]
    pair_token: Option<String>,

    #[arg(long)]
    input_file: Option<PathBuf>,

    #[arg(long)]
    reset_pairing: bool,
}

#[derive(Serialize, Deserialize, Clone)]
struct Config {
    pair_token: String,
}

#[derive(Deserialize)]
struct PairRequest {
    token: String,
}

#[derive(Serialize)]
struct StatusResponse {
    running: bool,
    device_name: String,
    sample_rate: u32,
    bph: u32,
    lift_angle: f64,
    session_id: Option<String>,
}

struct AppState {
    pair_token: String,
    running: bool,
    device_name: String,
    sample_rate: u32,
    bph: u32,
    lift_angle: f64,
    session_id: Option<String>,
}

type SharedState = Arc<RwLock<AppState>>;

fn config_dir() -> Result<PathBuf> {
    dirs::home_dir()
        .map(|home| home.join(".config").join("timebridge"))
        .context("$HOME not set; cannot determine config directory")
}

fn config_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("config.json"))
}

fn load_config() -> Result<Option<Config>> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(None);
    }
    let contents = fs::read_to_string(&path).context("failed to read config file")?;
    let config: Config = serde_json::from_str(&contents).context("failed to parse config file")?;
    Ok(Some(config))
}

fn save_config(config: &Config) -> Result<()> {
    let dir = config_dir()?;
    fs::create_dir_all(&dir).context("failed to create config directory")?;
    let path = config_path()?;
    let json = serde_json::to_string_pretty(config).context("failed to serialize config")?;
    fs::write(&path, json).context("failed to write config file")?;
    Ok(())
}

fn persist_token(token: String) -> Result<Config> {
    let config = Config {
        pair_token: token,
    };
    save_config(&config)?;
    Ok(config)
}

fn reset_pairing() -> Result<()> {
    let path = config_path()?;
    if path.exists() {
        fs::remove_file(&path).context("failed to remove config file")?;
    }
    if let Ok(dir) = config_dir() {
        let _ = fs::remove_dir(&dir);
    }
    Ok(())
}

fn list_input_devices() -> Vec<String> {
    let host = cpal::default_host();
    match host.input_devices() {
        Ok(devices) => devices.filter_map(|d| d.name().ok()).collect(),
        Err(_) => vec![],
    }
}

async fn auth(
    State(state): State<SharedState>,
    headers: HeaderMap,
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    let token = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or((StatusCode::UNAUTHORIZED, "missing Authorization header"))?;

    let pair_token = state.read().await.pair_token.clone();
    if token != pair_token {
        return Err((StatusCode::UNAUTHORIZED, "invalid pairing token"));
    }

    Ok(next.run(request).await)
}

async fn pair_handler(
    State(state): State<SharedState>,
    Json(body): Json<PairRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, &'static str)> {
    let token = body.token;
    let config = Config {
        pair_token: token.clone(),
    };
    save_config(&config).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "failed to save pairing token",
        )
    })?;
    state.write().await.pair_token = token.clone();

    Ok(Json(serde_json::json!({ "status": "ok", "token": token })))
}

async fn devices_handler() -> Json<Vec<String>> {
    Json(list_input_devices())
}

async fn status_handler(
    State(state): State<SharedState>,
) -> Json<StatusResponse> {
    let s = state.read().await;
    Json(StatusResponse {
        running: s.running,
        device_name: s.device_name.clone(),
        sample_rate: s.sample_rate,
        bph: s.bph,
        lift_angle: s.lift_angle,
        session_id: s.session_id.clone(),
    })
}

async fn start_handler(
    State(state): State<SharedState>,
) -> Result<Json<serde_json::Value>, (StatusCode, &'static str)> {
    let mut s = state.write().await;
    if s.running {
        return Err((StatusCode::CONFLICT, "session already running"));
    }
    s.running = true;
    s.session_id = Some(Uuid::new_v4().to_string());
    Ok(Json(serde_json::json!({ "status": "started", "session_id": s.session_id.clone() })))
}

async fn stop_handler(
    State(state): State<SharedState>,
) -> Result<Json<serde_json::Value>, (StatusCode, &'static str)> {
    let mut s = state.write().await;
    if !s.running {
        return Err((StatusCode::CONFLICT, "no running session"));
    }
    s.running = false;
    let session_id = s.session_id.take();
    Ok(Json(serde_json::json!({ "status": "stopped", "session_id": session_id })))
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    if args.reset_pairing {
        reset_pairing()?;
        println!("Pairing token reset.");
        return Ok(());
    }

    let config = if let Some(token) = args.pair_token {
        persist_token(token)?
    } else {
        match load_config()? {
            Some(existing) => existing,
            None => persist_token(Uuid::new_v4().to_string())?,
        }
    };

    let state: SharedState = Arc::new(RwLock::new(AppState {
        pair_token: config.pair_token.clone(),
        running: false,
        device_name: String::new(),
        sample_rate: 96000,
        bph: 28800,
        lift_angle: 52.0,
        session_id: None,
    }));

    let public_routes = Router::new().route("/pair", post(pair_handler));

    let protected_routes = Router::new()
        .route("/devices", get(devices_handler))
        .route("/status", get(status_handler))
        .route("/start", post(start_handler))
        .route("/stop", post(stop_handler))
        .layer(middleware::from_fn_with_state(state.clone(), auth));

    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], args.port));
    println!("Pairing token: {}", config.pair_token);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("Listening on http://{}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}