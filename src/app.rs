use axum::{
    extract::{self, State},
    response::IntoResponse,
    routing::{get, post},
    serve::Serve,
    http::{header, StatusCode},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use log::info;
use std::sync::Arc;
use std::collections::HashMap;

use crate::db::{DB, SearchResult};

struct AppState {
    db: DB
}

pub async fn serve() {
    info!("Starting server...");
    
    let shared_state = Arc::new(AppState{db: DB::new("archive", "archive", "localhost:5433", "archive_db").await});

    let app = Router::new()
        .route("/", get(homepage))
        .route("/ping", get(pong))
        .route("/search", get(search))
        .route("/api/search", get(api_search))
        .fallback(handler_404)
        .with_state(shared_state);

    // TODO: customisable port

    let listener = tokio::net::TcpListener::bind("0.0.0.0:22001").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn homepage() -> impl IntoResponse{
    (StatusCode::OK, [
            (header::CONTENT_TYPE, "text/html; charset=utf-8")
        ], 
        String::from(r#"
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

async fn search(State(state): State<Arc<AppState>>, extract::Query(query): extract::Query<HashMap<String, String>>) -> impl IntoResponse {
    
    let mut default_page = String::from(r#"
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

async fn api_search(State(state): State<Arc<AppState>>, extract::Query(query): extract::Query<HashMap<String, String>>) -> Json<Vec<SearchResult>> {
    if query.contains_key("q") {
        Json(state.db.search(query.get("q").unwrap().as_str()).await.unwrap())
    } else {
        Json(Vec::<SearchResult>::new())
    }
}

async fn pong() -> &'static str {
    "pong!"
}

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "404: nothing to see here")
}
