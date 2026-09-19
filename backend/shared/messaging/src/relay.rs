//! outbox の未送信の出来事を、記録した順に送る。
//!
//! outbox テーブルはどのサービスも同じ形にする(各サービスのマイグレーションで作る):
//! id(連番)・aggregate_type・aggregate_id・event_type・payload(JSON)・created_at・published_at・
//! attempts・last_error・traceparent

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use sqlx::mysql::{MySqlConnection, MySqlPool};
use thiserror::Error;
use tokio_util::sync::CancellationToken;

use crate::envelope::OutboxEnvelope;
use crate::publisher::{Outgoing, Publisher};

#[derive(Debug, Error)]
pub enum RelayError {
    #[error("DB: {0}")]
    Db(#[from] sqlx::Error),
    #[error("JSON: {0}")]
    Json(#[from] serde_json::Error),
}

/// 送れなかった出来事を諦めるまでの回数。諦めた出来事は error ログに出し、同じ集約の後ろの出来事を先に進める
const MAX_ATTEMPTS: i32 = 10;

/// 送れていない出来事がこれより長く残っていたら warn ログを出す(秒)
const BACKLOG_WARN_SECONDS: i64 = 300;

/// relay を1つのインスタンスだけで動かすためのロック(MySQL の `GET_LOCK`)の名前。
/// 複数のインスタンスが並んで送ると、同じ集約の出来事が outbox の順と違う順で届きうる。
/// ロックの名前は DB サーバー全体で共有されるので、同じ MySQL に載るサービスごとに別の名前にする
#[derive(Debug, Clone)]
pub struct RelayLock(String);

impl RelayLock {
    /// サービス名から決める(例: payroll → payroll_outbox_relay)
    pub fn for_service(service: &str) -> Self {
        Self(format!("{service}_outbox_relay"))
    }

    pub fn name(&self) -> &str {
        &self.0
    }
}

#[derive(sqlx::FromRow)]
struct Row {
    id: i64,
    aggregate_type: String,
    aggregate_id: i64,
    event_type: String,
    attempts: i32,
    traceparent: Option<String>,
    payload: sqlx::types::JsonValue,
}

/// 未送信の出来事を outbox の順に送り、送った件数を返す。
///
/// 送るのはロックを取れたインスタンスだけで、取れなかったときは何もせず 0 を返す。
/// ロックは接続に結びつくので、プロセスが落ちて接続が切れれば外れる。
///
/// `stop` が止められたら、次の1件を送る前にやめる(停止を待たせない。送り終えた行は送信済みにしてある)
pub async fn relay_once(
    pool: &MySqlPool,
    publisher: &dyn Publisher,
    lock: &RelayLock,
    stop: &CancellationToken,
) -> Result<usize, RelayError> {
    let mut conn = pool.acquire().await?;
    let locked: Option<i64> =
        sqlx::query_scalar("select get_lock(?, 0)").bind(lock.name()).fetch_one(&mut *conn).await?;
    if locked != Some(1) {
        return Ok(0);
    }

    let sent = send_unpublished(&mut conn, publisher, stop).await;

    // 外せなかったロックを接続ごとプールに戻すと、以後どのインスタンスも送れなくなる。
    // 外せなかったときは接続を閉じる(閉じればロックも外れる)
    let released =
        sqlx::query("select release_lock(?)").bind(lock.name()).execute(&mut *conn).await;
    if let Err(err) = released {
        tracing::warn!(error = %err, "failed to release relay lock, closing the connection");
        conn.close_on_drop();
    }
    sent
}

async fn send_unpublished(
    conn: &mut MySqlConnection,
    publisher: &dyn Publisher,
    stop: &CancellationToken,
) -> Result<usize, RelayError> {
    let rows: Vec<Row> = sqlx::query_as(
        "select id, aggregate_type, aggregate_id, event_type, attempts, traceparent, payload
         from outbox
         where published_at is null and attempts < ?
         order by id
         limit 100",
    )
    .bind(MAX_ATTEMPTS)
    .fetch_all(&mut *conn)
    .await?;

    let mut sent = 0;
    // 送れなかった出来事の集約。同じ集約の後ろの出来事は、それを送れるまで送らない(順序を保つ)
    let mut held_back = HashSet::new();
    for row in &rows {
        if stop.is_cancelled() {
            // 片付け(滞留の確認・古い行の削除)は次の起動に任せる
            return Ok(sent);
        }
        // 同じ集約の出来事は同じグループに入れ、キューの中でも順序を保つ
        let group = format!("{}-{}", row.aggregate_type, row.aggregate_id);
        if held_back.contains(&group) {
            continue;
        }
        let body = serde_json::to_string(&OutboxEnvelope {
            event_id: row.id,
            event_type: row.event_type.clone(),
            aggregate_type: row.aggregate_type.clone(),
            aggregate_id: row.aggregate_id,
            payload: &row.payload,
        })?;
        let deduplication_id = row.id.to_string();
        let result = publisher
            .publish(&Outgoing {
                group: &group,
                deduplication_id: &deduplication_id,
                event_type: &row.event_type,
                body: &body,
                traceparent: row.traceparent.as_deref(),
            })
            .await;

        match result {
            // 送れた行はすぐに送信済みにする。後の行で失敗しても送り直さない
            Ok(()) => {
                sqlx::query("update outbox set published_at = current_timestamp(6) where id = ?")
                    .bind(row.id)
                    .execute(&mut *conn)
                    .await?;
                sent += 1;
            }
            Err(err) => {
                let error = err.to_string();
                held_back.insert(group);
                let last_error: String = error.chars().take(1000).collect();
                sqlx::query(
                    "update outbox set attempts = attempts + 1, last_error = ? where id = ?",
                )
                .bind(last_error)
                .bind(row.id)
                .execute(&mut *conn)
                .await?;
                if row.attempts + 1 >= MAX_ATTEMPTS {
                    tracing::error!(outbox_id = row.id, error, "gave up sending an outbox row");
                } else {
                    tracing::warn!(
                        outbox_id = row.id,
                        error,
                        "failed to send an outbox row, will retry"
                    );
                }
            }
        }
    }

    warn_if_backlogged(conn).await?;
    // 送信済みで保持期間を過ぎた行を消す(1回に消す数は抑える)
    sqlx::query(
        "delete from outbox where published_at < current_timestamp(6) - interval 30 day limit 1000",
    )
    .execute(&mut *conn)
    .await?;
    Ok(sent)
}

/// 送れていない出来事が長く残っていたら知らせる(送信先の設定の誤りなどで、送れない状態が続いている)
async fn warn_if_backlogged(conn: &mut MySqlConnection) -> Result<(), RelayError> {
    let oldest: Option<i64> = sqlx::query_scalar(
        "select timestampdiff(second, min(created_at), current_timestamp(6)) from outbox
         where published_at is null and attempts < ?",
    )
    .bind(MAX_ATTEMPTS)
    .fetch_one(&mut *conn)
    .await?;
    if let Some(seconds) = oldest.filter(|s| *s > BACKLOG_WARN_SECONDS) {
        tracing::warn!(oldest_unpublished_seconds = seconds, "outbox is backlogged");
    }
    Ok(())
}

/// server プロセス内の常駐タスクとして relay を回す。キャンセルされたら抜ける
pub async fn run(
    pool: MySqlPool,
    publisher: Arc<dyn Publisher>,
    lock: RelayLock,
    interval: Duration,
    cancel: CancellationToken,
) {
    loop {
        match relay_once(&pool, publisher.as_ref(), &lock, &cancel).await {
            Ok(0) => {}
            Ok(n) => tracing::info!(sent = n, "outbox relayed"),
            Err(err) => tracing::warn!(error = %err, "outbox relay failed"),
        }
        tokio::select! {
            () = cancel.cancelled() => break,
            () = tokio::time::sleep(interval) => {}
        }
    }
    tracing::info!("outbox relay stopped");
}
