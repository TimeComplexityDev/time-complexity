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
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use serde::{Deserialize, Serialize};
use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc,
};
use tokio::sync::RwLock;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Audio stream backend — mic (cpal) or file (symphonia)
// ---------------------------------------------------------------------------

struct FilePlayback {
    stop_flag: Arc<AtomicBool>,
    join_handle: Option<std::thread::JoinHandle<()>>,
}

impl Drop for FilePlayback {
    fn drop(&mut self) {
        self.stop_flag.store(true, Ordering::Relaxed);
        if let Some(handle) = self.join_handle.take() {
            let _ = handle.join();
        }
    }
}

#[allow(dead_code)]
enum StreamBackend {
    Mic(cpal::Stream),
    File(FilePlayback),
}

struct SafeStream(Option<StreamBackend>);
unsafe impl Send for SafeStream {}
unsafe impl Sync for SafeStream {}

impl SafeStream {
    fn is_active(&self) -> bool {
        self.0.is_some()
    }
}

struct CaptureSession {
    stream: SafeStream,
    device_name: String,
    sample_count: Arc<AtomicU64>,
    sample_rate: u32,
}

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = 9001)]
    port: u16,

    #[arg(long)]
    pair_token: Option<String>,

    /// Path to an audio file (MP3/WAV) for offline analysis instead of live mic
    #[arg(long)]
    input_file: Option<PathBuf>,

    /// Loop file playback at EOF
    #[arg(long, default_value_t = false)]
    loop_playback: bool,

    #[arg(long)]
    reset_pairing: bool,
}

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone)]
struct Config {
    pair_token: String,
}

#[derive(Deserialize)]
struct PairRequest {
    token: String,
}

#[derive(Serialize, Clone)]
struct DeviceConfig {
    device_name: String,
    sample_rate: u32,
    bph: u32,
    lift_angle: f64,
}

#[derive(Serialize)]
struct StatusResponse {
    running: bool,
    device: DeviceConfig,
    session_id: Option<String>,
    total_samples: u64,
}

struct AppState {
    pair_token: String,
    device: DeviceConfig,
    session_id: Option<String>,
    audio_stream: SafeStream,
    sample_count: Arc<AtomicU64>,
    input_file: Option<PathBuf>,
    loop_playback: bool,
}

impl AppState {
    fn running(&self) -> bool {
        self.audio_stream.is_active()
    }
}

type SharedState = Arc<RwLock<AppState>>;

// ---------------------------------------------------------------------------
// Config helpers
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Device discovery
// ---------------------------------------------------------------------------

fn list_input_devices() -> Vec<String> {
    let host = cpal::default_host();
    match host.input_devices() {
        Ok(devices) => devices.filter_map(|d| d.name().ok()).collect(),
        Err(_) => vec![],
    }
}

// ---------------------------------------------------------------------------
// Mic capture (cpal)
// ---------------------------------------------------------------------------

fn find_input_config(
    device: &cpal::Device,
) -> Result<cpal::SupportedStreamConfig, String> {
    if let Ok(supported) = device.supported_input_configs() {
        for range in supported {
            let max_rate = range.max_sample_rate().0;
            if max_rate >= 96000 && range.sample_format() == cpal::SampleFormat::F32 {
                return Ok(range
                    .with_sample_rate(cpal::SampleRate(96000)));
            }
        }
    }
    device
        .default_input_config()
        .map_err(|e| format!("no default audio input config: {}", e))
}

