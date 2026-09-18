// SQSに流すメッセージの封筒。event_id は outbox.id で、consumerの二重処理判定に使う。
// キューの形はinfrastructureの関心事なので、domainには置かない
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct OutboxEnvelope<P> {
    pub event_id: i64,
    pub payload: P,
}
