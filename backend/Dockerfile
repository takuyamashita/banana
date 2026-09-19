# syntax=docker/dockerfile:1
# サービスごとのイメージ(<サービス>-server と <サービス>-migrate)を作る。SERVICE でサービスを選ぶ。
# ビルドはリポジトリ直下をコンテキストにする:
#   docker build --platform linux/arm64 -f backend/Dockerfile --build-arg SERVICE=payroll .
# (ECS のタスクは ARM64。ベースイメージはタグの付け替えで中身が変わらないようダイジェストで固定し、Renovate が更新する)
FROM lukemathwalker/cargo-chef:0.1.78-rust-1.98.1-slim-trixie@sha256:38dfdbf4fda95c516f873f33032e490baa988b75f7d83c7d12f788f770785b36 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
ARG SERVICE
# query! は .sqlx/ のオフラインキャッシュでコンパイルする(ビルド時に DB は不要)
ENV SQLX_OFFLINE=true
COPY --from=planner /app/recipe.json recipe.json
COPY rust-toolchain.toml .
# 依存だけ先にビルドしてレイヤーキャッシュに乗せる
RUN cargo chef cook --release --recipe-path recipe.json -p "${SERVICE}-server" -p "${SERVICE}-migrate"
COPY . .
# migrate も同じイメージに入れ、ECS の単発タスクでは entryPoint を /app/migrate に差し替える。
# どのサービスのイメージでも、中の名前は /app/server と /app/migrate にそろえる(タスク定義を同じ形にする)
RUN cargo build --release -p "${SERVICE}-server" -p "${SERVICE}-migrate" --locked \
  && mkdir /out \
  && cp "target/release/${SERVICE}-server" /out/server \
  && cp "target/release/${SERVICE}-migrate" /out/migrate

FROM gcr.io/distroless/cc-debian13:nonroot@sha256:54df941ed0d06a1bd95ef5e0ce391fd8d9f94b64782dc9a60062727849ee3f97 AS runtime
WORKDIR /app
# 設定(config/*.toml)はバイナリに埋め込んであるので、ファイルは置かない
COPY --from=builder /out/server /out/migrate /app/
EXPOSE 50051
# distroless の nonroot ユーザー(数値で指定するとホスト側でも解決できる)
USER 65532:65532
ENTRYPOINT ["/app/server"]
