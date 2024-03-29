use futures::stream::{self, StreamExt};
use serde::{Deserialize};
use std::io::{BufRead, BufReader, Cursor};
use reqwest::Client;
use std::error::Error;
use futures::stream::Stream;

#[derive(Deserialize, Debug)]
struct User {
    id: u32,
    name: String,
}

fn create_line_stream<R: BufRead>(reader: R) -> impl Iterator<Item = Result<String, std::io::Error>> {
    let mut buf_reader = BufReader::new(reader);
    std::iter::from_fn(move || {
        let mut line = String::new();
        match buf_reader.read_line(&mut line) {
            Ok(0) => None,
            Ok(_) => Some(Ok(line.trim_end().to_string())),
            Err(e) => Some(Err(e)),
        }
    })
}

async fn fetch_lines(url: &str) -> Result<impl Stream<Item = Result<String, Box<dyn Error + Send + Sync>>>, Box<dyn Error + Send + Sync>> {
    let client = Client::new();
    let response = client.get(url).send().await?;

    let status = response.status();
    let body = response.bytes_stream();

    if status.is_success() {
        Ok(body.map(|chunk| {
            chunk.map_err(|e| e.into()).and_then(|chunk| {
                String::from_utf8(chunk.to_vec()).map_err(|e| e.into())
            })
        }))
    } else {
        let error_msg = format!("Error: {}", status);
        Err(std::io::Error::new(std::io::ErrorKind::Other, error_msg).into())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let url = "http://127.0.0.1:8080/users";
    let mut lines = fetch_lines(url).await?;

    while let Some(line) = lines.next().await {
        if let Ok(line) = line {
            if let Ok(user) = serde_json::from_str::<User>(&line) {
                println!("User: {:?}", user);
            }
        }
    }

    Ok(())
}
