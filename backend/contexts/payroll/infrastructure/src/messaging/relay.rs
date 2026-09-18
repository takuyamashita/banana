use std::time::Duration;

use aws_sdk_sqs::Client as SqsClient;
use sqlx::mysql::MySqlPool;
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

// 未送信行を取り出してSQSへ送る。
// SKIP LOCKED により、複数インスタンスで同時に動かしても同じ行を掴まない
pub async fn relay_once(
    pool: &MySqlPool,
    sqs: &SqsClient,
    queue_url: &str,
) -> Result<usize, RelayError> {
    let mut tx = pool.begin().await?;

    let rows = sqlx::query!(
        r#"select id, aggregate_id, payload as "payload: sqlx::types::JsonValue" from outbox
         where published_at is null
         order by id
         limit 100
         for update skip locked"#
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
