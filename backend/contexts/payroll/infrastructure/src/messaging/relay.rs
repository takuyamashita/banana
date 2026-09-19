use std::time::Duration;

use aws_sdk_sqs::Client as SqsClient;
use sqlx::Connection as _;
use sqlx::mysql::{MySqlConnection, MySqlPool};
use thiserror::Error;
use tokio_util::sync::CancellationToken;

use super::envelope::OutboxEnvelope;

#[derive(Debug, Error)]
pub enum RelayError {
    #[error("DB: {0}")]
    Db(#[from] sqlx::Error),
    #[error("SQS: {0}")]
    Sqs(String),
    #[error("JSON: {0}")]
    Json(#[from] serde_json::Error),
}

/// relay を1つのインスタンスだけで動かすためのロック名(MySQL の `GET_LOCK`)。
/// 複数のインスタンスが並んで送ると、同じ集約の出来事が outbox の順と違う順で SQS に届きうる
const RELAY_LOCK: &str = "payroll_outbox_relay";

/// 未送信の出来事を outbox の順に SQS へ送り、送った件数を返す。
///
/// 送るのはロックを取れたインスタンスだけで、取れなかったときは何もせず 0 を返す。
/// ロックは接続に結びつくので、プロセスが落ちて接続が切れれば外れる
pub async fn relay_once(
    pool: &MySqlPool,
    sqs: &SqsClient,
    queue_url: &str,
) -> Result<usize, RelayError> {
    let mut conn = pool.acquire().await?;
    let locked: Option<i64> =
        sqlx::query_scalar("select get_lock(?, 0)").bind(RELAY_LOCK).fetch_one(&mut *conn).await?;
    if locked != Some(1) {
        return Ok(0);
    }

    let sent = send_unpublished(&mut conn, sqs, queue_url).await;

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
) -> Result<usize, RelayError> {
    let mut tx = conn.begin().await?;

    let rows = sqlx::query!(
        r#"select id, aggregate_id, payload as "payload: sqlx::types::JsonValue" from outbox
         where published_at is null
         order by id
         limit 100"#
    )
    .fetch_all(&mut *tx)
    .await?;

    for row in &rows {
        sqs.send_message()
            .queue_url(queue_url)
            // outbox の id をイベントIDとして封筒に載せ、consumerの二重処理判定に使う
            .message_body(serde_json::to_string(&OutboxEnvelope {
                event_id: row.id,
                payload: &row.payload,
            })?)
            // 同じ集約のイベント順序を保つ
            .message_group_id(row.aggregate_id.to_string())
            // relayが二重に送ってもSQS側で除去される(重複排除の窓は5分)
            .message_deduplication_id(row.id.to_string())
            .send()
            .await
            .map_err(|e| RelayError::Sqs(aws_sdk_sqs::error::DisplayErrorContext(e).to_string()))?;

        sqlx::query!("update outbox set published_at = current_timestamp(6) where id = ?", row.id)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;
    Ok(rows.len())
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
        match relay_once(&pool, &sqs, &queue_url).await {
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
