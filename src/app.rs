use axum::{
    extract::{self, State},
    response::IntoResponse,
    routing::get,
    http::{header, StatusCode},
    Json, Router,
};
use tower_http::compression::CompressionLayer;
use log::info;
use std::sync::Arc;
use std::collections::HashMap;

use crate::db::{DB, SearchResult};

/// State struct to hold the database for the axum server
struct AppState {
    db: DB
}

/// Start the server with a DB connection
pub async fn serve(db: DB) {
    info!("Starting server...");
    
    let shared_state = Arc::new(AppState{db});

    let app = Router::new()
        .route("/", get(homepage))
        .route("/ping", get(pong))
        .route("/search", get(search))
        .route("/api/search", get(api_search))
        .layer(CompressionLayer::new())
        .fallback(handler_404)
        .with_state(shared_state);

    // TODO: customisable port

    let listener = tokio::net::TcpListener::bind("0.0.0.0:22001").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

/// Returns the home page response
async fn homepage() -> impl IntoResponse{
    (StatusCode::OK, [
            (header::CONTENT_TYPE, "text/html; charset=utf-8")
        ], 
        String::from(r#"<!DOCTYPE html>
<html>
    <head>
        <title>Hunter-Searcher</title>
    </head>
    <body>
        <center>
            <h1>Hunter-Searcher</h1>
            <form action="/search">
                <input type="text" name="q">
                <input type="submit">
            </form>
        </center>
    </body>
</html>
"#))
}

/// Returns the search results response
async fn search(State(state): State<Arc<AppState>>, extract::Query(query): extract::Query<HashMap<String, String>>) -> impl IntoResponse {
    
    let mut default_page = String::from(r#"<!DOCTYPE html>
<html>
    <head>
        <title>Hunter-Searcher</title>
    </head>
    <body>
        <center>
            <h1>Hunter-Searcher</h1>
            <form action="/search">
                <input type="text" name="q">
                <input type="submit">
            </form>
        </center>
        <!---->
    </body>
</html>"#);
    if query.contains_key("q") {
        default_page = default_page.replace("name=\"q\"", format!("name=\"q\" value=\"{}\"", &query.get("q").unwrap()).as_str());

        //TODO check for bangs
        
        let mut search_items = String::new();
        for res in state.db.search(query.get("q").unwrap().as_str()).await.unwrap() {
        search_items += format!(r#"
            <a href="{1}"><b>{0}</b>
            <i>{1}</i><br>
            {2}</a><br><br>
            "#, res.title, res.url, res.blurb.unwrap_or(String::new())).as_str();
        }
        default_page = default_page.replace("<!---->", search_items.as_str());
    }
    (StatusCode::OK, [(header::CONTENT_TYPE, "text/html; charset=utf-8")], default_page)
}

/// API endpoint for search results
async fn api_search(State(state): State<Arc<AppState>>, extract::Query(query): extract::Query<HashMap<String, String>>) -> Json<Vec<SearchResult>> {
    if query.contains_key("q") {
        Json(state.db.search(query.get("q").unwrap().as_str()).await.unwrap())
    } else {
        Json(Vec::<SearchResult>::new())
    }
}

/// Simple ping response
async fn pong() -> &'static str {
    "pong!"
}

/// Simple 404 response
async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "404: nothing to see here")
}
