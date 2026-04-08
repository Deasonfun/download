use crate::{config::Config, run_download::run_download};

use std::{
    fs::{self},
    path::PathBuf,
};

const VIDEO_FORMATS: [&str; 6] = ["avi", "flv", "mkv", "mov", "mp4", "webm"];
const AUDIO_FORMATS: [&str; 8] = ["aac", "alac", "flac", "m4a", "mp3", "opus", "vorbis", "wav"];

pub enum CmdArgs {
    Add,
    Remove,
    ExportAudio,
    VideoFormat,
    AudioFormat,
    DownloadDest,
    Execute,
    None,
}

impl CmdArgs {
    pub fn from_arg(arg: &str) -> Self {
        match arg {
            "-a" => CmdArgs::Add,
            "-r" => CmdArgs::Remove,
            "-A" => CmdArgs::ExportAudio,
            "-f" => CmdArgs::VideoFormat,
            "-F" => CmdArgs::AudioFormat,
            "-d" => CmdArgs::DownloadDest,
            "-e" => CmdArgs::Execute,
            _ => CmdArgs::None,
        }
    }

    pub async fn run(
        &self,
        args: Vec<String>,
        arg_num: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        fn read_config() -> Result<Config, Box<dyn std::error::Error>> {
            let config_json = fs::read_to_string("config.json")
                .map_err(|e| format!("Could not open config: {e}"))?;
            let config: Config = serde_json::from_str(&config_json)?;
            Ok(config)
        }
        fn write_config(config: Config) -> Result<(), Box<dyn std::error::Error>> {
            let _ = fs::write(
                "config.json",
                serde_json::to_string_pretty(&config)
                    .map_err(|e| format!("Could not read new config: {e}"))?,
            )
            .map_err(|e| format!("Could not write new config: {e}"))?;
            Ok(())
        }
        match self {
            CmdArgs::Add => {
                if let Some(url) = args.get(arg_num + 1) {
                    let mut config = read_config()?;
                    config.videos.push(url.clone());

                    write_config(config)?;
                } else {
                    println!("No URL input with -a");
                }
                Ok(())
            }
            CmdArgs::Remove => {
                if let Some(url) = args.get(arg_num + 1) {
                    let mut config = read_config()?;
                    let r_url_index = config
                        .videos
                        .binary_search(url)
                        .map_err(|_| format!("Cannot find URL specified: {url}"))?;

                    config.videos.remove(r_url_index);

                    write_config(config)?;
                } else {
                    println!("No URL input with -a");
                }
                Ok(())
            }
            CmdArgs::ExportAudio => {
                let mut config = read_config()?;
                config.audio_export = !config.audio_export;

                write_config(config)?;
                Ok(())
            }
            CmdArgs::VideoFormat => {
                if let Some(format) = args.get(arg_num + 1) {
                    let mut config = read_config()?;
                    if VIDEO_FORMATS.contains(&format.as_str()) {
                        config.video_format = format.clone();
                    } else {
                        panic!(
                            "Please choose from the available formats: {:?}",
                            VIDEO_FORMATS
                        );
                    }

                    write_config(config)?;
                } else {
                    println!("No format input with -f");
                }
                Ok(())
            }
            CmdArgs::AudioFormat => {
                if let Some(format) = args.get(arg_num + 1) {
                    let mut config = read_config()?;
                    if AUDIO_FORMATS.contains(&format.as_str()) {
                        config.audio_format = format.clone();
                    } else {
                        panic!(
                            "Please choose from the available formats: {:?}",
                            AUDIO_FORMATS
                        );
                    }

                    write_config(config)?;
                } else {
                    println!("No format input with -F");
                }
                Ok(())
            }
            CmdArgs::DownloadDest => {
                if let Some(dest) = args.get(arg_num + 1) {
                    let mut config = read_config()?;
                    if fs::exists(dest)? {
                        config.download_dest = dest.clone();
                    } else {
                        panic!("The file path {dest} does not exist.");
                    }

                    write_config(config)?;
                } else {
                    println!("No download destination input with -d");
                }
                Ok(())
            }
            CmdArgs::Execute => {
                run_download(PathBuf::from("libs")).await?;
                Ok(())
            }
            CmdArgs::None => Ok(()),
        }
    }
}