fn start_mic_capture(device_name: Option<&str>) -> Result<CaptureSession, String> {
    let host = cpal::default_host();
    let device = if let Some(name) = device_name {
        host.input_devices()
            .map_err(|e| format!("failed to enumerate devices: {}", e))?
            .find(|d| d.name().ok().as_deref() == Some(name))
            .ok_or_else(|| format!("device '{}' not found", name))?
    } else {
        host.default_input_device()
            .ok_or_else(|| "no default input device found".to_string())?
    };
    let actual_name = device.name().unwrap_or_else(|_| "unknown".to_string());

    let config = find_input_config(&device)?;
    let sample_rate = config.sample_rate().0;
    let stream_config = config.config();

    println!(
        "Starting mic capture: device='{}', sample_rate={}, channels={}, format={:?}",
        actual_name,
        sample_rate,
        stream_config.channels,
        config.sample_format()
    );

    let sample_count = Arc::new(AtomicU64::new(0));
    let count = sample_count.clone();
    let name = actual_name.clone();

    let err_fn = move |err: cpal::StreamError| {
        eprintln!("mic error on '{}': {}", name, err);
    };

    let data_fn = move |data: &[f32], _: &cpal::InputCallbackInfo| {
        count.fetch_add(data.len() as u64, Ordering::Relaxed);
    };

    let stream = device
        .build_input_stream(&stream_config, data_fn, err_fn, None)
        .map_err(|e| format!("failed to build mic stream: {}", e))?;

    stream
        .play()
        .map_err(|e| format!("failed to start mic stream: {}", e))?;

    println!("Mic capture running: {} @ {} Hz", actual_name, sample_rate);

    Ok(CaptureSession {
        stream: SafeStream(Some(StreamBackend::Mic(stream))),
        device_name: actual_name,
        sample_count,
        sample_rate,
    })
}

// ---------------------------------------------------------------------------
// File capture (symphonia)
// ---------------------------------------------------------------------------

fn start_file_capture(
    path: &std::path::Path,
    loop_playback: bool,
) -> Result<CaptureSession, String> {
    use symphonia::core::codecs::DecoderOptions;
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::probe::Hint;

    let file =
        std::fs::File::open(path).map_err(|e| format!("cannot open '{}': {}", path.display(), e))?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let probed = symphonia::default::get_probe()
        .format(&Hint::new(), mss, &FormatOptions::default(), &MetadataOptions::default())
        .map_err(|e| format!("failed to probe audio file: {}", e))?;

    let mut format = probed.format;

    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
        .ok_or_else(|| "no audio track found".to_string())?;

    let codec_params = track.codec_params.clone();
    let track_id = track.id;

    let mut decoder = symphonia::default::get_codecs()
        .make(&codec_params, &DecoderOptions::default())
        .map_err(|e| format!("failed to create decoder: {}", e))?;

    let sample_rate = codec_params.sample_rate.unwrap_or(48_000);
    let device_name = format!(
        "file: {} ({} Hz, {} ch)",
        path.file_name()
            .map(|s| s.to_string_lossy())
            .unwrap_or(std::borrow::Cow::Borrowed("unknown")),
        sample_rate,
        codec_params.channels.map(|c| c.count()).unwrap_or(0),
    );

    println!("Starting file capture: {} (loop={})", device_name, loop_playback);

    let sample_count = Arc::new(AtomicU64::new(0));
    let count = sample_count.clone();
    let stop_flag = Arc::new(AtomicBool::new(false));
    let flag = stop_flag.clone();
    let path = path.to_path_buf();

    let join_handle = std::thread::spawn(move || {
        let _ = play_file(
            &path,
            loop_playback,
            &mut format,
            &mut decoder,
            track_id,
            &count,
            &flag,
        );
    });

    let handle = FilePlayback {
        stop_flag,
        join_handle: Some(join_handle),
    };

    Ok(CaptureSession {
        stream: SafeStream(Some(StreamBackend::File(handle))),
        device_name,
        sample_count,
        sample_rate,
    })
}

