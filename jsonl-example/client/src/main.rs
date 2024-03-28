use futures::StreamExt;
use reqwest::Client;
use serde::Deserialize;
use std::error::Error;
use futures::stream::{self, BoxStream};

#[derive(Deserialize, Debug)]
struct User {
    id: u32,
    name: String,
}

async fn fetch_users(client: &Client, url: &str) -> Result<BoxStream<'static, Result<User, Box<dyn Error>>>, Box<dyn Error>> {
    let response = client.get(url).send().await?.bytes_stream();

    let user_stream = stream::unfold(response, |mut response| async {
        match response.next().await {
            Some(Ok(chunk)) => {
                let json_str = match std::str::from_utf8(&chunk) {
                    Ok(s) => s,
                    Err(e) => return Some((Err(Box::new(e) as Box<dyn Error>), response)),
                };
                match serde_json::from_str::<User>(json_str) {
                    Ok(user) => Some((Ok(user), response)),
                    Err(e) => Some((Err(Box::new(e) as Box<dyn Error>), response)),
                }
            },
            Some(Err(e)) => Some((Err(Box::new(e) as Box<dyn Error>), response)),
            None => None, // End of stream
        }
    }).boxed();

    Ok(user_stream)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = Client::new();
    let url = "http://127.0.0.1:8080/users";

    let user_stream = fetch_users(&client, url).await?;

    user_stream.for_each(|user_result| async {
        match user_result {
            Ok(user) => println!("User: {:?}", user),
            Err(e) => eprintln!("Error: {}", e),
        }
    }).await;

    Ok(())
}