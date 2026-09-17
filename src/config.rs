use serde::{Deserialize, Serialize};

use std::fs;

pub const VIDEO_FORMATS: [&str; 6] = ["avi", "flv", "mkv", "mov", "mp4", "webm"];
pub const AUDIO_FORMATS: [&str; 8] = ["aac", "alac", "flac", "m4a", "mp3", "opus", "vorbis", "wav"];

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub download_dest: String,
    pub video_format: String,
    pub audio_export: bool,
    pub audio_format: String,
    pub thumbnail_export: bool,
    pub videos: Vec<String>,
}

impl Config {
    pub fn load() -> Result<Config, Box<dyn std::error::Error>> {
        let config_json =
            fs::read_to_string("config.json").map_err(|e| format!("Could not open config: {e}"))?;
        let config: Config = serde_json::from_str(&config_json)?;
        Ok(config)
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        fs::write(
            "config.json",
            serde_json::to_string_pretty(self)
                .map_err(|e| format!("Could not read new config: {e}"))?,
        )
        .map_err(|e| format!("Could not write new config: {e}"))?;
        Ok(())
    }
}
