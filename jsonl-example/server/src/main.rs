use actix_web::{web, App, HttpResponse, HttpServer};
use futures::stream::StreamExt;
use serde::Serialize;

#[derive(Serialize)]
struct User {
    id: u32,
    name: String,
}

#[derive(Serialize)]
struct Done {
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
    let user_stream = futures::stream::iter(users)
        .map(|user| {
            let json = serde_json::to_string(&user).unwrap();
            // Wait 1 second before sending the next user
            std::thread::sleep(std::time::Duration::from_secs(1));
            Ok::<_, actix_web::error::Error>(actix_web::web::Bytes::from(format!("{}\n", json)))
        });

    // Create a stream for the Done object
    let done_stream = futures::stream::once(async {
        let done = Done { done: true };
        let json = serde_json::to_string(&done).unwrap();
        Ok::<_, actix_web::error::Error>(actix_web::web::Bytes::from(format!("{}\n", json)))
    });

    // Concatenate the user stream with the Done stream
    let stream = user_stream.chain(done_stream);

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
        let mut app = test::init_service(
            App::new().route("/users", web::get().to(stream_users))
        ).await;

        let req = test::TestRequest::get().uri("/users").to_request();

        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
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
        ];

        for user in expected_users {
            let json = serde_json::to_string(&user).unwrap();
            assert!(body_str.contains(&format!("{}\n", json)));
        }

        let done = Done { done: true };
        let json = serde_json::to_string(&done).unwrap();
        assert!(body_str.contains(&format!("{}\n", json)));
    }

    
    #[actix_rt::test]
    async fn test_server_setup() {
        use super::*;
        use actix_web::{test, App};

        let mut app = test::init_service(
            App::new().route("/users", web::get().to(stream_users))
        ).await;

        let req = test::TestRequest::get().uri("/users").to_request();

        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
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
        ];

        for user in expected_users {
            let json = serde_json::to_string(&user).unwrap();
            assert!(body_str.contains(&format!("{}\n", json)));
        }

        let done = Done { done: true };
        let json = serde_json::to_string(&done).unwrap();
        assert!(body_str.contains(&format!("{}\n", json)));
    }
}