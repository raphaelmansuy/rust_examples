use futures::stream::{self, TryStreamExt};
use serde::{Deserialize, Serialize};
use reqwest::Client;
use thiserror::Error;
use futures::stream::TryStream;

#[derive(Serialize,Deserialize,Debug)]
struct User {
    id: u32,
    name: String,
}

#[derive(Serialize,Deserialize,Debug)]
struct Chunk {
    data: User,
    done: bool,
}

#[derive(Error, Debug)]
enum FetchStreamError {
    #[error("Request failed with status: {0}")]
    RequestFailed(reqwest::StatusCode),
    #[error("Failed to deserialize user: {0}")]
    DeserializationFailed(serde_json::Error),
    #[error("Request error: {0}")]
    RequestError(#[from] reqwest::Error),
}

async fn fetch_stream(url: &str) -> Result<impl TryStream<Ok = Chunk, Error = FetchStreamError>, FetchStreamError> {
    let client = Client::new();
    let response = client.get(url).send().await?;

    let status = response.status();
    let body = response.bytes_stream();

    if status.is_success() {
        Ok(body.map_err(FetchStreamError::RequestError).and_then(|chunk| {
            async move {
                let chunk = serde_json::from_slice(&chunk).map_err(FetchStreamError::DeserializationFailed)?;
                Ok(chunk)
            }
        }))
    } else {
        Err(FetchStreamError::RequestFailed(status))
    }
}

#[tokio::main]
async fn main() -> Result<(), FetchStreamError> {
    const USER_API_URL: &str = "http://127.0.0.1:8080/users";
    let user_stream = fetch_stream(USER_API_URL).await?;

    user_stream
        .try_for_each(|chunk| async move {
            println!("User: {:?}", chunk.data);
            Ok(())
        })
        .await?;

    Ok(())
}