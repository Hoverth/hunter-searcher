use axum::{
    routing::{get, post},
    serve::Serve,
    http::StatusCode,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use log::info;

pub async fn serve() {
    info!("Starting server...");
    
    // TODO: document serve & api endpoints for database queries

    let app = Router::new()
        .route("/ping", get(pong));

    // TODO: customisable port

    let listener = tokio::net::TcpListener::bind("0.0.0.0:22001").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn pong() -> &'static str {
    "pong!"
}
