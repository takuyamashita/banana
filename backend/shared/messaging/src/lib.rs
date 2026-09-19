//! サービスの間の出来事の送受信。
//!
//! 送る側: 出来事は業務の記録と同じトランザクションで outbox テーブルに書き、[`relay`] が後から
//! [`Publisher`](publisher::Publisher) で送る。AWS は SNS の FIFO トピック、ローカルは受け手の SQS キューへ直接送る。
//! 受ける側: 自分の SQS キューを [`consumer`] が読み、1件ずつ処理する。
//!
//! どちらも SQS・SNS の「少なくとも1回」届ける性質の上にあるので、受け手は同じ出来事が2回届いても
//! 結果が変わらないように作る

pub mod consumer;
pub mod envelope;
pub mod publisher;
pub mod relay;

use std::time::Duration;

/// SQS のクライアント。呼び出しにタイムアウトを付ける(SDK の既定は接続のタイムアウトだけで、
/// 応答が返らないと relay がトランザクションと接続を持ったまま待ち続ける)。
/// `endpoint` はローカル(ElasticMQ)のときだけ指定する
pub fn sqs_client(aws: &aws_config::SdkConfig, endpoint: &str) -> aws_sdk_sqs::Client {
    let mut builder = aws_sdk_sqs::config::Builder::from(aws).timeout_config(timeouts());
    if !endpoint.is_empty() {
        builder = builder.endpoint_url(endpoint);
    }
    aws_sdk_sqs::Client::from_conf(builder.build())
}

/// SNS のクライアント。タイムアウトは SQS と同じ
pub fn sns_client(aws: &aws_config::SdkConfig) -> aws_sdk_sns::Client {
    aws_sdk_sns::Client::from_conf(
        aws_sdk_sns::config::Builder::from(aws).timeout_config(timeouts()).build(),
    )
}

fn timeouts() -> aws_config::timeout::TimeoutConfig {
    aws_config::timeout::TimeoutConfig::builder()
        .connect_timeout(Duration::from_secs(2))
        .operation_attempt_timeout(Duration::from_secs(5))
        .operation_timeout(Duration::from_secs(10))
        .build()
}
