use actix_web::{web, App, HttpResponse, HttpServer};
use futures::stream::StreamExt;
use serde::Serialize;

#[derive(Serialize)]
struct User {
    id: u32,
    name: String,
}

#[derive(Serialize)]
struct Chunk {
    data: User,
    done: bool,
}

async fn stream_users() -> HttpResponse {
    // Create a vector of users
    let users = vec![
        User {
            id: 1,
            name: "Alice".to_string(),
        },
        User {
            id: 2,
            name: "Bob".to_string(),
        },
        User {
            id: 3,
            name: "Charlie".to_string(),
        },
        User {
            id: 4,
            name: "David".to_string(),
        },
        // Add more users as needed
    ];

    // Create a stream from the vector and map each user to a JSON string
    let stream = futures::stream::iter(users)
        .map(|user| {
            let is_last = user.id == 4;
            let chunk = Chunk {
                data: user,
                done: is_last,
            };
            let json = serde_json::to_string(&chunk).unwrap();
            Ok::<_, actix_web::error::Error>(actix_web::web::Bytes::from(format!("{}\n", json)))
        });



    HttpResponse::Ok()
        .content_type("application/x-ndjson")
        .streaming(Box::pin(stream))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let url_server = "127.0.0.1:8080";

    println!("Server running at http://{}", url_server);
    
    HttpServer::new(|| {
        App::new().route("/users", web::get().to(stream_users))
    })
    .bind(url_server)?
    .run()
    .await
}


#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};

    #[actix_rt::test]
    async fn test_stream_users() {
        let resp = stream_users().await;
        assert!(resp.status().is_success());

        let body = actix_web::web::Bytes::from(actix_web::web::block_on(resp.into_body().map_err(|_| ()).fold(Vec::new(), |mut v, chunk| {
            v.extend_from_slice(&chunk);
            futures::future::ready(Ok::<_, ()>(v))
        })).unwrap());

        let body_str = std::str::from_utf8(&body).unwrap();

        let expected_users = vec![
            User {
                id: 1,
                name: "Alice".to_string(),
            },
            User {
                id: 2,
                name: "Bob".to_string(),
            },
            User {
                id: 3,
                name: "Charlie".to_string(),
            },
            User {
                id: 4,
                name: "David".to_string(),
            },
        ];

        for user in expected_users {
            let is_last = user.id == 4;
            let chunk = Chunk {
                data: user,
                done: is_last,
            };
            let json = serde_json::to_string(&chunk).unwrap();
            assert!(body_str.contains(&format!("{}\n", json)));
        }
    }

    
    #[actix_rt::test]
    async fn test_server_setup() {
        let mut app = test::init_service(App::new().route("/users", web::get().to(stream_users)).await).await;
        let req = test::TestRequest::get().uri("/users").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success());


    }
}