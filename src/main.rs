use std::fs::File;
use std::path::PathBuf;
use std::process::Command;

use yt_dlp::Youtube;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let execs_dir = PathBuf::from("libs");
    let output_dir = PathBuf::from("output");

    let _fetcher = Youtube::with_new_binaries(&execs_dir, output_dir).await?;

    let dlp_bin = execs_dir.join("yt-dlp");

    println!("yes");
    let videos_list_csv = File::open("videos.csv")?;
    let mut rdr = csv::Reader::from_reader(videos_list_csv);
    for result in rdr.records() {
        let record = result?;
        println!("{:?}", &record[0]);
        let output = Command::new(&dlp_bin)
            .arg(&record[0])
            .output()
            .expect("youtube-dlp command failed");
        println!("{}", output.status);
    }

    //let url = String::from("https://www.youtube.com/watch?v=WbvNur3L-yc");
    //let video_path = fetcher.download_video_from_url(url, "video.mp4").await?;
    Ok(())
}
