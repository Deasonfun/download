use crate::config::Config;

use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::fs::{self};

pub async fn run_download(execs_dir: PathBuf) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let dlp_bin = execs_dir.join("yt-dlp");
    let ffmpeg_bin = execs_dir.join("ffmpeg");

    let config_json = fs::read_to_string("config.json").map_err(|e| format!("Could not read config: {e}"))?;
    let config: Config = serde_json::from_str(&config_json).map_err(|e| format!("Could not convert convert config to string: {e}"))?;

    let cert_path = execs_dir.join("certs").join("cacert.pem");

    let mut command_args = vec!["--no-part"];

    command_args.push("-P");
    command_args.push(&config.download_dest);
    println!("Download dest: {}", config.download_dest);

    command_args.push("-t");
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
            command_args.push(ffmpeg_bin.to_str().ok_or("Could not find path to ffmpeg")?);
        }
        false => (),
    }

    match config.thumbnail_export {
        true => {
            println!("Thumbnail export: enabled");
            command_args.push("--write-thumbnail");
        }
        false => (),
    }

    fs::write("output.log", "")?;
    for result in config.videos {
        let record = result;

        println!("{}", record);

        let output = Command::new(&dlp_bin)
            .env("SSL_CERT_FILE", &cert_path)
            .env("REQUESTS_CA_BUNDLE", &cert_path)
            .args(&command_args)
            .arg(record.clone())
            .output()?;
        println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        if let Ok(mut log_file) = fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open("output.log") {
            log_file.write_all(format!("Processing URL: {}\n", record).as_bytes())?;
            log_file.write_all(&output.stdout)?;
            log_file.write_all(&output.stderr)?;
            log_file.write_all(b"\n\n\n")?;
            println!("{}", output.status);
        } else {
            println!("Could not open log file.");
        }
        
    }

    Ok(())
}