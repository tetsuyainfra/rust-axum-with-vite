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
cargo add tracing
cargo add tracing tracing-subscriber --features=env-filter

# build

```
CROSS_CONTAINER_ENGINE=podman cross run --target x86_64-pc-windows-gnu
```

## 覚書

- build.rs が毎回走るのでこのままだと npm run build が都度実行される
  境変変数で ON/OFF するか、なにかファイルを見て実行制御したいところ

- npm がないよー
