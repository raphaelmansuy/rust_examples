use futures::stream::{self, TryStreamExt};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use reqwest::Client;
use thiserror::Error;
use futures::stream::TryStream;
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug)]
struct User {
    id: u32,
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Chunk<T> {
    data: T,
    done: bool,
}

#[derive(Error, Debug)]
enum FetchStreamError {
    /// Request failed with the specified status code.
    #[error("Request failed with status: {0}")]
    RequestFailed(reqwest::StatusCode),
    /// Failed to deserialize the received data.
    #[error("Failed to deserialize data: {0}")]
    DeserializationFailed(serde_json::Error),
    /// An error occurred during the request.
    #[error("Request error: {0}")]
    RequestError(#[from] reqwest::Error),
}

struct FetchConfig {
    headers: reqwest::header::HeaderMap,
    timeout: Option<Duration>,
    // Add other configuration options as needed
}

/// Fetches a stream of data from the specified URL and deserializes it into chunks of type `T`.
///
/// # Arguments
///
/// * `url` - The URL to fetch the data from.
/// * `config` - The configuration options for the request.
///
/// # Returns
///
/// A `Result` containing a `TryStream` of `Chunk<T>` if the request is successful,
/// or a `FetchStreamError` if an error occurs.
async fn fetch_stream<T>(url: &str, config: FetchConfig) -> Result<impl TryStream<Ok = Chunk<T>, Error = FetchStreamError>, FetchStreamError>
where
    T: DeserializeOwned,
{
    let client = Client::new();
    let mut request = client.get(url);

    // Apply the configuration options
    request = request.headers(config.headers);
    if let Some(timeout) = config.timeout {
        request = request.timeout(timeout);
    }

    let response = request.send().await?;

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

    let config = FetchConfig {
        headers: reqwest::header::HeaderMap::new(),
        timeout: Some(Duration::from_secs(5)),
    };

    let user_stream = fetch_stream::<User>(USER_API_URL, config).await?;

    user_stream
        .try_for_each(|chunk| async move {
            println!("User: {:?}", chunk.data);
            Ok(())
        })
        .await?;

    Ok(())
}