
# structure
- server
- frontend(vite)

# setup server 

```
cargo add axum --git=https://github.com/tokio-rs/axum --branch=main
cargo add tracing tracing-subscriber
```
cargo add tokio --features full
cargo add hyper@1.0.0-rc.4 --features=full
cargo add tower
cargo add tower-http --features=cors,fs,trace
