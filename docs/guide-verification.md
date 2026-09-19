# 構成ガイド検証ログ

「バックエンド構成ガイド(Rust × React × Terraform)」をこのリポジトリに実装し、記述どおりに動くかを確かめた記録。
環境: WSL2 Ubuntu 24.04 / Docker 28.0.4 / 2026-09-19 時点の最新版ツール。
下表の「対応」は 2026-09-19 にガイド本体(構成ガイド・実装ガイドの両タブ)へ反映済み。

| #   | ガイドの箇所                               | 記述                                                                                                  | 実際                                                                                                                                                                                                                                                               | 対応                                                                                                                                                                                                                                                  |
| --- | ------------------------------------------ | ----------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | セットアップ手順                           | 前提は mise・rustup・Docker のみ                                                                      | Rust のビルドに C コンパイラとリンカ(`cc`)が要る。素の Ubuntu には入っていない                                                                                                                                                                                     | 前提に `build-essential`・`pkg-config` を追加する                                                                                                                                                                                                     |
| 2   | 開発環境とタスク(compose.yaml)             | ElasticMQ の設定を `/etc/elasticmq.conf` にマウント                                                   | `elasticmq-native` イメージは `/opt/elasticmq.conf` を読む。ガイドどおりだとキューが1つも作られない(ListQueues が空になることを確認)                                                                                                                               | マウント先を `/opt/elasticmq.conf` にする                                                                                                                                                                                                             |
| 3   | 開発環境とタスク(elasticmq.conf)           | `payroll-events` を標準キューで定義                                                                   | relay が付ける `MessageDeduplicationId` は FIFO 専用で、標準キューでは `InvalidParameterValue` になり送信できない(確認済み)。`MessageGroupId` は標準キューでも受け付けられるが、順序は保証されない                                                                 | `"payroll-events.fifo" { fifo = true }` にする。AWS 側も FIFO キュー(名前は `.fifo` で終わる)                                                                                                                                                         |
| 4   | 開発環境とタスク(mise)                     | cargo 系ツールは mise から cargo-binstall 経由で入れる                                                | sqlx-cli にはビルド済みバイナリがなくソースビルドになる。その際 (a) rustup に既定ツールチェーンが要る(リポジトリ外で cargo が動くため、rust-toolchain.toml だけでは足りない)(b) 既定機能は native-tls なので `libssl-dev` が要る                                   | 手順に `rustup default <版>` を加える。mise.toml では `"cargo:sqlx-cli" = { version, default-features = false, features = "mysql,rustls" }` にして OpenSSL を不要にする                                                                               |
| 5   | 認証(クレームの差)                         | Keycloak は `realm_access.roles` で標準的な `aud`                                                     | ロールは記述どおり。ただし `aud` は既定だと `account` で、API を表す値は入らない。そのままでは aud 検証ができない                                                                                                                                                  | クライアントに `oidc-audience-mapper` を付け、`aud` に API 識別子(例: `platform-api`)を入れる。mapper で検証する aud をプロバイダ別に持つ                                                                                                             |
| 6   | 実装ガイド(KeycloakUserDirectory)          | 管理用トークンは master realm の client_credentials で取得                                            | master realm のクライアントは realm import(`--import-realm`)では作れない                                                                                                                                                                                           | 対象 realm にサービスアカウント付きクライアントを置き、`realm-management` の `manage-users`・`view-users` を付与する。import だけで完結し、200 を確認済み                                                                                             |
| 7   | 実装ガイド(repository専用APIの守り方)      | usecase・handler の clippy.toml に `disallowed-methods` を書く                                        | 禁止は記述どおり効き、`reason` もエラーに出る(確認済み)。ただし crate 側の clippy.toml はルートの clippy.toml を丸ごと置き換え、マージしない。ルートの `allow-unwrap-in-tests` などが黙って外れる(確認済み)                                                        | crate 側の clippy.toml にルートの共通設定も再掲する。lint レベルは workspace の `[workspace.lints.clippy] disallowed_methods = "deny"` で一元化すれば、各 lib.rs の `#![deny]` は不要                                                                 |
| 8   | 実装ガイド(infrastructure の各コード)      | `.map_err(RepositoryError::from)`・`UserDirectoryError::from`・`PayoutError::from` で下位エラーを変換 | infrastructure で `impl From<sqlx::Error> for RepositoryError` は書けない。どちらの型も外部 crate のもので、孤児ルール(E0117)に触れる(確認済み)。domain 側に書けば domain が sqlx に依存し、原則1を破る                                                            | infrastructure に変換関数(`fn db_err(sqlx::Error) -> RepositoryError` など)を置き、`.map_err(db_err)` と書く。一意制約違反はここで `Conflict` に振り分ける                                                                                            |
| 9   | 実装ガイド(payslips のスキーマ)            | `pay_year smallint`・`pay_month tinyint`(signed)                                                      | sqlx の `query!` は signed 列を i16 / i8 に推論する。ガイドの `PayPeriod::new(row.pay_year, row.pay_month)`(u16 / u8)がコンパイルエラーになる(確認済み)                                                                                                            | 列を `smallint unsigned`・`tinyint unsigned` にする                                                                                                                                                                                                   |
| 10  | Lint・フォーマット                         | clippy は pedantic                                                                                    | 日本語の doc コメントで `doc_markdown` が誤判定を連発する(全角括弧の中を識別子とみなす)。`must_use_candidate` もビルダー関数ごとに出てノイズが多い                                                                                                                 | workspace の lints で `doc_markdown`・`must_use_candidate`・`missing_errors_doc` を allow にする方針をガイドに書く                                                                                                                                    |
| 11  | セットアップ手順・データベース             | `mise run gen` の次に `cargo run -p migrate`。`.sqlx/` をコミット                                     | `DATABASE_URL` が設定されていると sqlx は `.sqlx/` より live DB を優先する。空の DB では infrastructure の `query!` がコンパイルエラーになり、migrate(bootstrap 経由で infrastructure に依存)自体がビルドできない(確認済み)                                        | mise の `[env]` に `SQLX_OFFLINE = "true"` を置き、キャッシュ更新は `SQLX_OFFLINE=false cargo sqlx prepare --workspace`(`mise run sqlx-prepare`)で行う。これで空の DB からでも `cargo run -p migrate` が通る(確認済み)                                |
| 12  | 実装ガイド(KeycloakUserDirectory)          | `username`・`email`・`enabled`・`credentials` だけでユーザーを作る                                    | Keycloak 24+ の既定ユーザープロファイルは `firstName`・`lastName` が必須。未設定のユーザーはパスワードを確定しても「Account is not fully set up」でログインできない(確認済み)                                                                                      | realm 定義(`components` の declarative-user-profile)で姓名を任意にする。Cognito と揃い、UserDirectory の trait を変えずに済む                                                                                                                         |
| 13  | Lint・フォーマット                         | `unwrap` は本番コードで禁止、テストでは許可                                                           | clippy の `allow-unwrap-in-tests` が効くのは `#[test]` 関数と `#[cfg(test)]` モジュールだけ。`tests/` のフェイクや補助関数は対象外で、`unwrap_used = "deny"` に引っかかる(確認済み)                                                                                | `tests/*.rs` の先頭に `#![allow(clippy::unwrap_used)]` を置く。フェイクのリポジトリが `reconstruct` を呼ぶ箇所は `#[allow(clippy::disallowed_methods, reason = ...)]` で個別に許可する                                                                |
| 14  | 実装ガイド(domainを守る原則1)              | deny.toml の bans で許可 crate を絞り、domain が I/O 系 crate に依存しないようにする                  | cargo-deny の bans は workspace 全体のルール。`wrappers`(直接の親として許す crate)は外部 crate にも適用されるので、tokio を「infrastructure 以外禁止」にすると tonic・opentelemetry_sdk などの外部の親まで全部列挙することになる(確認済み)                         | domain の純度は check-deps.sh で「domain の推移的依存に I/O 系 crate がないこと」として検査する(違反を仕込んで検出を確認済み)。deny.toml はライセンス・脆弱性・ORM や native-tls の全面禁止、sqlx の wrappers のように親が自前 crate だけのものに使う |
| 15  | ディレクトリ構成(infra/modules)            | modules は network・database・backend-service・lambda-function・frontend-hosting                      | 実装が前提にしている Cognito(ユーザープール・グループ・PKCE クライアント)、SQS FIFO キューと DLQ、ECR、DB 接続文字列のシークレットを置くモジュールがない                                                                                                           | `auth`(Cognito)と `messaging`(SQS FIFO + DLQ)を追加。ECR は backend-service、シークレットは database に含めた。3環境とも `terraform validate`・tflint・trivy が通ることを確認(AWS への apply は未実施)                                                |
| 16  | Lint・フォーマット / CI(infra)             | Terraform を tflint と trivy で検査                                                                   | 素直な構成でも trivy が HIGH/CRITICAL を出す(公開 ALB、外向き 0.0.0.0/0、CloudFront に WAF なし、S3 が CMK でない)。扱い方の記述がない                                                                                                                             | 外向きは 443 と VPC 内 3306 に絞る。残りは `.trivyignore.yaml` に理由付きで記録する運用をガイドに書く                                                                                                                                                 |
| 17  | CI/CD(deploy の図)・データベース(実行主体) | migrate を ECS の単発タスクとして実行                                                                 | ガイドの Dockerfile は server 用とだけあり、migrate を動かすイメージが定義されていない                                                                                                                                                                             | server イメージに migrate バイナリも入れ、migrate のタスク定義では `entryPoint` を `/app/migrate` に差し替える                                                                                                                                        |
| 18  | セットアップ手順・CI                       | `mise install` で各種ツールを揃える(CI は jdx/mise-action)                                            | ツールチェーン未導入の環境では、mise が cargo 系ツールを並列にビルドする際、各 cargo が rust-toolchain.toml の版と rustfmt・clippy を同時に取りに行き、rustup のダウンロードが競合して失敗する(GitHub Actions の初回実行で3本とも失敗、コンテナで再現・修正を確認) | `mise install` の前に `rustup toolchain install` を1回実行する。ワークフローでは mise-action の前のステップに置く                                                                                                                                     |

