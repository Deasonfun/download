use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub download_dest: String,
    pub video_format: String,
    pub audio_export: bool,
    pub audio_format: String,
    pub thumbnail_export: bool,
    pub videos: Vec<String>,
}