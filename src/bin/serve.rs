//! Web server for the `download` CLI.
//!
//! Serves the neo-brutalist HTMX interface from `web/` and exposes fragment
//! endpoints that read and write the same `config.json` the CLI uses. The Run
//! button launches the `download` binary itself (`download -e`) as a child
//! process, and `output.log` is tailed and streamed to the browser over SSE.
//!
//! Run from the project root:
//!   cargo build --bins
//!   cargo run --bin serve

use axum::{
    body::Bytes,
    extract::{Form, Path, State},
    http::{header, StatusCode},
    response::{
        sse::{Event, KeepAlive, Sse},
        Html, IntoResponse, Json, Response,
    },
    routing::{delete, get, post},
    Router,
};
use download::config::{Config, AUDIO_FORMATS, VIDEO_FORMATS};
use std::{
    collections::BTreeMap,
    convert::Infallible,
    fs,
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::broadcast;
use tokio_stream::{wrappers::BroadcastStream, StreamExt};

const STATUS_IDLE: &str = "<span class=\"status-pill\"><span class=\"dot\"></span>idle</span>";
const STATUS_RUNNING: &str =
    "<span class=\"status-pill status-live\"><span class=\"dot\"></span>running</span>";

struct AppState {
    child: Mutex<Option<Child>>,
    log_tx: broadcast::Sender<String>,
    progress: Mutex<std::collections::HashMap<String, f32>>,
    current_url: Mutex<Option<String>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (log_tx, _) = broadcast::channel(256);
    let state = Arc::new(AppState {
        child: Mutex::new(None),
        log_tx,
        progress: Mutex::new(std::collections::HashMap::new()),
        current_url: Mutex::new(None),
    });

    spawn_log_tail(state.clone());

    let app = Router::new()
        .route("/", get(index))
        .route("/style.css", get(stylesheet))
        .route("/pages/{name}", get(page))
        .route("/api/queue", get(queue).post(queue_add))
        .route("/api/queue/{index}", delete(queue_remove))
        .route("/api/settings/audio-export", post(set_audio_export))
        .route("/api/settings/thumbnails", post(set_thumbnails))
        .route("/api/settings/video-format", post(set_video_format))
        .route("/api/settings/audio-format", post(set_audio_format))
        .route("/api/settings/dest", post(set_dest))
        .route("/api/status", get(status))
        .route("/api/run", post(run))
        .route("/api/stop", post(stop))
        .route("/api/logs/stream", get(log_stream))
        .route("/api/logs/clear", post(clear_logs))
        .route("/api/debug/progress", get(debug_progress))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    println!("download UI -> http://127.0.0.1:3000");
    axum::serve(listener, app).await?;
    Ok(())
}

// ---------- static assets & page fragments ----------

async fn index() -> Html<&'static str> {
    Html(include_str!("../../web/index.html"))
}

async fn stylesheet() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/css")],
        include_str!("../../web/style.css"),
    )
}

async fn page(Path(name): Path<String>) -> Response {
    let html = match name.as_str() {
        "library" => render_library(),
        "settings" => render_settings(),
        "logs" => Some(include_str!("../../web/pages/logs.html").to_string()),
        "help" => Some(include_str!("../../web/pages/help.html").to_string()),
        _ => None,
    };

    match html {
        Some(html) => Html(html).into_response(),
        None => (StatusCode::NOT_FOUND, "unknown page").into_response(),
    }
}

fn render_library() -> Option<String> {
    let config = Config::load().ok()?;
    Some(
        include_str!("../../web/pages/library.html")
            .replace("{{DEST_VALUE}}", &html_escape(&config.download_dest)),
    )
}

fn render_settings() -> Option<String> {
    let config = Config::load().ok()?;
    let audio_checked = if config.audio_export { "checked" } else { "" };
    let thumb_checked = if config.thumbnail_export {
        "checked"
    } else {
        ""
    };
    Some(
        include_str!("../../web/pages/settings.html")
            .replace("{{AUDIO_CHECKED}}", audio_checked)
            .replace("{{THUMB_CHECKED}}", thumb_checked)
            .replace(
                "{{VIDEO_OPTIONS}}",
                &options(&VIDEO_FORMATS, &config.video_format),
            )
            .replace(
                "{{AUDIO_OPTIONS}}",
                &options(&AUDIO_FORMATS, &config.audio_format),
            ),
    )
}

