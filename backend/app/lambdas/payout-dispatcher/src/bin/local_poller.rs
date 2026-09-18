//! ローカル専用。ElasticMQ をポーリングして Lambda と同じ処理を呼ぶ。
//! AWS では SQS のイベントソースマッピングがこの役を担う

use payout_dispatcher::{Deps, process};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = bootstrap::load_config()?;
    let _telemetry = bootstrap::init_telemetry(&config, "payout-dispatcher-local")?;
    let aws = bootstrap::aws_config().await;
    let sqs = bootstrap::sqs_client(&config, &aws);
    let deps = Deps::build().await?;
    tracing::info!(queue = %config.messaging.queue_url, "polling");

    loop {
        let out = sqs
            .receive_message()
            .queue_url(&config.messaging.queue_url)
            .max_number_of_messages(10)
            .wait_time_seconds(10)
            .send()
            .await?;

        for message in out.messages() {
            match process(&deps, message.body().unwrap_or_default()).await {
                Ok(()) => {
                    sqs.delete_message()
                        .queue_url(&config.messaging.queue_url)
                        .receipt_handle(message.receipt_handle().unwrap_or_default())
                        .send()
                        .await?;
                }
                // 消さずに残すと可視性タイムアウト後に再配信される
                Err(err) => tracing::warn!(error = %err, "processing failed, will retry"),
            }
        }
    }
}
