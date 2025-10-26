use std::fs::{self, File};
use std::path::PathBuf;
use std::process::Command;

use serde::{Deserialize, Serialize};
use serde_json::Result;

#[derive(Serialize, Deserialize)]
struct Config {
    videos: Vec<String>,
}

#[tokio::main]
pub async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let execs_dir = PathBuf::from("libs");

    let dlp_bin = execs_dir.join("yt-dlp");

    let config_json = fs::read_to_string("config.json")?;
    let config: Config = serde_json::from_str(&config_json)?;

    //let videos_list_csv = File::open("videos.csv")?;
    //let mut rdr = csv::Reader::from_reader(videos_list_csv);
    for result in config.videos {
        let record = result;

        println!("{}", record);

        let output = Command::new(&dlp_bin)
            .args(["-f mp4", &record])
            .output()?;
        println!("{}", output.status);
    }

    Ok(())
}