## ガイドに書かれていないが、実装で必要だったこと

誤りではないが、ガイドに書き足すと迷わずに済む点。

- **gRPC-Web の CORS**: ブラウザ(Connect-ES)から別オリジンの server を叩くには、CORS で `x-grpc-web`・`x-user-agent`・`connect-protocol-version` などのヘッダを許可する必要がある。加えて `grpc-status`・`grpc-message` を expose しないと、JS からエラー内容が読めない。tower-http の `CorsLayer` を server に入れた。
- **/health と gRPC の同居**: tonic の `Routes::into_axum_router()` で axum の Router にまとめると、`/health` を認証の外に置ける。`GrpcWebLayer` も同じ Router に載る(`accept_http1` は不要)。
- **AuthenticatedUser の置き場所**: `shared/auth` に置くと、usecase が JWT・HTTP 系 crate に推移的に依存する。型だけを `shared/kernel` に置き、検証処理は `shared/auth` に分けた。
- **振込の冪等キー**: 「処理済みイベントIDを記録して二重処理を弾く」だけでは、振込 API の呼び出しと記録の間で落ちたときに二重振込になる。振込 API に outbox の id を冪等キー(`Idempotency-Key`)として渡した。
- **事前チェックのエラー文言**: 一意制約違反をそのまま返すと、MySQL の文言(`Duplicate entry ... for key ...`)がクライアントに漏れる。usecase の事前チェックで分かりやすい文言を返し、制約違反はログにだけ詳細を残す。
- **OIDC と React StrictMode**: 開発モードでは effect が2回走り、認可コードを2回交換して「Code not valid」で失敗する(E2E で確認)。コールバック処理は1回だけ実行するようにした。
- **フロントの接続先**: ガイドの「バイナリは全環境で同一」をフロントにも当てはめ、接続先はビルド時の環境変数ではなく実行時に読む `config.json` にした。Terraform が環境ごとに生成して S3 に置く。
- **フロントのテスト**: ガイドは MSW を挙げているが、gRPC-Web のバイナリ応答を MSW で組み立てるのは手間がかかる。Connect の `createRouterTransport`(インメモリでサービスを差し替える)の方が素直だった。
- **SQS トリガーの Lambda をローカルで動かす方法**: ガイドは EventBridge 系については書いているが、SQS トリガーは書いていない。ElasticMQ をポーリングして同じ処理を呼ぶ `local_poller` を用意した。Lambda 本体も `cargo lambda watch` / `invoke` で動作を確認した。
- **Lambda の一覧**: 構成ガイドは `lambdas/{payroll-monthly-close,timesheet-import}` を挙げている。一方、実装ガイドの consumer は振込の Lambda(ここでは payout-dispatcher)で、名前と役割が一致しない。業務仕様がない2つは作っていない。
- **opentelemetry の版**: `tracing-opentelemetry` の最新(0.33)は `opentelemetry` 0.32 用。各 crate の最新版をそのまま並べると型が合わない。
- **reqwest 0.13**: `form()` が `form` feature に分かれた。

