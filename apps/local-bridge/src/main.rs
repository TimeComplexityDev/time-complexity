use anyhow::{Context, Result};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    http::{HeaderMap, Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
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
    Arc, Mutex,
};
use std::time::Duration;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use uuid::Uuid;

mod dsp;
mod metrics;

// ---------------------------------------------------------------------------
// Application error type
// ---------------------------------------------------------------------------

#[derive(Debug)]
enum AppError {
    BadRequest(&'static str),
    Conflict(&'static str),
    Unauthorized,
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::BadRequest(m) => (StatusCode::BAD_REQUEST, m).into_response(),
            AppError::Conflict(m) => (StatusCode::CONFLICT, m).into_response(),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized").into_response(),
            AppError::Internal(m) => (StatusCode::INTERNAL_SERVER_ERROR, m).into_response(),
        }
    }
}

impl From<String> for AppError {
    fn from(s: String) -> Self {
        AppError::Internal(s)
    }
}

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

// cpal 0.15 marks Stream as !Send and !Sync via NotSendSyncAcrossAllPlatforms
// for portability safety. On macOS/CoreAudio the stream handle is backed by
// an AudioUnit which is fully thread-safe (Send + Sync). This wrapper is the
// standard workaround for single-platform macOS Rust audio apps.
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

#[derive(Deserialize)]
struct SetParamsRequest {
    bph: Option<u32>,
    bandpass_freq: Option<f64>,
    bandpass_q: Option<f64>,
}

#[derive(Deserialize)]
struct StreamQuery {
    token: Option<String>,
}

// ---------------------------------------------------------------------------
// Start request — discriminated union via serde(untagged)
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct MicConfig {
    #[serde(default)]
    device_name: Option<String>,
}

#[derive(Deserialize)]
struct FileConfig {
    path: String,
    #[serde(default)]
    loop_playback: bool,
}

#[derive(Deserialize)]
struct SimulatorConfig {
    #[serde(default = "default_bph")]
    bph: u16,
    #[serde(default)]
    drift_s_per_day: f64,
    #[serde(default)]
    beat_error_ms: f64,
}

fn default_bph() -> u16 { 21600 }

#[derive(Deserialize)]
#[serde(untagged)]
enum StartRequest {
    Mic { mic: MicConfig },
    File { file: FileConfig },
    Simulator { simulator: SimulatorConfig },
}

#[derive(Serialize, Clone)]
struct DeviceConfig {
    device_name: String,
    sample_rate: u32,
    lift_angle: f64,
}

#[derive(Serialize)]
struct StatusResponse {
    running: bool,
    device: DeviceConfig,
    session_id: Option<String>,
    total_samples: u64,
    bph: u32,
}

struct SessionState {
    session_id: Option<String>,
    audio_stream: SafeStream,
    sample_count: Arc<AtomicU64>,
}

impl SessionState {
    fn running(&self) -> bool {
        self.audio_stream.is_active()
    }
}

