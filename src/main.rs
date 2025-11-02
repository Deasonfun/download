use std::fs::{self};
use std::path::PathBuf;
use std::process::Command;
use std::io::Write;

use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};

use axum::{
    routing::get,
    Router,
    response::Html,
    extract::Query,
    debug_handler,
};

#[derive(Serialize, Deserialize)]
struct Config {
    download_dest: String,
    video_format: String,
    audio_export: bool,
    audio_format: String,
    thumbnail_export: bool,
    videos: Vec<String>,
}

#[derive(Deserialize)]
struct DLQuery {
    url: String,
    video_format: String,
    audio_export: bool,
    thumbnail_export: bool,
}

enum QueryError {
    IoError(std::io::Error),
    SerdeError(serde_json::Error),
}

impl IntoResponse for QueryError {
    fn into_response(self) -> axum::response::Response {
        match self {
            QueryError::IoError(e) => {
                let body = format!("IO Error: {}", e);
                (axum::http::StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
            }
            QueryError::SerdeError(e) => {
                let body = format!("Serialization Error: {}", e);
                (axum::http::StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
            }
        }
    }
}

async fn handler() -> Html<String> {
    Html(fs::read_to_string("index.html").unwrap())
}

async fn add_video(Query(params): Query<DLQuery>) -> Html<String> {
    println!("Received URL: {}", params.url);
    println!("Video Format: {:?}", params.video_format);
    println!("Export Audio: {:?}", params.audio_export);
    println!("Export Audio: {:?}", params.thumbnail_export);
    let config_json = fs::read_to_string("config.json").unwrap();
    let mut config: Config = serde_json::from_str(&config_json).unwrap();

    config.videos.push(params.url);
    config.video_format = params.video_format;
    config.audio_export = params.audio_export;
    config.thumbnail_export = params.thumbnail_export;

    let updated_config = serde_json::to_string_pretty(&config).unwrap();
    fs::write("config.json", updated_config).unwrap();

    let mut vids_list = vec![];
    for vid in &config.videos {
        let vid_component = fs::read_to_string("video_list_component.html").unwrap();
        let vid_component_filled = vid_component.replace("%VIDEO_URL%", &vid);
        vids_list.push(vid_component_filled);
    }
    println!("Updated video list: {:?}", config.videos);

    Html(vids_list.join("\n"))
}


#[debug_handler]
async fn get_video_list() -> Html<String> {
    let config_json = fs::read_to_string("config.json").unwrap();
    let config: Config = serde_json::from_str(&config_json).unwrap();
    
    let mut vids_list = vec![];
    for vid in &config.videos {
        let vid_component = fs::read_to_string("video_list_component.html").unwrap();
        let vid_component_filled = vid_component.replace("%VIDEO_URL%", &vid);
        vids_list.push(vid_component_filled);
    }
    println!("Updated video list: {:?}", config.videos);

    Html(vids_list.join("\n"))
}

async fn clear_queue() -> Html<String> {
    let config_json = fs::read_to_string("config.json").unwrap();
    let mut config: Config = serde_json::from_str(&config_json).unwrap();
    config.videos = [].to_vec();
    let updated_config = serde_json::to_string_pretty(&config).unwrap();
    fs::write("config.json", updated_config).unwrap();
    Html("".to_string())
}

async fn download() -> Result<(), QueryError> {
    let execs_dir = PathBuf::from("libs");

    let dlp_bin = execs_dir.join("yt-dlp");
    let ffmpeg_bin = execs_dir.join("ffmpeg");

    let config_json = fs::read_to_string("config.json").unwrap();
    let config: Config = serde_json::from_str(&config_json).unwrap();

    
    let mut command_args = vec!["--no-part"];

    command_args.push("-P");
    command_args.push(&config.download_dest);
    println!("Download dest: {}", config.download_dest);

    command_args.push("-f");
    command_args.push(config.video_format.as_str());
    println!("Video format: {}", config.video_format);

    let audio_format = config.audio_format;

    match config.audio_export {
        true => {
            println!("Audio format: {}", audio_format);
            command_args.push("-x");
            command_args.push("--audio-format");
            command_args.push(audio_format.as_str());
            command_args.push("--ffmpeg-location");
            command_args.push(ffmpeg_bin.to_str().unwrap());
        },
        false => ()
    }

    match config.thumbnail_export {
        true => {
            println!("Thumbnail export: enabled");
            command_args.push("--write-thumbnail");
        },
        false => ()
    }


    for result in config.videos {
        let record = result;

        println!("{}", record);

        let output = Command::new(&dlp_bin)
            .args(&command_args)
            .arg(record.clone())
            .output().unwrap();
        println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        let mut log_file = fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open("output.log").unwrap();
        log_file.write_all(&output.stdout).unwrap();
        log_file.write_all(&output.stderr).unwrap();
        println!("{}", output.status);
    }
    Ok(())
}

#[tokio::main]
pub async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {

    let app = Router::new()
    .route("/", get(handler))
    .route("/add/", get(add_video))
    .route("/video_list/", get(get_video_list))
    .route("/clear/", get(clear_queue))
    .route("/download/", get(download));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