## 検証を受けて見直した設計

ガイドの記述どおりに動いたが、使い続けると困る点が見つかり、ガイドの方針ごと改めたもの。ガイドは改訂済み。

- **出来事は集約に溜めず、操作の戻り値で返す**: 当初は集約が `events` に出来事を溜め、リポジトリの `insert` が `take_events()` で取り出して outbox に書いていた。
  - この形だと、取り出し忘れ(`update` 側で outbox に書き忘れるなど)をコンパイラが検出できない。集約の記録と outbox の記録がリポジトリの中に隠れ、ユースケースを読んでも出来事の流れが追えない。
  - `finalize()` が出来事を戻り値で返す形(cqrs-es などの Decider パターン)にした。
  - `PayslipEvent`・`PayrollEvent` に `#[must_use]` を付け、workspace の lints で `unused_must_use = "deny"` にした。`Result<PayslipEvent, _>` を `?` で包んで捨てた場合も、`(FinalizedPayslip, PayslipEvent)` のタプルごと捨てた場合も検出される(「unused `PayslipEvent` in tuple element 1」)ことを確認した。
- **給与明細の状態を型で分ける**: 当初は `status` フィールドを持つ1つの型で、`finalize(&mut self)` が作成中かを実行時に確かめ、確定済みなら `AlreadyFinalized` を返していた。
  - 作成中と確定済みを別の型(`DraftPayslip`・`FinalizedPayslip`)にし、`finalize(self) -> (FinalizedPayslip, PayslipEvent)` は作成中の型にだけ置いた。確定済みに `finalize` を呼ぶとコンパイルエラー(E0599)になることを compile_fail の doc テストで固定し、`AlreadyFinalized` はなくした。
  - 形は「共通の内容 + 状態ごとの型 + 状態を問わない enum」にした。
    - `PayslipContent<Id>` に共通のデータ(番号・派遣社員・対象月・明細行)とアクセサ・支給額を1回だけ持つ。
    - `DraftPayslip<Id>`・`FinalizedPayslip<Id>` は `content` を持つ状態ごとの構造体。`finalize` は作成中だけに置き、状態ごとの情報(確定日時など)はここに足す。
    - `enum Payslip<Id> { Draft, Finalized }` は状態を問わない給与明細で、`content()` と `status()` を持つ。呼び出し側は `payslip.content().lines()` のように1段たどって読む。
    - 型引数は登録(番号)の軸だけ。
  - 他のプロダクトや文献を調べて決めた(2026-09-19)。
    - 状態ごとのレコードを sum type でまとめ、共通データを共通のレコードにするのは、F# の Scott Wlaschin(_Designing with types_・_Domain Modeling Made Functional_)の形。Rust 公式入門書の `DraftPost`・`Post` の例、corrode の記事(状態は基本 enum、typestate は型引数で読みにくくなる)とも合う。
    - 実運用のプロダクト(Lemmy・zero2prod・cqrs-es の例)は、1つの構造体に状態のフィールド(bool や status 列)を持ち、実行時に確かめる形が最も多い。
    - typestate + 保存用の enum の組み合わせを勧める記事もあるが、保存される集約では必ず enum を経由するので、アクセサが2か所になる。
  - 途中で試してやめた形:
    - 状態を型引数で持つ形(`PayslipIn<State, Id>` + `PhantomData<State>`)+ 状態を問わない `enum Payslip`: アクセサが2か所になる(enum 側は `Self::Draft(p) | Self::Finalized(p)` と書けず E0308、`each_state!` マクロで1行にしていた)。
    - 状態の型引数に実行時の状態(`PayslipStatus`)も入れる形(`Payslip<S = PayslipStatus, Id>` + `State` trait + `PayslipState`): アクセサは1か所になったが、型引数の省略時の意味や `State` trait など独自の仕組みが増え、他の人が読んで定番と分かる形ではなかった。
    - 状態ごとの構造体に共通のフィールドとアクセサをそれぞれ書く形: 状態の数だけ重複する。
  - Rust では `&mut self` の書き換えも所有権で1か所に限られるので、「元を消費して新しいものを返す」こと自体の利点は小さい。消費する形にしたのは、戻り値の型を変える(状態を型で表す)ためだけ。
  - リポジトリは状態を問わない `Payslip` を返し、状態の確かめは usecase が `match` か `let Payslip::Draft(draft) = payslip else { .. }` で行う。確かめずに `payslip.finalize()` と書くとコンパイルエラー(E0599)になり、確かめてから取り出した `draft.finalize()` は通る。状態が違うときは `FailedPrecondition` を返し、存在しない(`NotFound`)と区別する。
  - `find_draft(id) -> Option<DraftPayslip>` のような状態ごとの取り出しはリポジトリに置かない。状態が違うのか存在しないのかが区別できず、状態が増えるたびにメソッドも増えるため。同じ確かめ方が複数のユースケースに出てきたら、`Payslip` に `into_draft()` のような取り出しを足す(今は新規作成直後の `finalize` だけなので未実装)。
