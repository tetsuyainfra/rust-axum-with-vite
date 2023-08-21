use std::net::{IpAddr, SocketAddr};

use axum::{
    extract::ConnectInfo,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(root_handler))
        .into_make_service_with_connect_info::<SocketAddr>();

    let sock_addr = "127.0.0.1:3000".parse::<SocketAddr>().unwrap();

    println!("listening on http://{}", sock_addr);
    let tcp_listener = TcpListener::bind(sock_addr).await.unwrap();

    axum::serve(tcp_listener, app.clone()).await.unwrap();
}

async fn root_handler(ConnectInfo(remote_addr): ConnectInfo<SocketAddr>) -> impl IntoResponse {
    Html(format!("root from {remote_addr}"))
}
