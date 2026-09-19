# platform

Rust(バックエンド)・React(フロントエンド)・Terraform(インフラ)・proto(API 契約)を1つのリポジトリで管理する。
構成の考え方は「バックエンド構成ガイド(Rust × React × Terraform)」に従い、ガイドとの差分と検証結果は
[docs/guide-verification.md](docs/guide-verification.md) にまとめてある。

## セットアップ

前提: mise・rustup・Docker と、C のビルドツール(Ubuntu なら `sudo apt-get install -y build-essential pkg-config`)。
E2E を画面で見る(`mise run e2e:ui` など)なら日本語フォントも入れる(Ubuntu なら `sudo apt-get install -y fonts-noto-cjk`。ないとブラウザの日本語が文字化けする)。

```sh
# rustup・mise をシェルから使えるようにする(インストーラが設定しなかった場合)
echo '. "$HOME/.cargo/env"' >> ~/.bashrc
echo 'eval "$(~/.local/bin/mise activate bash)"' >> ~/.bashrc && exec bash

rustup default "$(sed -n 's/^channel = "\(.*\)"/\1/p' rust-toolchain.toml)"  # リポジトリ外で動く cargo(sqlx-cli のビルド)用
rustup toolchain install          # rust-toolchain.toml の版を先に入れる(mise install の並列ビルドで rustup が競合しないように)
mise trust && mise install        # ツール一式(sqlx-cli はソースビルドで数分かかる)
pnpm install
lefthook install
cp .env.example .env
docker compose up -d --wait       # MySQL・Keycloak・ElasticMQ・SeaweedFS・Jaeger
mise run gen                      # proto → Rust・TS
mise run migrate                  # 空の DB からでも通る(query! は .sqlx/ のキャッシュでコンパイルする)
mise run dev-backend              # gRPC(+ gRPC-Web)サーバー :50051
mise run dev-frontend             # Vite :5173
```

ブラウザで http://localhost:5173 を開き、`admin@example.com` / `password` でログインする。

## よく使うコマンド

| コマンド                | 内容                                                                  |
| ----------------------- | --------------------------------------------------------------------- |
| `mise run lint`         | 全言語の lint(Rust・TS・proto・Terraform・Dockerfile・シークレット)   |
| `mise run test`         | Rust(nextest。DB 結合・API テストは testcontainers)とフロント(Vitest) |
| `mise run deps:stop`    | 依存サービスを止める(データは残す)                                    |
| `mise run e2e`          | 依存サービス起動・マイグレーション・server/Vite 起動・Playwright      |
| `mise run sqlx-prepare` | `query!` を変えたら .sqlx/ を更新する                                 |
| `scripts/smoke-test.sh` | 起動中の server に grpcurl で主要シナリオを流す                       |
| `mise run lambda-build` | payout-dispatcher の zip を作る                                       |
| `mise run tf-plan`      | dev 環境の terraform plan                                             |

非同期側(outbox → SQS → Lambda)をローカルで動かすには、server を起動した状態で
`cargo run -p payout-dispatcher --bin local_poller` を実行する(ElasticMQ をポーリングして Lambda と同じ処理を呼ぶ)。
Lambda 本体は `cargo lambda watch -p payout-dispatcher` と
`cargo lambda invoke payout-dispatcher --data-file backend/app/lambdas/payout-dispatcher/events/sqs-payslip-finalized.json` で確認できる。

## worktree で並行開発する

worktree ごとに別の compose(DB・Keycloak なども別)と別のポートで動かせる。

```sh
mise run worktree:new -- feature-x     # ../banana-feature-x を作り(ブランチも)、スロットとポートを .env.worktree に書く
cd ../banana-feature-x
mise run e2e                           # この worktree 専用の依存サービス・server・Vite で動く
mise run worktree:list                 # worktree ごとのスロットとポート
mise run worktree:remove -- feature-x  # compose(データも)と worktree を片付ける。ブランチは残す
```

- スロットは 1〜9(main は 0)。ポートは「既定値 + スロット × 100」(スロット 1 なら API :50151・画面 :5273・Keycloak :8180・MySQL :3406)。
- mise が `.env.worktree` を読み、`COMPOSE_NAME` と各ポート、server の接続先(`DATABASE_URL`・`APP__*`)を環境変数で渡す。
  `docker compose` も mise を有効にしたシェル(または `mise exec --`)から実行する。そうしないと main の compose を操作してしまう。
- `target/` は worktree ごとに作られるので、初回の cargo ビルドには時間がかかる。

## 構成

```text
proto/acme/payroll/v1/        API 契約(buf)
backend/
  contexts/payroll/{domain,usecase,infrastructure,handler}   レイヤーごとの crate
  shared/{kernel,telemetry,auth}                            最小限の共通値・OTel・OIDC 検証
  gen/                                                      proto 生成コード
  app/{bootstrap,server,migrate,lambdas/payout-dispatcher}  組み立てと実行ファイル
frontend/apps/web                Vite + React(features/ 間の import は oxlint で禁止)
frontend/packages/{api-client,ui}
infra/modules/{network,database,backend-service,lambda-function,frontend-hosting,auth,messaging}
infra/envs/{dev,stg,prod}
e2e/                             Playwright
```
