//! 出来事の送り先。relay は送り先を知らず、[`Publisher`] に渡すだけにする

use async_trait::async_trait;
use aws_sdk_sqs::types::MessageAttributeValue as SqsAttribute;
use thiserror::Error;

/// 送る1件
#[derive(Debug, Clone, Copy)]
pub struct Outgoing<'a> {
    /// 順序を保つ単位(同じ集約の出来事は同じグループ)。FIFO のメッセージグループになる
    pub group: &'a str,
    /// 重複を除く鍵(outbox.id)。送り直しても、受け手のキューには1回だけ入る(除去の窓は5分)
    pub deduplication_id: &'a str,
    /// 出来事の種類。受け手のキューは、SNS の購読のフィルターでこの値を見て必要なものだけを受ける
    pub event_type: &'a str,
    /// 封筒(JSON)
    pub body: &'a str,
    /// 出来事を記録したリクエストのトレース。受け手の処理をその続きにする
    pub traceparent: Option<&'a str>,
}

#[derive(Debug, Error)]
#[error("出来事を送れませんでした: {0}")]
pub struct PublishError(pub String);

#[async_trait]
pub trait Publisher: Send + Sync {
    async fn publish(&self, message: &Outgoing<'_>) -> Result<(), PublishError>;
}

/// SNS の FIFO トピックに送る(AWS)。受け手ごとの SQS キューへの配り分けは、トピックの購読が行う
pub struct SnsPublisher {
    client: aws_sdk_sns::Client,
    topic_arn: String,
}

impl SnsPublisher {
    pub fn new(client: aws_sdk_sns::Client, topic_arn: String) -> Self {
        Self { client, topic_arn }
    }
}

#[async_trait]
impl Publisher for SnsPublisher {
    async fn publish(&self, message: &Outgoing<'_>) -> Result<(), PublishError> {
        let attribute = |value: &str| {
            aws_sdk_sns::types::MessageAttributeValue::builder()
                .data_type("String")
                .string_value(value)
                .build()
                .map_err(|e| PublishError(e.to_string()))
        };
        let mut request = self
            .client
            .publish()
            .topic_arn(&self.topic_arn)
            .message(message.body)
            .message_group_id(message.group)
            .message_deduplication_id(message.deduplication_id)
            .message_attributes("event_type", attribute(message.event_type)?);
        if let Some(traceparent) = message.traceparent {
            request = request.message_attributes("traceparent", attribute(traceparent)?);
        }
        request
            .send()
            .await
            .map_err(|e| PublishError(aws_sdk_sns::error::DisplayErrorContext(e).to_string()))?;
        Ok(())
    }
}

/// 受け手の SQS キューへ直接送る(ローカル。ElasticMQ には SNS がないので、トピックの代わりに配る)。
///
/// 購読のフィルターはないので、どのキューにもすべての出来事を送る(受け手は知らない種類を読み飛ばす)。
/// 途中のキューで失敗したら失敗を返し、relay が後で全部を送り直す(送れていたキューには重複を除く鍵で1回だけ入る)
pub struct SqsFanoutPublisher {
    client: aws_sdk_sqs::Client,
    queue_urls: Vec<String>,
}

impl SqsFanoutPublisher {
    pub fn new(client: aws_sdk_sqs::Client, queue_urls: Vec<String>) -> Self {
        Self { client, queue_urls }
    }
}

#[async_trait]
impl Publisher for SqsFanoutPublisher {
    async fn publish(&self, message: &Outgoing<'_>) -> Result<(), PublishError> {
        let attribute = |value: &str| {
            SqsAttribute::builder()
                .data_type("String")
                .string_value(value)
                .build()
                .map_err(|e| PublishError(e.to_string()))
        };
        for queue_url in &self.queue_urls {
            let mut request = self
                .client
                .send_message()
                .queue_url(queue_url)
                .message_body(message.body)
                .message_group_id(message.group)
                .message_deduplication_id(message.deduplication_id)
                .message_attributes("event_type", attribute(message.event_type)?);
            if let Some(traceparent) = message.traceparent {
                request = request.message_attributes("traceparent", attribute(traceparent)?);
            }
            request.send().await.map_err(|e| {
                PublishError(aws_sdk_sqs::error::DisplayErrorContext(e).to_string())
            })?;
        }
        Ok(())
    }
}
