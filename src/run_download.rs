use crate::config::Config;

use std::fs::{self};
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

pub async fn run_download(
    execs_dir: PathBuf,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let dlp_bin = execs_dir.join("yt-dlp");

    let config_json =
        fs::read_to_string("config.json").map_err(|e| format!("Could not read config: {e}"))?;
    let config: Config = serde_json::from_str(&config_json)
        .map_err(|e| format!("Could not convert convert config to string: {e}"))?;

    let cert_path = execs_dir.join("certs").join("cacert.pem");

    let mut command_args = vec!["--no-part", "--force-overwrites"];

    let deno_bin = if cfg!(windows) {
        execs_dir.join("deno.exe")
    } else {
        execs_dir.join("deno")
    };

    let js_runtimes = if deno_bin.exists() {
        Some(format!("deno:{}", deno_bin.to_string_lossy()))
    } else {
        None
    };

    if let Some(js_runtimes) = js_runtimes.as_ref() {
        command_args.push("--js-runtimes");
        command_args.push(js_runtimes.as_str());
    }

    command_args.push("-P");
    command_args.push(&config.download_dest);
    println!("Download dest: {}", config.download_dest);

    command_args.push("--ffmpeg-location");
    command_args.push(execs_dir.to_str().ok_or("Could not find path to ffmpeg")?);

    let audio_format = config.audio_format;

    match config.audio_export {
        true => {
            println!("Audio format: {}", audio_format);
            command_args.push("-x");
            command_args.push("--audio-format");
            command_args.push(audio_format.as_str());
        }
        false => {
            command_args.push("-t");
            command_args.push(config.video_format.as_str());
            println!("Video format: {}", config.video_format);
        }
    }

    match config.thumbnail_export {
        true => {
            println!("Thumbnail export: enabled");
            command_args.push("--write-thumbnail");
        }
        false => (),
    }

    fs::write("output.log", "")?;
    let log_file = Arc::new(Mutex::new(
        fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open("output.log")?,
    ));
    for result in config.videos {
        let record = result;

        println!("{}", record);

        {
            let mut file = log_file.lock().unwrap();
            file.write_all(format!("Processing URL: {}\n", record).as_bytes())?;
        }

        let mut child = Command::new(&dlp_bin)
            .env("SSL_CERT_FILE", &cert_path)
            .env("REQUESTS_CA_BUNDLE", &cert_path)
            .args(&command_args)
            .arg(record.clone())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let mut stdout = child.stdout.take().expect("stdout");
        let mut stderr = child.stderr.take().expect("stderr");

        let log_file_stdout = Arc::clone(&log_file);
        let stdout_handle = thread::spawn(move || {
            let _ = io::copy(&mut stdout, &mut *log_file_stdout.lock().unwrap());
        });

        let log_file_stderr = Arc::clone(&log_file);
        let stderr_handle = thread::spawn(move || {
            let _ = io::copy(&mut stderr, &mut *log_file_stderr.lock().unwrap());
        });

        let status = child.wait()?;
        stdout_handle.join().ok();
        stderr_handle.join().ok();

        {
            let mut file = log_file.lock().unwrap();
            file.write_all(b"\n\n\n")?;
        }
        println!("{}", status);
    }

    Ok(())
}
