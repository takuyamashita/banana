//! マイグレーションと、アプリが DB に接続するユーザーの用意。
//!
//! AWS では migrate だけが管理者で接続し、テーブルを作り変える。アプリ(server・Lambda)は
//! 読み書きしかできないユーザーで接続するので、アプリの不具合や乗っ取りでテーブルを消されることはない

use serde::Deserialize;
use sqlx::MySqlPool;

/// DB のユーザー名とパスワード。RDS が管理する管理者のシークレットと同じ形(JSON)
#[derive(Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

impl Credentials {
    pub fn from_json(json: &str) -> anyhow::Result<Self> {
        Ok(serde_json::from_str(json)?)
    }
}

impl std::fmt::Debug for Credentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Credentials").field("username", &self.username).finish_non_exhaustive()
    }
}

/// アプリ用のユーザーを作り(あればパスワードを合わせ)、`database` の表の読み書きだけを許す。
///
/// パスワードを変えたシークレットで流し直せば、パスワードも変わる
pub async fn ensure_app_user(
    pool: &MySqlPool,
    database: &str,
    user: &Credentials,
) -> anyhow::Result<()> {
    // CREATE USER・GRANT はユーザー名とパスワードを引数(?)で渡せないので、文に埋め込む。
    // 埋め込んでも文の意味が変わらない文字だけを受け付ける
    let plain = |s: &str| !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
    anyhow::ensure!(plain(&user.username) && user.username.len() <= 32, "使えないユーザー名です");
    anyhow::ensure!(plain(&user.password) && user.password.len() >= 16, "使えないパスワードです");
    anyhow::ensure!(plain(database), "使えないデータベース名です");

    let (name, password) = (&user.username, &user.password);
    // 埋め込むのは上で確かめた値だけ
    sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
        "create user if not exists '{name}'@'%' identified by '{password}';
         alter user '{name}'@'%' identified by '{password}';
         grant select, insert, update, delete on `{database}`.* to '{name}'@'%';"
    )))
    .execute(pool)
    .await?;
    Ok(())
}
