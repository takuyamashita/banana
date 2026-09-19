# 0002 sqlx の query! は既定でオフラインキャッシュを使う

- 日付: 2026-09-19
- 状態: 採用

## 決定

mise の `[env]` で `SQLX_OFFLINE=true` を既定にし、コミット済みの `.sqlx/` でコンパイルする。
クエリを変えたら `mise run sqlx-prepare` でキャッシュを更新し(未適用のマイグレーションは sqlx-cli で先に当てる。列を足してクエリも変えたときは、古いキャッシュのままでは migrate 自体がビルドできないため)、CI の `cargo sqlx prepare --check` で更新漏れを弾く。

## 理由

`DATABASE_URL` があると sqlx は live DB を優先する。空の DB では infrastructure の `query!` がコンパイルできず、
bootstrap 経由で infrastructure に依存する migrate 自体がビルドできなくなる(鶏と卵)。
