use std::fs::{self};
use std::path::PathBuf;
use std::process::Command;
use std::io::Write;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Config {
    download_dest: String,
    video_format: String,
    audio_export: bool,
    audio_format: String,
    thumbnail_export: bool,
    videos: Vec<String>,
}

#[tokio::main]
pub async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {

    let execs_dir = PathBuf::from("libs");

    let dlp_bin = execs_dir.join("yt-dlp");
    let ffmpeg_bin = execs_dir.join("ffmpeg");

    let config_json = fs::read_to_string("config.json")?;
    let config: Config = serde_json::from_str(&config_json)?;

    
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
            .output()?;
        println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        let mut log_file = fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open("output.log")?;
        log_file.write_all(&output.stdout)?;
        log_file.write_all(&output.stderr)?;
        println!("{}", output.status);
    }

    Ok(())
}
