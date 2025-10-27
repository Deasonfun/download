use std::fs::{self, File};
use std::path::PathBuf;
use std::process::Command;

use serde::{Deserialize, Serialize};
use serde_json::Result;

#[derive(Serialize, Deserialize)]
struct Config {
    audio_export: bool,
    audio_format: String,
    videos: Vec<String>,
}

#[tokio::main]
pub async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let execs_dir = PathBuf::from("libs");

    let dlp_bin = execs_dir.join("yt-dlp");
    let ffmpeg_bin = execs_dir.join("ffmpeg");

    let config_json = fs::read_to_string("config.json")?;
    let config: Config = serde_json::from_str(&config_json)?;

    let mut command_args = vec!["-f mp4"];

    let audio_format = config.audio_format;
    println!("Audio format: {}", audio_format);

    match config.audio_export {
        true => {
            command_args.push("-x");
            command_args.push("--audio-format");
            command_args.push(audio_format.as_str());
            command_args.push("--ffmpeg-location");
            command_args.push(ffmpeg_bin.to_str().unwrap());
        },
        false => ()
    }

    //let videos_list_csv = File::open("videos.csv")?;
    //let mut rdr = csv::Reader::from_reader(videos_list_csv);
    for result in config.videos {
        let record = result;

        println!("{}", record);
        println!("{:?}", &command_args);

        let output = Command::new(&dlp_bin)
            .args(&command_args)
            .arg(record.clone())
            .output()?;
        println!("{}", output.status);

        
    }

    Ok(())
}
