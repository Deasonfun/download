use crate::{
    config::{Config, AUDIO_FORMATS, VIDEO_FORMATS},
    run_download::run_download,
};

use std::{fs, path::PathBuf};

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
        match self {
            CmdArgs::Add => {
                if let Some(url) = args.get(arg_num + 1) {
                    let mut config = Config::load()?;
                    config.videos.push(url.clone());

                    config.save()?;
                } else {
                    println!("No URL input with -a");
                }
                Ok(())
            }
            CmdArgs::Remove => {
                if let Some(url) = args.get(arg_num + 1) {
                    let mut config = Config::load()?;
                    let r_url_index = config
                        .videos
                        .binary_search(url)
                        .map_err(|_| format!("Cannot find URL specified: {url}"))?;

                    config.videos.remove(r_url_index);

                    config.save()?;
                } else {
                    println!("No URL input with -a");
                }
                Ok(())
            }
            CmdArgs::ExportAudio => {
                let mut config = Config::load()?;
                config.audio_export = !config.audio_export;

                config.save()?;
                Ok(())
            }
            CmdArgs::VideoFormat => {
                if let Some(format) = args.get(arg_num + 1) {
                    let mut config = Config::load()?;
                    if VIDEO_FORMATS.contains(&format.as_str()) {
                        config.video_format = format.clone();
                    } else {
                        panic!(
                            "Please choose from the available formats: {:?}",
                            VIDEO_FORMATS
                        );
                    }

                    config.save()?;
                } else {
                    panic!("No format input with -f");
                }
                Ok(())
            }
            CmdArgs::AudioFormat => {
                if let Some(format) = args.get(arg_num + 1) {
                    let mut config = Config::load()?;
                    if AUDIO_FORMATS.contains(&format.as_str()) {
                        config.audio_format = format.clone();
                    } else {
                        panic!(
                            "Please choose from the available formats: {:?}",
                            AUDIO_FORMATS
                        );
                    }

                    config.save()?;
                } else {
                    panic!("No format input with -F");
                }
                Ok(())
            }
            CmdArgs::DownloadDest => {
                if let Some(dest) = args.get(arg_num + 1) {
                    let mut config = Config::load()?;
                    if fs::exists(dest)? {
                        config.download_dest = dest.clone();
                    } else {
                        panic!("The file path {dest} does not exist.");
                    }

                    config.save()?;
                } else {
                    panic!("No download destination input with -d");
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