- **リポジトリは書き込み先を受け取り、トランザクションを張るかは usecase が決める**: ガイドの「1トランザクションで複数集約を更新しない」は、1つのユースケースで複数の集約を扱う場面が出ると守れない。制約は「コンテキストをまたいで1トランザクションで更新しない」に緩めた。
  - 採用した形:
    - usecase は `Database` から書き込み先を用意する。一緒に確定させたい記録は `transaction()` に書いて `commit()`、1件だけ書くときは `connection()` に書く。
    - リポジトリの `insert`・`update` と `EventOutbox::append` は書き込み先 `Db` を受け取り、それがトランザクションか接続かを区別しない(Go の sqlc の `DBTX`、sqlx の `Executor` と同じ考え方)。取り出し(`find` など)はプール経由のまま。
    - 給与明細の `insert` は明細行と一緒に書くので、受け取った書き込み先の中で `begin()` する。sqlx では、外側がトランザクションなら SAVEPOINT、接続なら本物のトランザクションになる。外側を捨てれば SAVEPOINT ごと消えること、接続に書いても明細行まで丸ごと確定することを DB 結合テストで確認した。
    - `Db` は中身を隠した箱(`Box<dyn DbHandle>`)で、usecase・handler・bootstrap は型引数を持たない。infrastructure は `mysql()` で箱から MySQL の接続を取り出す。別の実装の `Db` が渡されるのは組み立ての誤りで、記録せずに `Unavailable` を返す(単体テストで確認)。
  - トランザクションの張り忘れ(給与明細と出来事を別の書き込み先に書くなど)は型では防げない。usecase の責務として、レビューと usecase のテスト(出来事の記録に失敗したら給与明細も残らない)で守る。
  - 途中で次の方式も試した。
    - スコープ(Unit of Work)方式: スコープから記録先(`tx.payslips().insert(..)`)を借りる形。型引数は広がらないが、記録先がどのトランザクションを使っているかが読み取りにくかった。
    - `tx.insert(&payslip)` を型ごとの `Insert<T>` trait で呼び分ける形: `Box<dyn …>` 越しでも型推論が効くことは実験で確認したが、採らなかった。
    - tx を必須の引数にし、その型を関連型(`type Tx`)にする形: 取り違えはコンパイル時に分かるが、ユースケースと handler に型引数 `<T: Transactions>` が広がった。
    - tx を必須の引数にし、中身を隠した箱にする形: 型引数は消えたが、1件だけ書くときも必ずトランザクションを張ることになる。張るかどうかは usecase が決めることなので、今の形にした。
  - `&mut` 参照は同時に1つしか持てないので、1つの書き込み先への書き込みは順番にしか行えない(E0499 で確認)。

