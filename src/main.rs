use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum::{routing::get, Router};

use rusqlite::{params, Connection, MappedRows, Result};

use askama::Template;

use std::fs::{self, File};
use std::path::PathBuf;
use std::process::Command;

//#[derive(Template)]
//#[template(path = "list.html")]
//struct ListTemplate<'a> {
//    list: MappedRows<'a, String>,
//}

#[derive(Debug)]
struct Trans {
    url: String,
}

enum RouteError {
    DatabaseError,
}

impl IntoResponse for RouteError {
    fn into_response(self) -> axum::response::Response {
        let body = match self {
            RouteError::DatabaseError => "Database error",
        };

        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}

async fn index() -> Html<String> {
    let index_html = Html(fs::read_to_string("index.html"));
    match index_html.0 {
        Ok(index_html) => Html(index_html),
        Err(err) => panic!("{err}"),
    }
}

#[axum::debug_handler]
async fn download_add(Path(url): Path<String>) -> Result<Html<String>, RouteError> {
    println!("{url}");
    let conn = Connection::open("database.sql").expect("Unable to connect to database");
    let trans = Trans { url: url };
    conn.execute("INSERT INTO trans (URL) VALUES (?1)", params![&trans.url])
        .expect("Unable to insert data into database");
    let mut url_query = conn
        .prepare("SELECT URL FROM trans")
        .expect("Could not select rows from database");
    let url_iter = url_query
        .query_map([], |row| Ok(Trans { url: row.get(0)? }))
        .expect("could not insert data into rows");
    let mut list = "".to_string();
    for url in url_iter {
        list = format!("{}<tr><td>{}</td></tr>", list, url.unwrap().url);
        println!("{list}");
    }
    let urls_list = format!("{}", list).to_string();
    Ok(Html(urls_list))
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let execs_dir = PathBuf::from("libs");

    let dlp_bin = execs_dir.join("yt-dlp");

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

    let app = Router::new()
        .route("/", get(index))
        .route("/download_add/{url}", get(download_add));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:6969").await.unwrap();

    axum::serve(listener, app).await.unwrap();

    Ok(())
}
