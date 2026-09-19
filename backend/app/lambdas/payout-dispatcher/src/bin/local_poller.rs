//! ローカル専用。ElasticMQ をポーリングして Lambda と同じ処理を呼ぶ。
//! AWS では SQS のイベントソースマッピングがこの役を担う

use std::time::Duration;

use aws_sdk_sqs::types::MessageSystemAttributeName;
use payout_dispatcher::{Deps, Message, handle_batch, process};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = bootstrap::load_config()?;
    let _telemetry = bootstrap::init_telemetry(&config, "payout-dispatcher-local")?;
    let aws = bootstrap::aws_config().await;
    let sqs = bootstrap::sqs_client(&config, &aws);
    let queue_url = config.messaging.queue_url.clone();
    let deps = Deps::build().await?;
    tracing::info!(queue = %queue_url, "polling");

    loop {
        let received = sqs
            .receive_message()
            .queue_url(&queue_url)
            .max_number_of_messages(10)
            .wait_time_seconds(10)
            .message_system_attribute_names(MessageSystemAttributeName::MessageGroupId)
            .message_attribute_names("traceparent")
            .send()
            .await;
        let out = match received {
            Ok(out) => out,
            Err(err) => {
                tracing::warn!(error = %aws_sdk_sqs::error::DisplayErrorContext(err), "receive failed");
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }
        };

        let messages: Vec<Message> = out
            .messages()
            .iter()
            .map(|m| Message {
                id: m.message_id().unwrap_or_default().to_owned(),
                group: m
                    .attributes()
                    .and_then(|a| a.get(&MessageSystemAttributeName::MessageGroupId))
                    .cloned()
                    .unwrap_or_default(),
                traceparent: m
                    .message_attributes()
                    .and_then(|a| a.get("traceparent"))
                    .and_then(|a| a.string_value())
                    .map(str::to_owned),
                body: m.body().unwrap_or_default().to_owned(),
            })
            .collect();
        let failed = handle_batch(&messages, async |body: &str| process(&deps, body).await).await;

        // Lambda と同じく、成功した件だけ消す。失敗した件は可視性タイムアウト後に再配信される
        for m in out.messages() {
            if failed.iter().any(|id| Some(id.as_str()) == m.message_id()) {
                continue;
            }
            let deleted = sqs
                .delete_message()
                .queue_url(&queue_url)
                .receipt_handle(m.receipt_handle().unwrap_or_default())
                .send()
                .await;
            if let Err(err) = deleted {
                tracing::warn!(error = %aws_sdk_sqs::error::DisplayErrorContext(err), "delete failed");
            }
        }
    }
}
