use crate::config::{Config};

use std::{fs::{self}};

pub enum CmdArgs {
    Add,
    Remove,
    ExportAudio,
    VideoFormat,
    None,
}

impl CmdArgs {
    pub fn from_arg(arg: &str) -> Self {
        match arg {
            "-a" => CmdArgs::Add,
            "-r" => CmdArgs::Remove,
            "-A" => CmdArgs::ExportAudio,
            "-f" => CmdArgs::VideoFormat,
            _ => CmdArgs::None,
        }
    }
    pub fn run(&self, args: Vec<String>, arg_num: usize) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            CmdArgs::Add => {
                if let Some(url) = args.get(arg_num + 1) {
                    let config_json = fs::read_to_string("config.json").map_err(|e| format!("Could not open config: {e}"))?;
                    let mut config: Config = serde_json::from_str(&config_json)?;
                    config.videos.push(url.clone());

                    let _ = fs::write(
                        "config.json",
                        serde_json::to_string_pretty(&config).map_err(|e| format!("Could not read new config: {e}"))?
                    )
                    .map_err(|e| format!("Could not write new config: {e}"))?;
                } else {
                    println!("No URL input with -a");
                }
                Ok(())
            }
            CmdArgs::Remove => {
                if let Some(url) = args.get(arg_num + 1) {

                    let config_json = fs::read_to_string("config.json").map_err(|e| format!("Could not open config: {e}"))?;
                    let mut config: Config = serde_json::from_str(&config_json)?;
                    let r_url_index = config.videos.binary_search(url).map_err(|_| format!("Cannot find URL specified: {url}"))?;

                    config.videos.remove(r_url_index);

                    let _ = fs::write(
                        "config.json",
                        serde_json::to_string_pretty(&config).map_err(|e| format!("Could not write new config: {e}"))?
                    )
                    .map_err(|e| format!("Could not write new config: {e}"))?;
                } else {
                    println!("No URL input with -a");
                }
                Ok(())
            }
            CmdArgs::ExportAudio => {
                let config_json = fs::read_to_string("config.json").map_err(|e| format!("Could not open config: {e}"))?;
                let mut config: Config = serde_json::from_str(&config_json)?;
                config.audio_export = !config.audio_export;
                let _ = fs::write(
                    "config.json",
                    serde_json::to_string_pretty(&config).map_err(|e| format!("Could not write new config: {e}"))?
                )
                .map_err(|e| format!("Could not write new config: {e}"))?;
                Ok(())
            },
            CmdArgs::VideoFormat => {
                if let Some(format) = args.get(arg_num + 1) {
                    let config_json = fs::read_to_string("config.json").map_err(|e| format!("Could not open config: {e}"))?;
                    let mut config: Config = serde_json::from_str(&config_json)?;
                    config.video_format = format.clone();

                    let _ = fs::write(
                        "config.json",
                        serde_json::to_string_pretty(&config).map_err(|e| format!("Could not read new config: {e}"))?
                    )
                    .map_err(|e| format!("Could not write new config: {e}"))?;
                } else {
                    println!("No format input with -f");
                }
                Ok(())
            }
            CmdArgs::None => Ok(())
        }
    }
}