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

| コマンド                | 内容                                                                                                               |
| ----------------------- | ------------------------------------------------------------------------------------------------------------------ |
| `mise run lint`         | 全言語の lint(Rust・TS・CSS・依存の向き・proto・Terraform・Dockerfile・シークレット)                               |
| `mise run gen:styles`   | utilities.css を変えたら、クラス名の型(utilities.gen.ts)を作り直す                                                 |
| `mise run test`         | Rust(nextest。DB 結合・API テストは testcontainers)とフロント(Vitest)                                              |
| `mise run deps:stop`    | 依存サービスを止める(データは残す)                                                                                 |
| `mise run e2e`          | 依存サービス起動・マイグレーション・server/Vite 起動・Playwright                                                   |
| `mise run e2e:video`    | 動作確認の動画(mp4)を e2e/videos-out/ に撮る。`-- --only-changed=origin/main` で PR の台本だけ(台本は e2e/videos/) |
| `mise run pr:video`     | この PR の台本で動画を撮り、今のブランチの PR に貼る(gh の `--attach`。初回は本文、撮り直しはコメント)             |
| `mise run sqlx-prepare` | マイグレーションか `query!` を変えたら、DB に当てて .sqlx/ を更新する                                              |
| `scripts/smoke-test.sh` | 起動中の server に grpcurl で主要シナリオを流す                                                                    |
| `mise run lambda-build` | payout-dispatcher の zip を作る                                                                                    |
| `mise run tf-plan`      | dev 環境の terraform plan                                                                                          |

非同期側(outbox → SQS → Lambda)をローカルで動かすには、server を起動した状態で
`cargo run -p payout-dispatcher --bin local_poller` を実行する(ElasticMQ をポーリングして Lambda と同じ処理を呼ぶ。振込の結果は `payouts` テーブルに残り、5回処理できなかったメッセージは `payroll-events-dlq.fifo` に移る)。
Lambda 本体は `cargo lambda watch -p payout-dispatcher` と
`cargo lambda invoke payout-dispatcher --data-file backend/app/lambdas/payout-dispatcher/events/sqs-payslip-finalized.json` で確認できる。

## worktree で並行開発する

worktree ごとに別の compose(DB・Keycloak なども別)と別のポートで動かせる。

```sh
mise run worktree:new -- feature-x     # ../banana-feature-x を作り(ブランチも)、スロットとポートをその worktree の .env に書く
cd ../banana-feature-x
mise run e2e                           # この worktree 専用の依存サービス・server・Vite で動く
mise run worktree:list                 # worktree ごとのスロットとポート
mise run worktree:remove -- feature-x  # worktree と compose(データも)を片付ける。ブランチは残す
```

- スロットは 1〜9(main は 0)。ポートは「既定値 + スロット × 100」(スロット 1 なら API :50151・画面 :5273・Keycloak :8180・MySQL :3406)。
- 名前はブランチ名。既にあるブランチならそれを checkout し、なければ作る。ディレクトリ名と compose のプロジェクト名では `/` などを `-` にし、小文字にする(`feature/X` → `banana-feature-x`)。
- 画面はその worktree の `WEB_PORT` で開く(スロット 1 なら http://localhost:5273)。
- 値(`COMPOSE_NAME` と各ポート、server の接続先の `DATABASE_URL`・`APP__*`)は worktree の `.env` にある。
  `docker compose` はプロジェクトの `.env` を自分で読み、mise も読んで環境変数で渡すので、そのディレクトリでそのまま使える。
- `target/` は worktree ごとに作られるので、初回の cargo ビルドには時間がかかる。
- `worktree:remove` は、そのディレクトリのブランチが指定の名前と一致し、コミットしていない変更がないときだけ進む(`feature/x` と `feature-x` は同じディレクトリ名になるので、取り違えて消さないため)。

## AWS に構築する

1. `infra/bootstrap/terraform.tfvars` と `infra/envs/*/terraform.tfvars` の仮の値(`000000000000`・`REPLACE_ME`・`example.com`)を実際の値にする。
2. 管理者の権限で `infra/bootstrap` を apply する(state の置き場・Lambda の zip の置き場・GitHub Actions のロール)。
   最初はローカルの state で apply し、`backend.tf` のコメントを外して `terraform init -migrate-state` で作ったバケットに移す。
3. GitHub の Environments を作る: `dev`・`stg`・`prod`(変数 `AWS_DEPLOY_ROLE_ARN`)と `dev-plan`・`stg-plan`・`prod-plan`(変数 `AWS_PLAN_ROLE_ARN`)。
   値は bootstrap の出力。`stg`・`prod` には承認者(Required reviewers)を付ける。
4. 各環境を apply し(最初の1回は管理者が手元から。以後は deploy ワークフロー)、振込 API のキーを Secrets Manager に入れる。
5. deploy ワークフローを実行する。migrate がテーブルと、アプリが接続する DB ユーザー(読み書きだけ)を作る。
   それまで server は DB に接続できない。

## 構成

```text
proto/acme/payroll/v1/        API 契約(buf)
backend/
  contexts/payroll/{domain,usecase,infrastructure,handler}   レイヤーごとの crate
  shared/{kernel,telemetry,auth}                            最小限の共通値・OTel・OIDC 検証
  gen/                                                      proto 生成コード
  app/{bootstrap,server,migrate,lambdas/payout-dispatcher}  組み立てと実行ファイル
frontend/apps/web                Vite + React + TanStack Router(routes/ が画面の URL。依存の向きは .dependency-cruiser.cjs)
frontend/packages/{api-client,ui}  ui は部品と CSS(styles/ の層: reset・tokens・base・utilities)
infra/bootstrap                  アカウントに1回だけ作るもの(state・成果物の置き場、GitHub Actions のロール)
infra/modules/stack              1つの環境の組み立て(環境ごとの差は規模と保護の強さだけ)
infra/modules/{network,database,backend-service,lambda-function,frontend-hosting,auth,messaging}
infra/envs/{dev,stg,prod}        modules/stack に値を渡すだけ
e2e/                             Playwright
```