- **usecase のテストに mockall は入れない**: 状態を持つ手書きのフェイクのままにした。
  - `tests/` の統合テストから見るとライブラリは `cfg(test)` なしでビルドされるので、`#[cfg_attr(test, automock)]` のモックは見えない。使うには feature を用意する必要がある。
  - スコープ方式のときの `fn payslips(&mut self) -> Box<dyn PayslipStore + '_>` は、`#[automock]` でコンパイルエラーになった(E0106・E0637)。借用を返すメソッドは mockall では生成できない。
  - 確かめたいのは呼び出しの順序や回数ではなく、最後に何が記録されたか。フェイクの書き込み先もトランザクションは commit まで記録を溜めるので、それを確かめられる。外部に副作用を起こすポート(`PayoutGateway`・`UserDirectory` など)で呼び出し回数を固定したくなったら、そのポートにだけ入れる。

## この環境では確かめていないこと

- AWS への `terraform apply` と実際のデプロイ(`deploy` ワークフロー)。Terraform は validate・tflint・trivy まで。
- Cognito での動作(クレーム mapper と UserDirectory はコードとユニットテストのみ)。Cognito の OIDC ディスカバリには `end_session_endpoint` がないため、ログアウトは失敗時にローカルのセッションだけ消す実装にした(未検証)。
- GitHub Actions 上での実行(actionlint は通過)。
- ADOT collector 経由の X-Ray 送信(ローカルの Jaeger への OTLP 送信まで)。

