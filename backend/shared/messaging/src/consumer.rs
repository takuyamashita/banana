//! 自分のキューに届いた出来事を処理する。Lambda(SQS のイベントソース)でも、server の中の常駐タスク
//! ([`run`])でも、1回に受け取った分を [`handle_batch`] で同じように処理する

use std::collections::HashSet;
use std::time::Duration;

use async_trait::async_trait;
use aws_sdk_sqs::types::{DeleteMessageBatchRequestEntry, MessageSystemAttributeName};
use tokio_util::sync::CancellationToken;
use tracing::Instrument as _;

/// キューから受け取ったメッセージ1件
#[derive(Debug, Clone)]
pub struct Message {
    /// メッセージ ID
    pub id: String,
    /// 消すときに使う受け取りの印(Lambda では使わない)
    pub receipt_handle: String,
    /// FIFO のメッセージグループ(集約ごと)。同じグループの中では順に処理する
    pub group: String,
    pub body: String,
    /// 出来事を記録したリクエストのトレース(メッセージ属性 traceparent)。処理をその続きにする
    pub traceparent: Option<String>,
}

/// 届いた出来事1件の処理。同じ出来事が2回届いても結果が変わらないように作る
#[async_trait]
pub trait Handler: Send + Sync {
    async fn handle(&self, body: &str) -> anyhow::Result<()>;
}

/// 受け取った順に処理し、失敗した件のメッセージ ID を返す。失敗した件だけがキューに戻る。
///
/// 同じグループの中では順序を守るため、ある件が失敗したら同じグループの後ろの件は処理せずに失敗として返す。
/// 別のグループ(別の集約)は、そのまま処理を続ける
pub async fn handle_batch(messages: &[Message], handler: &impl Handler) -> Vec<String> {
    let mut failed_groups = HashSet::new();
    let mut failed = Vec::new();
    for message in messages {
        if failed_groups.contains(&message.group) {
            failed.push(message.id.clone());
            continue;
        }
        let span = tracing::info_span!("message", message_id = %message.id);
        platform_telemetry::set_parent(&span, |key| {
            (key == "traceparent").then(|| message.traceparent.clone()).flatten()
        });
        if let Err(err) = handler.handle(&message.body).instrument(span).await {
            tracing::warn!(message_id = %message.id, error = %err, "processing failed, will retry");
            failed_groups.insert(message.group.clone());
            failed.push(message.id.clone());
        }
    }
    failed
}

/// 1回の受け取りで待つ秒数(ロングポーリング)。止める合図にすぐ気づけるよう、SQS の上限(20秒)より短くする
const WAIT_SECONDS: i32 = 5;
/// 受け取りに失敗したときに、次に試すまで待つ時間
const RETRY_AFTER: Duration = Duration::from_secs(5);

/// server プロセスの中で自分のキューを読み続ける。処理できた件はキューから消し、失敗した件は残す
/// (見えない時間が過ぎると再配信され、何度も失敗したものはキューの設定で DLQ に移る)。キャンセルされたら抜ける
pub async fn run(
    sqs: aws_sdk_sqs::Client,
    queue_url: String,
    handler: impl Handler,
    cancel: CancellationToken,
) {
    loop {
        let received = tokio::select! {
            () = cancel.cancelled() => break,
            received = receive(&sqs, &queue_url) => received,
        };
        let messages = match received {
            Ok(messages) => messages,
            Err(err) => {
                tracing::warn!(error = %err, "failed to receive messages");
                tokio::select! {
                    () = cancel.cancelled() => break,
                    () = tokio::time::sleep(RETRY_AFTER) => continue,
                }
            }
        };
        if messages.is_empty() {
            continue;
        }
        // 受け取った分は、止める合図が来ても処理し終える(途中でやめると、同じグループの順序が崩れうる)
        let failed = handle_batch(&messages, &handler).await;
        let done: Vec<_> = messages.iter().filter(|m| !failed.contains(&m.id)).collect();
        if let Err(err) = delete(&sqs, &queue_url, &done).await {
            // 消せなかった件は再配信される。受け手は冪等なので、もう一度処理しても結果は変わらない
            tracing::warn!(error = %err, "failed to delete processed messages");
        }
    }
    tracing::info!(queue_url, "consumer stopped");
}