fn play_file(
    path: &std::path::Path,
    loop_playback: bool,
    format: &mut Box<dyn symphonia::core::formats::FormatReader>,
    decoder: &mut Box<dyn symphonia::core::codecs::Decoder>,
    track_id: u32,
    sample_count: &AtomicU64,
    stop_flag: &AtomicBool,
) -> Result<(), String> {

    loop {
        loop {
            if stop_flag.load(Ordering::Relaxed) {
                return Ok(());
            }

            match format.next_packet() {
                Ok(packet) => {
                    if packet.track_id() != track_id {
                        continue;
                    }

                    if let Ok(decoded) = decoder.decode(&packet) {
                        let frames = decoded.frames() as u64;
                        let channels = decoded.spec().channels.count() as u64;
                        sample_count.fetch_add(frames * channels, Ordering::Relaxed);
                    }
                }
                Err(symphonia::core::errors::Error::DecodeError(_)) => continue,
                Err(symphonia::core::errors::Error::IoError(ref e))
                    if e.kind() == std::io::ErrorKind::UnexpectedEof =>
                {
                    break;
                }
                Err(e) => {
                    eprintln!("file decode error: {}", e);
                    break;
                }
            }
        }

        if !loop_playback {
            break;
        }

        // Re-open file for next loop
        let file = match std::fs::File::open(path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("file error on re-open: {}", e);
                break;
            }
        };
        let mss = symphonia::core::io::MediaSourceStream::new(Box::new(file), Default::default());
        let probed = match symphonia::default::get_probe().format(
            &symphonia::core::probe::Hint::new(),
            mss,
            &symphonia::core::formats::FormatOptions::default(),
            &symphonia::core::meta::MetadataOptions::default(),
        ) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("file re-probe error: {}", e);
                break;
            }
        };
        *format = probed.format;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// HTTP handlers
// ---------------------------------------------------------------------------

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
        running: s.running(),
        device: s.device.clone(),
        session_id: s.session_id.clone(),
        total_samples: s.sample_count.load(Ordering::Relaxed),
    })
}

async fn start_handler(
    State(state): State<SharedState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    {
        let s = state.read().await;
        if s.running() {
            return Err((StatusCode::CONFLICT, "session already running".to_string()));
        }
    }

    let capture = {
        let s = state.read().await;
        if let Some(path) = &s.input_file {
            start_file_capture(path, s.loop_playback)
        } else {
            start_mic_capture(None)
        }
    };

    let mut s = state.write().await;
    match capture {
        Ok(session) => {
            s.session_id = Some(Uuid::new_v4().to_string());
            s.device.device_name = session.device_name;
            s.device.sample_rate = session.sample_rate;
            s.audio_stream = session.stream;
            s.sample_count = session.sample_count;
            Ok(Json(serde_json::json!({
                "status": "started",
                "session_id": s.session_id.clone()
            })))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to start capture: {}", e),
        )),
    }
}

async fn stop_handler(
    State(state): State<SharedState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut s = state.write().await;
    if !s.running() {
        return Err((StatusCode::CONFLICT, "no running session".to_string()));
    }
    let session_id = s.session_id.take();
    s.audio_stream = SafeStream(None);
    let total = s.sample_count.load(Ordering::Relaxed);
    println!("Session stopped: {} total samples processed", total);
    Ok(Json(serde_json::json!({ "status": "stopped", "session_id": session_id })))
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

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
        device: DeviceConfig {
            device_name: String::new(),
            sample_rate: 96000,
            bph: 28800,
            lift_angle: 52.0,
        },
        session_id: None,
        audio_stream: SafeStream(None),
        sample_count: Arc::new(AtomicU64::new(0)),
        input_file: args.input_file.clone(),
        loop_playback: args.loop_playback,
    }));

    // Auto-start file playback if --input-file was provided
    if let Some(path) = &args.input_file {
        println!("File mode active: {}", path.display());
        let capture = start_file_capture(path, args.loop_playback);
        match capture {
            Ok(session) => {
                let mut s = state.write().await;
                s.session_id = Some(Uuid::new_v4().to_string());
                s.device.device_name = session.device_name;
                s.device.sample_rate = session.sample_rate;
                s.audio_stream = session.stream;
                s.sample_count = session.sample_count;
                println!("File playback started automatically (--input-file mode)");
            }
            Err(e) => {
                eprintln!("Failed to start file capture: {}", e);
            }
        }
    }

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