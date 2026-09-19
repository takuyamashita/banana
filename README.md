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
mise run migrate                  # 給与・勤怠の DB に流す(query! は .sqlx/ のキャッシュでコンパイルする)
mise run dev-backend              # gRPC(+ gRPC-Web)サーバー: 給与 :50051・勤怠 :50052
mise run dev-frontend             # Vite :5173
```

ブラウザで http://localhost:5173 を開き、`admin@example.com` / `password` でログインする。

勤怠(timesheet)サービスを足す前の版から更新したときは、`mise run db:reset` でローカルの DB を作り直す
(データベースがサービスごと(`payroll`・`timesheet`)になり、初回の起動時にだけ作られるため)。

## よく使うコマンド

| コマンド                | 内容                                                                                                                        |
| ----------------------- | --------------------------------------------------------------------------------------------------------------------------- |
| `mise run lint`         | 全言語の lint(Rust・TS・CSS・依存の向き・proto・Terraform・Dockerfile・シークレット)                                        |
| `mise run gen:styles`   | utilities.css を変えたら、クラス名の型(utilities.gen.ts)を作り直す                                                          |
| `mise run test`         | Rust(nextest。DB 結合・API テストは testcontainers)とフロント(Vitest)                                                       |
| `mise run deps:stop`    | 依存サービスを止める(データは残す)                                                                                          |
| `mise run e2e`          | 依存サービス起動・マイグレーション・server/Vite 起動・Playwright                                                            |
| `mise run e2e:video`    | 動作確認の動画(mp4)を e2e/videos-out/ に撮る(台本は e2e/videos/。`-- e2e/videos/pr/12/` で指定したものだけ)                 |
| `mise run pr:video`     | 今のブランチの PR の台本(e2e/videos/pr/<PR 番号>/)で動画を撮り、PR に貼る(gh の `--attach`。初回は本文、撮り直しはコメント) |
| `mise run sqlx-prepare` | マイグレーションか `query!` を変えたら、DB に当てて .sqlx/ を更新する                                                       |
| `mise run smoke`        | 起動中の server(給与・勤怠)に grpcurl で主要シナリオと、サービスをまたぐ出来事の流れを流す                                  |
| `mise run lambda-build` | payout-dispatcher の zip を作る                                                                                             |
| `mise run tf-plan`      | dev 環境の terraform plan                                                                                                   |

サービスの間の出来事(給与 ⇄ 勤怠)は、server(`mise run dev-backend`)が自分のキューを読むので、両方を起動すれば流れる。
ローカルには SNS がないので、relay が受け手のキュー(ElasticMQ)へ直接送る(`config/<サービス>/local.toml` の `publish_queue_urls`)。

振込(給与明細の確定 → Lambda)をローカルで動かすには、server を起動した状態で
`cargo run -p payout-dispatcher --bin local_poller` を実行する(ElasticMQ をポーリングして Lambda と同じ処理を呼ぶ。振込の結果は `payouts` テーブルに残り、5回処理できなかったメッセージは `payroll-payout-dlq.fifo` に移る)。
Lambda 本体は `cargo lambda watch -p payout-dispatcher` と
`cargo lambda invoke payout-dispatcher --data-file backend/services/payroll/payout-dispatcher/events/sqs-payslip-finalized.json` で確認できる。

## worktree で並行開発する

worktree ごとに別の compose(DB・Keycloak なども別)と別のポートで動かせる。

```sh
mise run worktree:new -- feature-x     # ../banana-feature-x を作り(ブランチも)、スロットとポートをその worktree の .env に書く
cd ../banana-feature-x
mise run e2e                           # この worktree 専用の依存サービス・server・Vite で動く
mise run worktree:list                 # worktree ごとのスロットとポート
mise run worktree:remove -- feature-x  # worktree と compose(データも)を片付ける。ブランチは残す
```

- スロットは 1〜9(main は 0)。ポートは「既定値 + スロット × 100」(スロット 1 なら給与 :50151・勤怠 :50152・画面 :5273・Keycloak :8180・MySQL :3406)。
- 名前はブランチ名。既にあるブランチならそれを checkout し、なければ作る。ディレクトリ名と compose のプロジェクト名では `/` などを `-` にし、小文字にする(`feature/X` → `banana-feature-x`)。
- 画面はその worktree の `WEB_PORT` で開く(スロット 1 なら http://localhost:5273)。
- 値(`COMPOSE_NAME` と各ポート、server の接続先の `PAYROLL__*`・`TIMESHEET__*`)は worktree の `.env` にある。
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
5. deploy ワークフローを実行する。サービスごとの migrate が、そのサービスのデータベース・テーブルと、
   アプリが接続する DB ユーザー(そのデータベースの読み書きだけ)を作る。それまで server は DB に接続できない。

## 構成

```text
proto/acme/{payroll,timesheet}/v1/          サービスの API 契約(buf)
proto/acme/{payroll,timesheet}/events/v1/   サービスの間の出来事の約束
backend/
  contexts/{payroll,timesheet}/{domain,usecase,infrastructure,handler}   コンテキストごと・レイヤーごとの crate
  shared/{kernel,telemetry,auth}                                  最小限の共通値・OTel・OIDC 検証と認証の入口
  shared/{db,messaging,service}                                   DB 接続・出来事の送受信・サービスの起動と停止
  gen/                                                            proto 生成コード
  services/payroll/{bootstrap,server,migrate,payout-dispatcher}   給与サービスの組み立てと実行ファイル
  services/timesheet/{bootstrap,server,migrate}                   勤怠サービスの組み立てと実行ファイル
  Dockerfile                                                      サービスごとのイメージ(SERVICE で選ぶ)
config/{payroll,timesheet}/      サービスごとの設定(環境変数 PAYROLL__…・TIMESHEET__… で上書き)
frontend/apps/web                Vite + React + TanStack Router(routes/ が画面の URL。依存の向きは .dependency-cruiser.cjs)
frontend/packages/{api-client,ui}  api-client はサービスごとの宛先に振り分ける transport。ui は部品と CSS
infra/bootstrap                  アカウントに1回だけ作るもの(state・成果物の置き場、GitHub Actions のロール)
infra/modules/stack              1つの環境の組み立て(環境ごとの差は規模と保護の強さだけ)
infra/modules/{network,database,load-balancer,backend-service,event-topic,event-queue,lambda-function,frontend-hosting,auth}
infra/envs/{dev,stg,prod}        modules/stack に値を渡すだけ
e2e/                             Playwright
```