fn options(formats: &[&str], current: &str) -> String {
    formats
        .iter()
        .map(|format| {
            let selected = if *format == current { " selected" } else { "" };
            format!("<option value=\"{format}\"{selected}>{format}</option>")
        })
        .collect()
}

// ---------- queue ----------

async fn queue(State(state): State<Arc<AppState>>) -> Response {
    match Config::load() {
        Ok(config) => {
            let progress = state.progress.lock().unwrap().clone();
            Html(render_queue_with_progress(&config, &progress)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{e}")).into_response(),
    }
}

async fn queue_add(
    State(state): State<Arc<AppState>>,
    Form(form): Form<BTreeMap<String, String>>,
) -> Response {
    let Some(url) = form.get("url").map(|u| u.trim()).filter(|u| !u.is_empty()) else {
        return (StatusCode::UNPROCESSABLE_ENTITY, "missing url").into_response();
    };

    match Config::load() {
        Ok(mut config) => {
            config.videos.push(url.to_string());
            if let Err(e) = config.save() {
                return (StatusCode::INTERNAL_SERVER_ERROR, format!("{e}")).into_response();
            }
            // initialise progress entry for the new URL
            {
                let mut prog = state.progress.lock().unwrap();
                prog.insert(url.to_string(), 0.0);
            }
            let progress = state.progress.lock().unwrap().clone();
            Html(render_queue_with_progress(&config, &progress)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{e}")).into_response(),
    }
}

async fn queue_remove(State(state): State<Arc<AppState>>, Path(index): Path<usize>) -> Response {
    match Config::load() {
        Ok(mut config) => {
            if index < config.videos.len() {
                let removed_url = config.videos.remove(index);
                // clean up progress entry
                let mut prog = state.progress.lock().unwrap();
                prog.remove(&removed_url);
                drop(prog);
                if let Err(e) = config.save() {
                    return (StatusCode::INTERNAL_SERVER_ERROR, format!("{e}")).into_response();
                }
            }
            let progress = state.progress.lock().unwrap().clone();
            Html(render_queue_with_progress(&config, &progress)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{e}")).into_response(),
    }
}

fn render_queue_with_progress(
    config: &Config,
    progress: &std::collections::HashMap<String, f32>,
) -> String {
    let count = config.videos.len();
    let plural = if count == 1 { "" } else { "s" };
    let mut html = format!("<div class=\"queue-meta\">{count} video{plural} queued</div>");

    if config.videos.is_empty() {
        html.push_str(
            "<div class=\"queue-empty\">Queue is empty — paste a URL above to get started.</div>",
        );
        return html;
    }

    html.push_str("<ul class=\"queue-list\">");
    for (i, url) in config.videos.iter().enumerate() {
        let url_escaped = html_escape(url);
        let pct = progress.get(url).copied().unwrap_or(0.0);
        html.push_str(&format!("<li class=\"queue-item\">\n",));
        html.push_str(&format!(
            "<span class=\"queue-url\" title=\"{}\">{}</span>\n",
            url_escaped, url_escaped
        ));
        html.push_str(&format!(
            "<button class=\"item-remove\" hx-delete=\"/api/queue/{}\" hx-target=\"#queue-wrap\" hx-swap=\"innerHTML\" title=\"Remove\">✕</button>\n",
            i
        ));
        html.push_str(&format!(
            "<div class=\"queue-progress\"><div class=\"queue-progress-fill\" style=\"width:{}%\"></div><span class=\"queue-progress-label\">{:.0}%</span></div></li>\n",
            pct, pct
        ));
    }
    html.push_str("</ul>");
    html
}

// ---------- settings ----------

async fn set_audio_export(body: Bytes) -> StatusCode {
    toggle_setting("audio_export", form_has(&body, "audio_export"))
}

async fn set_thumbnails(body: Bytes) -> StatusCode {
    toggle_setting("thumbnail_export", form_has(&body, "thumbnail_export"))
}

/// An HTML checkbox only submits a value when checked, and an unchecked box may
/// arrive as an empty body without a form content type — so "missing" means off.
fn form_has(body: &Bytes, key: &str) -> bool {
    serde_urlencoded::from_bytes::<BTreeMap<String, String>>(body)
        .map(|values| values.contains_key(key))
        .unwrap_or(false)
}

fn toggle_setting(key: &str, enabled: bool) -> StatusCode {
    match Config::load() {
        Ok(mut config) => {
            match key {
                "audio_export" => config.audio_export = enabled,
                "thumbnail_export" => config.thumbnail_export = enabled,
                _ => {}
            }
            match config.save() {
                Ok(()) => StatusCode::OK,
                Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
            }
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

async fn set_video_format(Form(form): Form<BTreeMap<String, String>>) -> StatusCode {
    set_format("video_format", &VIDEO_FORMATS, &form)
}

async fn set_audio_format(Form(form): Form<BTreeMap<String, String>>) -> StatusCode {
    set_format("audio_format", &AUDIO_FORMATS, &form)
}

fn set_format(key: &str, allowed: &[&str], form: &BTreeMap<String, String>) -> StatusCode {
    let Some(value) = form.get(key) else {
        return StatusCode::UNPROCESSABLE_ENTITY;
    };
    if !allowed.contains(&value.as_str()) {
        return StatusCode::UNPROCESSABLE_ENTITY;
    }
    match Config::load() {
        Ok(mut config) => {
            match key {
                "video_format" => config.video_format = value.clone(),
                "audio_format" => config.audio_format = value.clone(),
                _ => {}
            }
            match config.save() {
                Ok(()) => StatusCode::OK,
                Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
            }
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

async fn set_dest(Form(form): Form<BTreeMap<String, String>>) -> Html<String> {
    let Some(dest) = form
        .get("download_dest")
        .map(|d| d.trim())
        .filter(|d| !d.is_empty())
    else {
        return Html("<span class=\"note-error\">✕ path cannot be empty</span>".to_string());
    };
    if fs::metadata(dest).is_err() {
        return Html(format!(
            "<span class=\"note-error\">✕ path does not exist: {}</span>",
            html_escape(dest)
        ));
    }
    match Config::load().and_then(|mut config| {
        config.download_dest = dest.to_string();
        config.save()
    }) {
        Ok(()) => Html("<span class=\"note-ok\">✓ saved</span>".to_string()),
        Err(e) => Html(format!(
            "<span class=\"note-error\">✕ {}</span>",
            html_escape(&e.to_string())
        )),
    }
}

// ---------- runner ----------

async fn status(State(state): State<Arc<AppState>>) -> Html<&'static str> {
    Html(if is_running(&state) {
        STATUS_RUNNING
    } else {
        STATUS_IDLE
    })
}

fn is_running(state: &AppState) -> bool {
    let mut guard = state.child.lock().unwrap();
    guard
        .as_mut()
        .map(|child| matches!(child.try_wait(), Ok(None)))
        .unwrap_or(false)
}

async fn run(State(state): State<Arc<AppState>>) -> Response {
    if is_running(&state) {
        return StatusCode::OK.into_response();
    }

    let Some(exe) = cli_exe() else {
        let _ = state
            .log_tx
            .send("── error: download binary not found ──".to_string());
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Html("⚠ download binary not found — run `cargo build --bins` and restart".to_string()),
        )
            .into_response();
    };

    // reset progress tracking for a fresh run
    {
        let mut prog = state.progress.lock().unwrap();
        prog.clear();
    }
    {
        let mut cur = state.current_url.lock().unwrap();
        *cur = None;
    }

    match Command::new(exe)
        .arg("-e")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => {
            *state.child.lock().unwrap() = Some(child);
            let _ = state.log_tx.send("── runner started ──".to_string());
            Html("▶ download started — watch the Logs tab".to_string()).into_response()
        }
        Err(e) => {
            let _ = state
                .log_tx
                .send(format!("── error: failed to start runner: {e} ──"));
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(format!("⚠ failed to start runner: {e}")),
            )
                .into_response()
        }
    }
}

async fn stop(State(state): State<Arc<AppState>>) -> StatusCode {
    let mut guard = state.child.lock().unwrap();
    if let Some(mut child) = guard.take() {
        let _ = child.kill();
        let _ = child.wait();
        let _ = state.log_tx.send("── runner stopped ──".to_string());
    }
    StatusCode::OK
}

/// The `download` binary sits next to `serve` in the cargo target directory.
fn cli_exe() -> Option<PathBuf> {
    let dir = std::env::current_exe().ok()?.parent()?.to_path_buf();
    let name = if cfg!(windows) {
        "download.exe"
    } else {
        "download"
    };
    let path = dir.join(name);
    path.is_file().then_some(path)
}

// ---------- logs ----------

async fn log_stream(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let stream = BroadcastStream::new(state.log_tx.subscribe()).filter_map(|result| match result {
        Ok(line) => Some(Ok::<Event, Infallible>(
            Event::default().data(format!("<div>{line}</div>")),
        )),
        Err(_) => None, // receiver lagged behind; drop the missed lines
    });
    Sse::new(stream).keep_alive(KeepAlive::default())
}

async fn clear_logs() -> Html<&'static str> {
    let _ = fs::write("output.log", "");
    Html("<div>[console cleared]</div>")
}

async fn debug_progress(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let progress = state.progress.lock().unwrap().clone();
    let current_url = state.current_url.lock().unwrap().clone();
    let value = serde_json::json!({
        "current_url": current_url,
        "progress": progress
    });
    Json(value)
}

/// Tails `output.log` (the same file `run_download` writes) and broadcasts new
/// lines to every connected SSE client.
fn spawn_log_tail(state: Arc<AppState>) {
    tokio::spawn(async move {
        let mut offset: u64 = 0;
        loop {
            if let Ok(metadata) = fs::metadata("output.log") {
                let len = metadata.len();
                if len < offset {
                    offset = 0;
                }
                if len > offset {
                    if let Ok(bytes) = fs::read("output.log") {
                        let chunk = String::from_utf8_lossy(&bytes[offset as usize..]);
                        offset = len;
                        for line in chunk.lines() {
                            // process each carriage-return segment for progress updates
                            for segment in line.split('\r') {
                                let seg = segment.trim();
                                if seg.is_empty() {
                                    continue;
                                }
                                // update progress tracking
                                if seg.starts_with("Processing URL: ") {
                                    let url = seg["Processing URL: ".len()..].trim().to_string();
                                    // mark previous download as complete
                                    if let Some(old) = state.current_url.lock().unwrap().clone() {
                                        let mut prog = state.progress.lock().unwrap();
                                        prog.insert(old, 100.0);
                                    }
                                    let mut cur = state.current_url.lock().unwrap();
                                    *cur = Some(url.clone());
                                    let mut prog = state.progress.lock().unwrap();
                                    prog.insert(url, 0.0);
                                } else if seg.contains("[download]") {
                                    // find the token ending with %
                                    if let Some(pct_pos) = seg.rfind('%') {
                                        let before = &seg[..pct_pos];
                                        if let Some(num_str) = before.split_whitespace().next_back()
                                        {
                                            if let Ok(pct) = num_str.parse::<f32>() {
                                                if let Some(cur_url) =
                                                    state.current_url.lock().unwrap().clone()
                                                {
                                                    let mut prog = state.progress.lock().unwrap();
                                                    let entry =
                                                        prog.entry(cur_url.clone()).or_insert(0.0);
                                                    // only update if pct is greater or equal to avoid regression
                                                    if pct >= *entry {
                                                        *entry = pct.min(100.0);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                } else if seg.contains("Destination:") {
                                    // yt-dlp prints Destination: after a successful download
                                    if let Some(cur_url) = state.current_url.lock().unwrap().clone()
                                    {
                                        let mut prog = state.progress.lock().unwrap();
                                        prog.insert(cur_url, 100.0);
                                    }
                                }
                            }
                            let _ = state.log_tx.send(html_escape(line));
                        }
                    }
                }
            }
            tokio::time::sleep(Duration::from_millis(150)).await;
        }
    });
}

fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
