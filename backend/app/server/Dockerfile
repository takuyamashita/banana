# syntax=docker/dockerfile:1
# ビルドはリポジトリ直下をコンテキストにする: docker build -f backend/app/server/Dockerfile .
FROM lukemathwalker/cargo-chef:0.1.78-rust-1.98.1-slim-trixie AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
# query! は .sqlx/ のオフラインキャッシュでコンパイルする(ビルド時に DB は不要)
ENV SQLX_OFFLINE=true
COPY --from=planner /app/recipe.json recipe.json
COPY rust-toolchain.toml .
# 依存だけ先にビルドしてレイヤーキャッシュに乗せる
RUN cargo chef cook --release --recipe-path recipe.json -p server -p migrate
COPY . .
# migrate も同じイメージに入れ、ECS の単発タスクでは entryPoint を /app/migrate に差し替える
RUN cargo build --release -p server -p migrate --locked

FROM gcr.io/distroless/cc-debian13:nonroot AS runtime
WORKDIR /app
# 設定(config/*.toml)はバイナリに埋め込んであるので、ファイルは置かない
COPY --from=builder /app/target/release/server /app/target/release/migrate /app/
EXPOSE 50051
# distroless の nonroot ユーザー(数値で指定するとホスト側でも解決できる)
USER 65532:65532
ENTRYPOINT ["/app/server"]
