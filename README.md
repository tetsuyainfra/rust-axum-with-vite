# structure

- server
- frontend(vite)

# setup server

```
# build
cargo build

# for windows
cp Cross.toml{.example,}
cross build


# release
cargo build --profile release
cross build --profile release
```

## 覚書

- build.rs が毎回走るのでこのままだと npm run build が都度実行される
  境変変数で ON/OFF するか、なにかファイルを見て実行制御したいところ

- npm がないよー

- Docker build で proxy を渡す
  docker build -f ./Docker-custom.file . --build-arg http_proxy=http://172.16.0.1:3128
