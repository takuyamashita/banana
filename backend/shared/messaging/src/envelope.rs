// キューに流すメッセージの封筒。キューの形は infrastructure の関心事なので、domain には置かない。
// 受け手は event_type でペイロードの形を選ぶ。ペイロードの互換のない変更は、新しい event_type で出す
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct OutboxEnvelope<P> {
    /// outbox.id。同じ出来事が2回以上届いたときの見分けに使う(送ったサービスの中だけで一意)
    pub event_id: i64,
    /// 出来事の種類(payslip.finalized など)
    pub event_type: String,
    /// 出来事が起きた集約の種類と番号
    pub aggregate_type: String,
    pub aggregate_id: i64,
    pub payload: P,
}