async fn receive(sqs: &aws_sdk_sqs::Client, queue_url: &str) -> anyhow::Result<Vec<Message>> {
    let output = sqs
        .receive_message()
        .queue_url(queue_url)
        .max_number_of_messages(10)
        .wait_time_seconds(WAIT_SECONDS)
        .message_system_attribute_names(MessageSystemAttributeName::MessageGroupId)
        .message_attribute_names("traceparent")
        .send()
        .await
        .map_err(|e| anyhow::anyhow!(aws_sdk_sqs::error::DisplayErrorContext(e).to_string()))?;
    Ok(output
        .messages
        .unwrap_or_default()
        .into_iter()
        .map(|m| Message {
            group: m
                .attributes
                .as_ref()
                .and_then(|a| a.get(&MessageSystemAttributeName::MessageGroupId).cloned())
                .unwrap_or_default(),
            traceparent: m
                .message_attributes
                .as_ref()
                .and_then(|a| a.get("traceparent"))
                .and_then(|a| a.string_value.clone()),
            id: m.message_id.unwrap_or_default(),
            receipt_handle: m.receipt_handle.unwrap_or_default(),
            body: m.body.unwrap_or_default(),
        })
        .collect())
}

async fn delete(
    sqs: &aws_sdk_sqs::Client,
    queue_url: &str,
    messages: &[&Message],
) -> anyhow::Result<()> {
    if messages.is_empty() {
        return Ok(());
    }
    let entries = messages
        .iter()
        .enumerate()
        .map(|(i, m)| {
            DeleteMessageBatchRequestEntry::builder()
                .id(i.to_string())
                .receipt_handle(&m.receipt_handle)
                .build()
                .map_err(|e| anyhow::anyhow!(e.to_string()))
        })
        .collect::<anyhow::Result<Vec<_>>>()?;
    let output = sqs
        .delete_message_batch()
        .queue_url(queue_url)
        .set_entries(Some(entries))
        .send()
        .await
        .map_err(|e| anyhow::anyhow!(aws_sdk_sqs::error::DisplayErrorContext(e).to_string()))?;
    anyhow::ensure!(output.failed.is_empty(), "{} messages were not deleted", output.failed.len());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn message(id: &str, group: &str, body: &str) -> Message {
        Message {
            id: id.into(),
            receipt_handle: String::new(),
            group: group.into(),
            body: body.into(),
            traceparent: None,
        }
    }

    /// 処理した本文を覚え、本文が "fail" なら失敗する
    #[derive(Default)]
    struct Recorder(std::sync::Mutex<Vec<String>>);

    #[async_trait]
    impl Handler for Recorder {
        async fn handle(&self, body: &str) -> anyhow::Result<()> {
            self.0.lock().unwrap().push(body.to_owned());
            if body == "fail" { Err(anyhow::anyhow!("down")) } else { Ok(()) }
        }
    }

    #[tokio::test]
    async fn a_failure_holds_back_only_the_rest_of_its_group() {
        let messages = [
            message("1", "payslip-1", "ok"),
            message("2", "payslip-2", "fail"),
            message("3", "payslip-1", "ok"),
            message("4", "payslip-2", "ok"),
            message("5", "payslip-3", "ok"),
        ];
        let recorder = Recorder::default();

        let failed = handle_batch(&messages, &recorder).await;

        // 失敗したグループの後ろの件(4)は処理せずに返し、他のグループは処理を続ける
        assert_eq!(failed, ["2", "4"]);
        assert_eq!(recorder.0.lock().unwrap().len(), 4);
    }
}
