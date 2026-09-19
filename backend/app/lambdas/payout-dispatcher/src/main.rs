use aws_lambda_events::sqs::{SqsBatchResponse, SqsEvent};
use lambda_runtime::{Error, LambdaEvent, service_fn};
use payout_dispatcher::{Deps, Message, handle_batch, process};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let config = bootstrap::load_config()?;
    let _telemetry = bootstrap::init_telemetry(&config, "payout-dispatcher")?;

    // DB 接続などは run の外で1回だけ行い、コールドスタートを抑える
    let deps = Deps::build().await?;
    let deps = &deps;

    lambda_runtime::run(service_fn(move |event: LambdaEvent<SqsEvent>| async move {
        let messages: Vec<Message> = event
            .payload
            .records
            .into_iter()
            .map(|record| Message {
                id: record.message_id.unwrap_or_default(),
                group: record.attributes.get("MessageGroupId").cloned().unwrap_or_default(),
                body: record.body.unwrap_or_default(),
            })
            .collect();

        // 失敗した件だけを返してキューに戻す(イベントソースの ReportBatchItemFailures)。
        // 成功した件と、失敗と関係のない給与明細の件は再配信されない
        let mut response = SqsBatchResponse::default();
        for id in handle_batch(&messages, async |body: &str| process(deps, body).await).await {
            response.add_failure(id);
        }
        Ok::<_, Error>(response)
    }))
    .await
}
