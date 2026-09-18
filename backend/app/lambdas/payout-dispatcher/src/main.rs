use aws_lambda_events::sqs::SqsEvent;
use lambda_runtime::{Error, LambdaEvent, service_fn};
use payout_dispatcher::{Deps, process};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let config = bootstrap::load_config()?;
    let _telemetry = bootstrap::init_telemetry(&config, "payout-dispatcher")?;

    // DB 接続などは run の外で1回だけ行い、コールドスタートを抑える
    let deps = Deps::build().await?;
    let deps = &deps;

    lambda_runtime::run(service_fn(move |event: LambdaEvent<SqsEvent>| async move {
        // 1件でも失敗したらバッチ全体を失敗させる。FIFO なので後続も同じグループで待たされ、
        // 再配信時は処理済みの分を冪等性で読み飛ばす
        for record in event.payload.records {
            let body = record.body.unwrap_or_default();
            process(deps, &body).await?;
        }
        Ok::<(), Error>(())
    }))
    .await
}
