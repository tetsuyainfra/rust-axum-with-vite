use std::net::SocketAddr;

use axum::{
    body::Body,
    extract::{Request, State},
    http::HeaderValue,
    response::{Html, IntoResponse, Response},
    routing::get,
    Json, Router,
};

use axum::extract::connect_info::ConnectInfo;

use hyper::{header, Method, StatusCode};
use reqwest::Client;
use rust_embed::RustEmbed;
use serde_json::json;
use tokio::net::TcpListener;
use tower_http::{
    cors::CorsLayer,
    trace::{DefaultMakeSpan, TraceLayer},
};
use tracing::info;
use tracing_subscriber::EnvFilter;

#[derive(Debug, Clone)]
enum UiProxyMode {
    Embed,
    Proxy,
}

#[tokio::main]
async fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", format!("debug,hyper=info"))
    }
    let proxy_mode = match std::env::var("UI_PROXY_MODE") {
        Ok(val) => {
            if val.to_uppercase() == "PROXY" {
                UiProxyMode::Proxy
            } else {
                UiProxyMode::Embed
            }
        }
        Err(_) => UiProxyMode::Embed,
    };
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let app = Router::new()
        .route("/", get(root_handler))
        .route("/api/1", get(api_handler))
        .route("/ui", get(ui_proxy)) //  http://foobar/ui に対応させる時
        .route("/ui/", get(ui_proxy)) //  http://foobar/ui/ に対応させる時
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
        .with_state(proxy_mode)
        .into_make_service_with_connect_info::<SocketAddr>();

    let sock_addr = "127.0.0.1:3000".parse::<SocketAddr>().unwrap();

    println!("listening on http://{}", sock_addr);
    let tcp_listener = TcpListener::bind(sock_addr).await.unwrap();

    axum::serve(tcp_listener, app.clone()).await.unwrap();
}

async fn root_handler(
    State(mode): State<UiProxyMode>,
    ConnectInfo(remote_addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    Html(format!(
        r##"root from {remote_addr}<br />
            MODE: {mode:?}
            <a href="/ui">ui</a><br />
        "##
    ))
}
async fn api_handler(_req: Request) -> impl IntoResponse {
    Json(json!({ "data": 42 }))
}

// async fn ui_proxy(State(proxy_mode): State<UiProxyMode>, req: Request) -> impl IntoResponse {
//     match proxy_mode {
//         UiProxyMode::Embed => ui_proxy_embed(req).await.into_response(),
//         UiProxyMode::Proxy => ui_proxy_proxy(req).await.into_response(),
//     }
// }

async fn ui_proxy(State(proxy_mode): State<UiProxyMode>, req: Request) -> impl IntoResponse {
    // async fn ui_proxy_proxy(req: Request) -> impl IntoResponse {
    let mut path = req.uri().path().trim_start_matches('/').to_string();
    if path == "ui" {
        // "ui" -> ""
        path = path.replace("ui", "");
    } else if path.starts_with("ui/") {
        // "ui/" -> ""
        // "ui/index.html" -> "index.html"
        path = path.replace("ui/", "");
    }

    match proxy_mode {
        UiProxyMode::Embed => StaticFile(path).into_response(),
        UiProxyMode::Proxy => {
            // include_dir!("")
            let url = format!("http://127.0.0.1:5173/ui/{path}");
            info!(?url);

            let client = Client::new();
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
                .into_response()
        }
    }
}

#[derive(RustEmbed)]
#[folder = "client/dist/"]
struct Asset;

pub struct StaticFile<T>(pub T);

impl<T> IntoResponse for StaticFile<T>
where
    T: Into<String>,
{
    fn into_response(self) -> Response {
        let path = String::from(self.0.into());
        info!("StaticFile request -> '{path}'");

        let req_path = match path.as_str() {
            "" => "index.html",
            _ => &path,
        };

        match Asset::get(req_path) {
            Some(content) => {
                let mime = mime_guess::from_path(req_path).first_or_octet_stream();
                info!("StaticFile mime -> {mime:?}");
                ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
            }
            None => (StatusCode::NOT_FOUND, "404 Not Found").into_response(),
        }
    }
}
