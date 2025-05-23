use axum::{
    response::Response,
    routing, Router,
};
use std::{sync::Arc};
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;


mod get;
mod ring;

#[tokio::main]
async fn main() {
    let ring = ring::Ring::new(include_str!("../members.toml"));

    let ring = Arc::new(RwLock::new(ring));

    let ring_clone = ring.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(60 * 60)).await;

            let mut ring = ring_clone.write().await;

            ring.shuffle();
        }
    });
    
    // serve static files from /public
    let landing_page = ServeDir::new("public")
    .append_index_html_on_directories(true);

    let app = Router::new()
        .route("/health", routing::get(health))
        .route("/all", routing::get(get::all))
        .route("/:id", routing::get(get::one))
        .route("/:id/prev", routing::get(get::prev))
        .route("/:id/next", routing::get(get::next))
        .route("/ring.js", routing::get(get::ring_js))
        .fallback_service(landing_page)
        .layer(CorsLayer::permissive())
        .with_state(ring);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("Listening on: {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}

async fn health() -> Response<String> {
    let commit = std::env::var("COMMIT").unwrap_or("unknown".to_string());

    Response::builder()
        .header("Content-Type", "text/plain")
        .body(format!("OK\n{}", commit))
        .unwrap()
}
