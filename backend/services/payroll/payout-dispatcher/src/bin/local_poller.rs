//! ローカル専用。ElasticMQ の振込のキューを読んで、Lambda と同じ処理を呼ぶ。
//! AWS では SQS のイベントソースマッピングがこの役を担う

use payout_dispatcher::{Consumer, Deps};
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = payroll_bootstrap::load_config()?;
    let _telemetry = payroll_bootstrap::init_telemetry(&config, "payout-dispatcher-local")?;
    let aws = payroll_bootstrap::aws_config().await;
    let sqs = platform_messaging::sqs_client(&aws, &config.messaging.sqs_endpoint);
    let queue_url = config.payout.queue_url.clone();
    anyhow::ensure!(!queue_url.is_empty(), "payout.queue_url is not configured");
    let consumer = Consumer(Deps::build().await?);
    tracing::info!(queue = %queue_url, "polling");

    let cancel = CancellationToken::new();
    tokio::spawn({
        let cancel = cancel.clone();
        async move {
            let _ = tokio::signal::ctrl_c().await;
            cancel.cancel();
        }
    });
    platform_messaging::consumer::run(sqs, queue_url, consumer, cancel).await;
    Ok(())
}