struct AppState {
    pair_token: String,
    device: DeviceConfig,
    session: SessionState,
    pipeline: Arc<Mutex<dsp::DspPipeline>>,
    bph_override: Option<u32>,
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

fn start_mic_capture(
    device_name: Option<&String>,
    pipeline: &Arc<Mutex<dsp::DspPipeline>>,
) -> Result<CaptureSession, String> {
    let host = cpal::default_host();

    let device = if let Some(name) = device_name {
        if name.is_empty() {
            host.default_input_device()
                .ok_or_else(|| "no default input device found".to_string())?
        } else {
            host.input_devices()
                .map_err(|e| format!("failed to enumerate devices: {}", e))?
                .find(|d| d.name().map(|n| n == *name).unwrap_or(false))
                .ok_or_else(|| format!("device '{}' not found", name))?
        }
    } else {
        host.default_input_device()
            .ok_or_else(|| "no default input device found".to_string())?
    };
    let actual_name = device.name().unwrap_or_else(|_| "unknown".to_string());

    let config = find_input_config(&device)?;
    let sample_rate = config.sample_rate().0;
    let stream_config = config.config();
    let channels = stream_config.channels as usize;

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

    let pipe = pipeline.clone();
    let data_fn = move |data: &[f32], _: &cpal::InputCallbackInfo| {
        count.fetch_add(data.len() as u64, Ordering::Relaxed);
        let samples = if channels > 1 {
            let mono: Vec<f32> = data
                .chunks(channels)
                .map(|chunk| chunk.iter().sum::<f32>() / channels as f32)
                .collect();
            mono
        } else {
            data.to_vec()
        };
        if let Ok(mut p) = pipe.lock() {
            p.process_samples(&samples);
        }
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
    pipeline: &Arc<Mutex<dsp::DspPipeline>>,
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
    let pipe = pipeline.clone();

    let join_handle = std::thread::spawn(move || {
        let _ = play_file(
            &path,
            loop_playback,
            &mut format,
            &mut decoder,
            track_id,
            &count,
            &flag,
            &pipe,
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
    pipeline: &Arc<Mutex<dsp::DspPipeline>>,
) -> Result<(), String> {
    use symphonia::core::audio::SampleBuffer;

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

                        let spec = *decoded.spec();
                        let mut buf = SampleBuffer::<f32>::new(frames, spec);
                        buf.copy_interleaved_ref(decoded);
                        let samples = buf.samples();

                        if channels > 1 {
                            let mono: Vec<f32> = samples
                                .chunks(channels as usize)
                                .map(|chunk| chunk.iter().sum::<f32>() / channels as f32)
                                .collect();
                            if let Ok(mut p) = pipeline.lock() {
                                p.process_samples(&mono);
                            }
                        } else if let Ok(mut p) = pipeline.lock() {
                            p.process_samples(samples);
                        }
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
// Simulator capture
// ---------------------------------------------------------------------------

fn start_simulator_capture(
    params: SimulatorConfig,
    pipeline: &Arc<Mutex<dsp::DspPipeline>>,
) -> Result<CaptureSession, String> {
    let sample_rate = 44100;
    let device_name = format!(
        "simulator: {} BPH, drift {drift:+.1} s/d, beat error {be} ms",
        params.bph,
        drift = params.drift_s_per_day,
        be = params.beat_error_ms,
    );

    println!("Starting simulator capture: {}", device_name);

    let sample_count = Arc::new(AtomicU64::new(0));
    let count = sample_count.clone();
    let stop_flag = Arc::new(AtomicBool::new(false));
    let flag = stop_flag.clone();
    let pipe = pipeline.clone();
    let chunk_size = 4096;
    let sleep_dur = Duration::from_millis((chunk_size as u64 * 1000) / sample_rate as u64);

    let join_handle = std::thread::spawn(move || {
        let mut simulator = dsp::TickSimulator::new(
            dsp::SimulatorParams {
                bph: params.bph,
                drift_s_per_day: params.drift_s_per_day,
                beat_error_ms: params.beat_error_ms,
            },
            sample_rate,
        );
        let mut buf = vec![0.0_f32; chunk_size];

        loop {
            if flag.load(Ordering::Relaxed) {
                break;
            }
            let written = simulator.generate_samples(&mut buf);
            if written > 0 {
                count.fetch_add(written as u64, Ordering::Relaxed);
                if let Ok(mut p) = pipe.lock() {
                    p.process_samples(&buf[..written]);
                }
            }
            std::thread::sleep(sleep_dur);
        }
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

// ---------------------------------------------------------------------------
// HTTP handlers
// ---------------------------------------------------------------------------

async fn auth(
    State(state): State<SharedState>,
    headers: HeaderMap,
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<impl IntoResponse, AppError> {
    let token = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(AppError::BadRequest("missing Authorization header"))?;

    let pair_token = state.read().await.pair_token.clone();
    if token != pair_token {
        return Err(AppError::Unauthorized);
    }

    Ok(next.run(request).await)
}

async fn pair_handler(
    State(state): State<SharedState>,
    Json(body): Json<PairRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let current = state.read().await.pair_token.clone();
    if body.token != current {
        return Err(AppError::Unauthorized);
    }
    // Token matches — confirm pairing
    let config = Config {
        pair_token: current.clone(),
    };
    save_config(&config).map_err(|_| AppError::Internal("failed to save pairing config".into()))?;

    Ok(Json(serde_json::json!({ "status": "ok", "token": current })))
}

async fn devices_handler() -> Json<Vec<String>> {
    Json(list_input_devices())
}

fn effective_bph(state: &AppState) -> u32 {
    state
        .bph_override
        .or_else(|| state.pipeline.lock().ok().map(|p| p.detected_bph))
        .unwrap_or(dsp::DEFAULT_BPH)
}

async fn status_handler(
    State(state): State<SharedState>,
) -> Json<StatusResponse> {
    let s = state.read().await;
    Json(StatusResponse {
        running: s.session.running(),
        device: s.device.clone(),
        session_id: s.session.session_id.clone(),
        total_samples: s.session.sample_count.load(Ordering::Relaxed),
        bph: effective_bph(&s),
    })
}

async fn start_handler(
    State(state): State<SharedState>,
    body: Option<Json<StartRequest>>,
) -> Result<Json<serde_json::Value>, AppError> {
    {
        let s = state.read().await;
        if s.session.running() {
            return Err(AppError::Conflict("session already running"));
        }
    }

    let capture = {
        let s = state.read().await;
        let pipe = s.pipeline.clone();

        match body.map(|Json(b)| b) {
            Some(StartRequest::Mic { mic }) => {
                start_mic_capture(mic.device_name.as_ref(), &pipe).map_err(AppError::Internal)
            }
            Some(StartRequest::File { file }) => {
                start_file_capture(std::path::Path::new(&file.path), file.loop_playback, &pipe)
                    .map_err(AppError::Internal)
            }
            Some(StartRequest::Simulator { simulator }) => {
                start_simulator_capture(simulator, &pipe).map_err(AppError::Internal)
            }
            None => {
                start_mic_capture(None, &pipe).map_err(AppError::Internal)
            }
        }
    };

    match capture {
        Ok(session) => {
            let mut s = state.write().await;
            s.pipeline.lock().unwrap().set_sample_rate(session.sample_rate as f64);
            s.session.session_id = Some(Uuid::new_v4().to_string());
            s.device.device_name = session.device_name;
            s.device.sample_rate = session.sample_rate;
            s.session.audio_stream = session.stream;
            s.session.sample_count = session.sample_count;
            Ok(Json(serde_json::json!({
                "status": "started",
                "session_id": s.session.session_id.clone()
            })))
        }
        Err(e) => Err(e),
    }
}

async fn stop_handler(
    State(state): State<SharedState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut s = state.write().await;
    if !s.session.running() {
        return Err(AppError::Conflict("no running session"));
    }
    let session_id = s.session.session_id.take();
    s.session.audio_stream = SafeStream(None);
    let total = s.session.sample_count.load(Ordering::Relaxed);
    println!("Session stopped: {} total samples processed", total);
    Ok(Json(serde_json::json!({ "status": "stopped", "session_id": session_id })))
}

// ---------------------------------------------------------------------------
// set_params handler
// ---------------------------------------------------------------------------

async fn set_params_handler(
    State(state): State<SharedState>,
    Json(body): Json<SetParamsRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut s = state.write().await;
    if let Some(bph) = body.bph {
        s.bph_override = Some(bph);
    }
    if let Some(freq) = body.bandpass_freq {
        let q = body.bandpass_q.unwrap_or(0.4);
        s.pipeline.lock().unwrap().set_bandpass(freq, q);
    }
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

// ---------------------------------------------------------------------------
// WebSocket stream
// ---------------------------------------------------------------------------

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<SharedState>,
    Query(query): Query<StreamQuery>,
) -> Result<impl IntoResponse, AppError> {
    let expected_token = state.read().await.pair_token.clone();
    let provided = query.token.as_deref().unwrap_or("");
    if provided != expected_token {
        return Err(AppError::Unauthorized);
    }
    Ok(ws.on_upgrade(move |socket| handle_socket(socket, state)))
}

async fn handle_socket(mut socket: WebSocket, state: SharedState) {
    let (sample_rate, session_id) = {
        let s = state.read().await;
        (s.device.sample_rate as f64, s.session.session_id.clone().unwrap_or_default())
    };

    let mut metrics = metrics::MetricsEngine::new(session_id, sample_rate);

    loop {
        tokio::select! {
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(cmd) = serde_json::from_str::<serde_json::Value>(&text) {
                            if let Some(bph) = cmd.get("bph").and_then(|v| v.as_u64()) {
                                let mut s = state.write().await;
                                s.bph_override = Some(bph as u32);
                                metrics.set_bph(bph as u32);
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
            _ = tokio::time::sleep(Duration::from_millis(50)) => {
                let (new_ticks, bph_detected, detected_bph, bph_override) = {
                    let guard = state.read().await;
                    let p = guard.pipeline.lock().unwrap();
                    (p.ticks.clone(), p.bph_detected, p.detected_bph, guard.bph_override)
                };

                let bph = bph_override.unwrap_or(detected_bph);
                if bph != metrics.bph() {
                    metrics.set_bph(bph);
                }

                if bph_detected || bph_override.is_some() {
                    metrics.ingest_ticks(&new_ticks);
                }
                let (tick_msgs, aggregate) = metrics.drain_messages();

                for msg in tick_msgs {
                    if let Ok(json) = serde_json::to_string(&msg) {
                        if socket.send(Message::Text(json.into())).await.is_err() {
                            break;
                        }
                    }
                }

                if let Some(agg) = aggregate {
                    if let Ok(json) = serde_json::to_string(&agg) {
                        if socket.send(Message::Text(json.into())).await.is_err() {
                            break;
                        }
                    }
                }
            }
        }
    }
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
            lift_angle: 52.0,
        },
        session: SessionState {
            session_id: None,
            audio_stream: SafeStream(None),
            sample_count: Arc::new(AtomicU64::new(0)),
        },
        pipeline: Arc::new(Mutex::new(dsp::DspPipeline::new(44100.0))),
        bph_override: None,
    }));

    let public_routes = Router::new()
        .route("/pair", post(pair_handler))
        .route("/stream", get(ws_handler));

    let protected_routes = Router::new()
        .route("/devices", get(devices_handler))
        .route("/status", get(status_handler))
        .route("/start", post(start_handler))
        .route("/stop", post(stop_handler))
        .route("/set_params", post(set_params_handler))
        .layer(middleware::from_fn_with_state(state.clone(), auth));

    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .with_state(state)
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([127, 0, 0, 1], args.port));
    println!("Pairing token: {}", config.pair_token);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("Listening on http://{}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}