## 確認済みの動作

| 対象                                                                                            | 方法                                                     | 結果                             |
| ----------------------------------------------------------------------------------------------- | -------------------------------------------------------- | -------------------------------- |
| Rust の lint(fmt・clippy pedantic・cargo-deny・依存ルール)                                      | `mise run lint:rust`                                     | 通過                             |
| domain・usecase の単体テスト、DB 結合テスト(testcontainers)、API テスト(tonic クライアント)     | `mise run test:rust`                                     | 33件通過                         |
| フロントの lint・型・コンポーネントテスト・ビルド                                               | `mise run lint:ts`・`test:ts`・`pnpm --filter web build` | 通過(2件)                        |
| gRPC の主要シナリオ(認証・認可・二重確定・入力検証・金額計算)                                   | `scripts/smoke-test.sh`                                  | 16件通過                         |
| outbox → relay → ElasticMQ → consumer(再配信の冪等性を含む)                                     | local_poller・`cargo lambda invoke`                      | 期待どおり                       |
| ブラウザの一連の流れ(Keycloak ログイン・初回パスワード変更・給与確定・本人だけが明細を見られる) | `mise run e2e`(Playwright)                               | 2件通過(3回反復でも安定)         |
| server イメージ(cargo-chef・distroless)                                                         | ビルド・起動・`/health`・SIGTERM・イメージ内 migrate     | 62MB、0.05秒でグレースフルに停止 |
| Terraform(3環境)                                                                                | validate・tflint・trivy                                  | 通過                             |
| ワークフロー                                                                                    | actionlint                                               | 通過                             |
