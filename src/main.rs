use std::path::PathBuf;
use std::process::Command;
use yt_dlp::Youtube;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let execs_dir = PathBuf::from("libs");
    let output_dir = PathBuf::from("output");

    let fetcher = Youtube::with_new_binaries(&execs_dir, output_dir).await?;

    let dlp_bin = execs_dir.join("yt-dlp");

    Command::new(dlp_bin)
        .arg("https://www.youtube.com/watch?v=WbvNur3L-yc")
        .spawn()
        .expect("youtube-dlp command failed");
    //let url = String::from("https://www.youtube.com/watch?v=WbvNur3L-yc");
    //let video_path = fetcher.download_video_from_url(url, "video.mp4").await?;
    Ok(())
}
