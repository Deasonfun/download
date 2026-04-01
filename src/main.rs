use std::env;
use std::env::consts;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

use serde::{Deserialize, Serialize};
use tar::Archive;
use xz2::read::XzDecoder;
use zip::ZipArchive;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    download_dest: String,
    video_format: String,
    audio_export: bool,
    audio_format: String,
    thumbnail_export: bool,
    videos: Vec<String>,
}

enum CmdArgs {
    Add,
    Remove,
    ExportAudio,
    None,
}

impl CmdArgs {
    fn from_arg(arg: &str) -> Self {
        match arg {
            "-a" => CmdArgs::Add,
            "-r" => CmdArgs::Remove,
            "-A" => CmdArgs::ExportAudio,
            _ => CmdArgs::None,
        }
    }
    fn run(&self, args: Vec<String>, arg_num: usize) {
        match self {
            CmdArgs::Add => {
                if let Some(url) = args.get(arg_num + 1) {
                    let config_json = fs::read_to_string("config.json").unwrap();
                    let mut config: Config = serde_json::from_str(&config_json).unwrap();
                    config.videos.push(url.clone());

                    let _ = fs::write(
                        "config.json",
                        serde_json::to_string_pretty(&config).unwrap(),
                    )
                    .unwrap();
                } else {
                    println!("No URL input with -a");
                }
            }
            CmdArgs::Remove => {
                if let Some(url) = args.get(arg_num + 1) {
                    let config_json = fs::read_to_string("config.json").unwrap();
                    let mut config: Config = serde_json::from_str(&config_json).unwrap();
                    let r_url_index = match config.videos.binary_search(url) {
                        Ok(i) => i,
                        Err(_) => {
                            println!("Cannot find URL {} in the queue.", url);
                            return;
                        }
                    };
                    config.videos.remove(r_url_index);

                    let _ = fs::write(
                        "config.json",
                        serde_json::to_string_pretty(&config).unwrap(),
                    )
                    .unwrap();
                } else {
                    println!("No URL input with -a");
                }
            }
            CmdArgs::ExportAudio => {
                if let Some(audio_format) = args.get(arg_num + 1) {
                    let config_json = fs::read_to_string("config.json").unwrap();
                    let mut config: Config = serde_json::from_str(&config_json).unwrap();
                    config.audio_export = true;
                    config.audio_format = audio_format.clone();
                    let _ = fs::write(
                        "config.json",
                        serde_json::to_string_pretty(&config).unwrap(),
                    )
                    .unwrap();
                } else {
                    println!("No audio format given.");
                }
            }
            CmdArgs::None => {}
        }
    }
}

