mod cmd_args;
mod config;
mod download_libraries;
mod run_download;

use crate::cmd_args::{CmdArgs};
use crate::download_libraries::download_libraries;
use crate::run_download::run_download;

use std::env;
use std::fs::{self, File};
use std::path::PathBuf;

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
            let _ = cmd.run(args.clone(), i).map_err(|e| format!("Could not run with command arguments: {e}"))?;
        }
    } else {
        run_download(execs_dir).await.map_err(|e| format!("Could not run: {e}"))?;
    }
    
    Ok(())
    
}
