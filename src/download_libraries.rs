use std::env::consts;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

use tar::Archive;
use xz2::read::XzDecoder;
use zip::ZipArchive;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

pub async fn download_libraries(
    execs_dir: PathBuf,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    match consts::OS {
        "windows" => {
            println!("Downloading libraries...");

            let dlp_url =
                "https://github.com/yt-dlp/yt-dlp/releases/download/2026.03.17/yt-dlp.exe";

            std::fs::create_dir_all(&execs_dir)
                .map_err(|e| format!("Could not create libraries directory: {e}"))?;

            let dlp = execs_dir.join("yt-dlp");
            let mut exec = File::create(&dlp)?;

            let response = reqwest::get(dlp_url)
                .await
                .map_err(|e| format!("There was an issue downloading dlp executable: {e}"))?;
            let bytes = response.bytes().await?;
            exec.write_all(&bytes)?;

            let ffmpeg = execs_dir.join("ffmpeg.zip");
            let mut exec = File::create(&ffmpeg)?;

            let response = reqwest::get("https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip")
                        .await.map_err(|e| format!("There was an issue downloading dlp executable: {e}"))?;
            let bytes = response.bytes().await?;
            exec.write_all(&bytes)?;

            let ffmpeg_file = File::open(&ffmpeg)?;

            let mut decompressor = ZipArchive::new(ffmpeg_file)?;

            decompressor
                .extract(&execs_dir)
                .map_err(|e| format!("Could not extract ffmpeg zip: {e}"))?;

            let ffmpeg_bin_dir = PathBuf::from("libs/ffmpeg-master-latest-win64-gpl/bin");
            let ffmpeg_bin = ffmpeg_bin_dir.join("ffmpeg.exe");
            let ffprobe_bin = ffmpeg_bin_dir.join("ffprobe.exe");

            fs::copy(ffmpeg_bin, &execs_dir.join("ffmpeg.exe"))?;
            fs::copy(ffprobe_bin, &execs_dir.join("ffprobe.exe"))?;
            let _ = fs::remove_file(PathBuf::from("libs/ffmpeg.zip"));
            let _ = fs::remove_dir_all(PathBuf::from("libs/ffmpeg-master-latest-win64-gpl"));
        }
        "macos" => {
            println!("Downloading libraries...");

            let dlp_url = "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_macos";
            let ffmpeg_url = "https://evermeet.cx/ffmpeg/ffmpeg-8.1.zip";
            let ffprobe_url = "https://evermeet.cx/ffmpeg/ffprobe-8.1.zip";

            std::fs::create_dir_all(&execs_dir)
                .map_err(|e| format!("Could not create libraries directory: {e}"))?;

            let dlp = execs_dir.join("yt-dlp");
            let mut exec = File::create(&dlp)?;

            let response = reqwest::get(dlp_url)
                .await
                .map_err(|e| format!("There was an issue downloading dlp executable: {e}"))?;
            let bytes = response.bytes().await?;
            exec.write_all(&bytes)?;

            #[cfg(unix)]
            {
                let mut perms = std::fs::metadata(&dlp)?.permissions();
                perms.set_mode(0o755);
                std::fs::set_permissions(&dlp, perms)
                    .map_err(|e| format!("Could not set permission on dlp: {e}"))?;
            }

            let ffmpeg = execs_dir.join("ffmpeg.zip");
            let mut exec = File::create(&ffmpeg)?;

            let response = reqwest::get(ffmpeg_url)
                .await
                .map_err(|e| format!("There was an issue downloading ffmpeg executable: {e}"))?;
            let bytes = response.bytes().await?;
            exec.write_all(&bytes)?;

            let ffmpeg_file = File::open(&ffmpeg)?;

            let mut decompressor = ZipArchive::new(ffmpeg_file)?;

            decompressor
                .extract(&execs_dir)
                .map_err(|e| format!("Could not extract ffmpeg zip: {e}"))?;

            let ffprobe = execs_dir.join("ffprobe.zip");
            let mut exec = File::create(&ffprobe)?;

            let response = reqwest::get(ffprobe_url)
                .await
                .map_err(|e| format!("There was an issue downloading ffmpeg executable: {e}"))?;
            let bytes = response.bytes().await?;
            exec.write_all(&bytes)?;

            let ffprobe_file = File::open(&ffprobe)?;

            let mut decompressor = ZipArchive::new(ffprobe_file)?;

            decompressor
                .extract(&execs_dir)
                .map_err(|e| format!("Could not extract ffmpeg zip: {e}"))?;

            let _ = fs::remove_file(PathBuf::from("libs/ffmpeg.zip"));
            let _ = fs::remove_file(PathBuf::from("libs/ffprobe.zip"));

            #[cfg(unix)]
            {
                let mut perms = std::fs::metadata(PathBuf::from("libs/ffmpeg"))?.permissions();
                perms.set_mode(0o755);
                std::fs::set_permissions(PathBuf::from("libs/ffmpeg"), perms)
                    .map_err(|e| format!("Could not set permission on ffmpeg: {e}"))?;
            }
        }
        "linux" => {
            println!("Downloading libraries...");

            let dlp_url =
                "https://github.com/yt-dlp/yt-dlp/releases/download/2026.03.17/yt-dlp_linux";

            std::fs::create_dir_all(&execs_dir)
                .map_err(|e| format!("Could not create libraries directory: {e}"))?;

            let dlp = execs_dir.join("yt-dlp");
            let mut exec = File::create(&dlp)?;

            let response = reqwest::get(dlp_url)
                .await
                .map_err(|e| format!("There was an issue downloading dlp executable: {e}"))?;
            let bytes = response.bytes().await?;
            exec.write_all(&bytes)?;

            #[cfg(unix)]
            {
                let mut perms = std::fs::metadata(&dlp)?.permissions();
                perms.set_mode(0o755);
                std::fs::set_permissions(&dlp, perms)
                    .map_err(|e| format!("Could not set permission on dlp: {e}"))?;
            }

            let ffmpeg = execs_dir.join("ffmpeg.tar.xz");
            let mut exec = File::create(&ffmpeg)?;

            let response =
                reqwest::get("https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-linux64-gpl.tar.xz")
                .await.map_err(|e| format!("There was an issue downloading ffmpeg executable: {e}"))?;
            let bytes = response.bytes().await?;
            exec.write_all(&bytes)?;

            let ffmpeg_file = File::open(&ffmpeg)?;

            let decompressor = XzDecoder::new(ffmpeg_file);
            let mut archive = Archive::new(decompressor);
            archive
                .unpack(&execs_dir)
                .map_err(|e| format!("Could not extract ffmpeg archive: {e}"))?;

            let ffmpeg_bin_dir = PathBuf::from("libs/ffmpeg-master-latest-linux64-gpl/bin");
            let ffmpeg_bin = ffmpeg_bin_dir.join("ffmpeg");
            let ffprobe_bin = ffmpeg_bin_dir.join("ffprobe");

            fs::copy(ffmpeg_bin, &execs_dir.join("ffmpeg"))?;
            fs::copy(ffprobe_bin, &execs_dir.join("ffprobe"))?;
            let _ = fs::remove_file(PathBuf::from("libs/ffmpeg.tar.xz"));
            let _ = fs::remove_dir_all(PathBuf::from("libs/ffmpeg-master-latest-linux64-gpl"));

            #[cfg(unix)]
            {
                let mut perms = std::fs::metadata(PathBuf::from("libs/ffmpeg"))?.permissions();
                perms.set_mode(0o755);
                std::fs::set_permissions(PathBuf::from("libs/ffmpeg"), perms)
                    .map_err(|e| format!("Could not set permission on ffmpeg: {e}"))?;
            }
        }
        _ => panic!("Your OS is not currently supported"),
    }
    Ok(())
}
