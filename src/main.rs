use std::net::SocketAddr;

use axum::{
    body::Body,
    extract::{ws::WebSocketUpgrade, Request, State},
    http::{request, HeaderValue},
    response::{Html, IntoResponse, Response},
    routing::get,
    Json, Router,
};

use axum::extract::connect_info::ConnectInfo;
use axum::extract::ws::CloseFrame;

use axum_extra::{headers, TypedHeader};
use hyper::{Method, StatusCode};
use reqwest::Client;
use serde_json::json;
use tokio::net::TcpListener;
use tower_http::{
    cors::CorsLayer,
    trace::{DefaultMakeSpan, TraceLayer},
};
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", format!("debug,hyper=info"))
    }
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let client = Client::new();

    let app = Router::new()
        .route("/", get(root_handler))
        .route("/api/1", get(api_handler))
        .route("/ui", get(ui_proxy))
        .route("/ui/*wild", get(ui_proxy))
        .layer(
            CorsLayer::new()
                .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
                .allow_origin("http://127.0.0.1:3000".parse::<HeaderValue>().unwrap())
                .allow_methods([Method::GET]),
        )
        // .layer(TraceLayer::new_for_http())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        )
        .with_state(client)
        .into_make_service_with_connect_info::<SocketAddr>();

    let sock_addr = "127.0.0.1:3000".parse::<SocketAddr>().unwrap();

    println!("listening on http://{}", sock_addr);
    let tcp_listener = TcpListener::bind(sock_addr).await.unwrap();

    axum::serve(tcp_listener, app.clone()).await.unwrap();
}

async fn root_handler(ConnectInfo(remote_addr): ConnectInfo<SocketAddr>) -> impl IntoResponse {
    Html(format!(
        r##"root from {remote_addr}<br />
            <a href="/ui">ui</a>"##
    ))
}
async fn api_handler(_req: Request) -> impl IntoResponse {
    Json(json!({ "data": 42 }))
}

async fn ui_proxy(State(client): State<Client>, req: Request) -> impl IntoResponse {
    // include_dir!("")
    let uri = req.uri();
    let url = format!("http://127.0.0.1:5173{uri}");
    info!(?url);

    let reqwest_response = match client.get(url).send().await {
        Ok(res) => res,
        Err(err) => {
            tracing::error!(%err, "request failed");
            return StatusCode::BAD_GATEWAY.into_response();
        }
    };

    let mut response_builder = Response::builder().status(reqwest_response.status());

    *response_builder.headers_mut().unwrap() = reqwest_response.headers().clone();

    response_builder
        // .header("Permissions-Policy", "geolocation=*") // 必要なさそう
        .body(Body::from_stream(reqwest_response.bytes_stream()))
        // Same goes for this unwrap
        .unwrap()
}
