use std::fs::File;
use std::path::PathBuf;
use std::process::Command;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let execs_dir = PathBuf::from("libs");

    let dlp_bin = execs_dir.join("yt-dlp");

    let videos_list_csv = File::open("videos.csv")?;
    let mut rdr = csv::Reader::from_reader(videos_list_csv);
    for result in rdr.records() {
        let record = result?;

        let output = Command::new(&dlp_bin)
            .arg(&record[0])
            .output()
            .expect("youtube-dlp command failed");
        println!("{}", output.status);
    }

    Ok(())
}
