use std::collections::HashSet;
use std::time::Duration;

use aws_sdk_sqs::Client as SqsClient;
use aws_sdk_sqs::types::MessageAttributeValue;
use sqlx::mysql::{MySqlConnection, MySqlPool};
use thiserror::Error;
use tokio_util::sync::CancellationToken;

use super::envelope::OutboxEnvelope;

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

/// relay を1つのインスタンスだけで動かすためのロック名(MySQL の `GET_LOCK`)。
/// 複数のインスタンスが並んで送ると、同じ集約の出来事が outbox の順と違う順で SQS に届きうる
const RELAY_LOCK: &str = "payroll_outbox_relay";

/// 未送信の出来事を outbox の順に SQS へ送り、送った件数を返す。
///
/// 送るのはロックを取れたインスタンスだけで、取れなかったときは何もせず 0 を返す。
/// ロックは接続に結びつくので、プロセスが落ちて接続が切れれば外れる。
///
/// `stop` が止められたら、次の1件を送る前にやめる(停止を待たせない。送り終えた行は送信済みにしてある)
pub async fn relay_once(
    pool: &MySqlPool,
    sqs: &SqsClient,
    queue_url: &str,
    stop: &CancellationToken,
) -> Result<usize, RelayError> {
    let mut conn = pool.acquire().await?;
    let locked: Option<i64> =
        sqlx::query_scalar("select get_lock(?, 0)").bind(RELAY_LOCK).fetch_one(&mut *conn).await?;
    if locked != Some(1) {
        return Ok(0);
    }

    let sent = send_unpublished(&mut conn, sqs, queue_url, stop).await;

    // 外せなかったロックを接続ごとプールに戻すと、以後どのインスタンスも送れなくなる。
    // 外せなかったときは接続を閉じる(閉じればロックも外れる)
    let released = sqlx::query("select release_lock(?)").bind(RELAY_LOCK).execute(&mut *conn).await;
    if let Err(err) = released {
        tracing::warn!(error = %err, "failed to release relay lock, closing the connection");
        conn.close_on_drop();
    }
    sent
}

async fn send_unpublished(
    conn: &mut MySqlConnection,
    sqs: &SqsClient,
    queue_url: &str,
    stop: &CancellationToken,
) -> Result<usize, RelayError> {
    let rows = sqlx::query!(
        r#"select id, aggregate_type, aggregate_id, event_type, attempts, traceparent,
                  payload as "payload: sqlx::types::JsonValue"
           from outbox
           where published_at is null and attempts < ?
           order by id
           limit 100"#,
        MAX_ATTEMPTS,
    )
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
        let mut request = sqs
            .send_message()
            .queue_url(queue_url)
            .message_body(body)
            .message_group_id(&group)
            // relay が二重に送っても SQS 側で除去される(重複排除の窓は5分)
            .message_deduplication_id(row.id.to_string());
        // 出来事を記録したリクエストのトレースを、受け手に引き継ぐ
        if let Some(traceparent) = &row.traceparent
            && let Ok(attribute) = MessageAttributeValue::builder()
                .data_type("String")
                .string_value(traceparent)
                .build()
        {
            request = request.message_attributes("traceparent", attribute);
        }
        let result = request.send().await;

        match result {
            // 送れた行はすぐに送信済みにする。後の行で失敗しても送り直さない
            Ok(_) => {
                sqlx::query!(
                    "update outbox set published_at = current_timestamp(6) where id = ?",
                    row.id
                )
                .execute(&mut *conn)
                .await?;
                sent += 1;
            }
            Err(err) => {
                let error = aws_sdk_sqs::error::DisplayErrorContext(err).to_string();
                held_back.insert(group);
                let last_error: String = error.chars().take(1000).collect();
                sqlx::query!(
                    "update outbox set attempts = attempts + 1, last_error = ? where id = ?",
                    last_error,
                    row.id
                )
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
    sqlx::query!(
        "delete from outbox where published_at < current_timestamp(6) - interval 30 day limit 1000"
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
    sqs: SqsClient,
    queue_url: String,
    interval: Duration,
    cancel: CancellationToken,
) {
    loop {
        match relay_once(&pool, &sqs, &queue_url, &cancel).await {
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