async fn download_libraries(
    execs_dir: PathBuf,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    if consts::OS == "windows" {
        println!("windows");

        println!("Downloading libraries...");

        std::fs::create_dir_all(&execs_dir)?;

        let dlp = execs_dir.join("yt-dlp");
        let mut exec = File::create(&dlp)?;

        let response = reqwest::get(
            "https://github.com/yt-dlp/yt-dlp/releases/download/2026.03.17/yt-dlp.exe",
        )
        .await?;
        let bytes = response.bytes().await?;
        exec.write_all(&bytes)?;

        let ffmpeg = execs_dir.join("ffmpeg.zip");
        let mut exec = File::create(&ffmpeg)?;

        let response =
                    reqwest::get("https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip")
                    .await?;
        let bytes = response.bytes().await?;
        exec.write_all(&bytes)?;

        let ffmpeg_file = File::open(&ffmpeg)?;

        let mut decompressor = ZipArchive::new(ffmpeg_file)?;

        decompressor.extract(&execs_dir)?;

        let ffmpeg_bin_dir = PathBuf::from("libs/ffmpeg-master-latest-win64-gpl/bin");
        let ffmpeg_bin = ffmpeg_bin_dir.join("ffmpeg.exe");

        fs::copy(ffmpeg_bin, &execs_dir.join("ffmpeg.exe"))?;
        let _ = fs::remove_file(PathBuf::from("libs/ffmpeg.zip"));
        let _ = fs::remove_dir_all(PathBuf::from("libs/ffmpeg-master-latest-win64-gpl"));
        Ok(())
    } else {
        //let yt_dlp_path = exec_dir.join("yt-dlp");
        //let mut yt_dlp_dest = File::create(&yt_dlp_path);

        println!("Downloading libraries...");

        std::fs::create_dir_all(&execs_dir)?;

        let dlp = execs_dir.join("yt-dlp");
        let mut exec = File::create(&dlp)?;

        let response =
            reqwest::get("https://github.com/yt-dlp/yt-dlp/releases/download/2026.03.17/yt-dlp")
                .await?;
        let bytes = response.bytes().await?;
        exec.write_all(&bytes)?;

        #[cfg(unix)]
        {
            let mut perms = std::fs::metadata(&dlp)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&dlp, perms)?;
        }

        let ffmpeg = execs_dir.join("ffmpeg.tar.xz");
        let mut exec = File::create(&ffmpeg)?;

        let response =
                        reqwest::get("https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-linux64-gpl.tar.xz")
                            .await?;
        let bytes = response.bytes().await?;
        exec.write_all(&bytes)?;

        let ffmpeg_file = File::open(&ffmpeg)?;

        let decompressor = XzDecoder::new(ffmpeg_file);
        let mut archive = Archive::new(decompressor);
        archive.unpack(&execs_dir)?;

        let ffmpeg_bin_dir = PathBuf::from("libs/ffmpeg-master-latest-linux64-gpl/bin");
        let ffmpeg_bin = ffmpeg_bin_dir.join("ffmpeg");

        fs::copy(ffmpeg_bin, &execs_dir.join("ffmpeg"))?;
        let _ = fs::remove_file(PathBuf::from("libs/ffmpeg.tar.xz"));
        let _ = fs::remove_dir_all(PathBuf::from("libs/ffmpeg-master-latest-linux64-gpl"));

        #[cfg(unix)]
        {
            let mut perms = std::fs::metadata(PathBuf::from("libs/ffmpeg"))?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(PathBuf::from("libs/ffmpeg"), perms)?;
        }
        Ok(())
    }
}

async fn run_download(execs_dir: PathBuf) -> std::result::Result<(), Box<dyn std::error::Error>> {
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
            .args(&command_args)
            .arg(record.clone())
            .output()
            .expect("Youtube DLP Not Found. Please download the executable.");
        println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        let mut log_file = fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open("output.log")?;
        log_file.write_all(format!("Processing URL: {}\n", record).as_bytes())?;
        log_file.write_all(&output.stdout)?;
        log_file.write_all(&output.stderr)?;
        log_file.write_all(b"\n\n\n")?;
        println!("{}", output.status);
    }

    Ok(())
}

#[tokio::main]
pub async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let execs_dir = PathBuf::from("libs");

    if !execs_dir.exists() {
        download_libraries(execs_dir.clone()).await?;
    }

    if !PathBuf::from("./config.json").exists() {
        File::create("./config.json").unwrap();
        let _ = fs::write(
            "./config.json",
            r#"{
                "download_dest": "./downloads",
                "video_format": "mp4",
                "audio_export": false,
                "audio_format": "mp3",
                "thumbnail_export": false,
                "videos": [
                    "https://www.youtube.com/watch?v=EwTZ2xpQwpA",
                    "https://www.youtube.com/watch?v=dQw4w9WgXcQ"
                ]
            }"#,
        );
    }

    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        for (i, arg) in args.iter().enumerate() {
            let cmd = CmdArgs::from_arg(&arg);
            cmd.run(args.clone(), i);
        }
    } else {
        run_download(execs_dir).await?;
    }
    
    Ok(())
    
}
