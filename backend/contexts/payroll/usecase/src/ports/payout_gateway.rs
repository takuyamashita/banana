use async_trait::async_trait;
use payroll_domain::staff::StaffId;
use platform_kernel::Money;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PayoutId(pub String);

#[derive(Debug, Error)]
pub enum PayoutError {
    /// 口座不備など、再試行しても通らない
    #[error("振込が拒否されました: {0}")]
    Rejected(String),
    /// 通信断など、再試行すれば通りうる
    #[error("振込APIが利用できません: {0}")]
    Unavailable(String),
}

// 振込はSQS経由でconsumer側から呼ぶ。確定のユースケースはこれに依存しない。
// idempotency_key は同じイベントの再配信で二重に振り込まないための鍵(outbox の id)
#[async_trait]
pub trait PayoutGateway: Send + Sync {
    async fn request_transfer(
        &self,
        staff_id: StaffId,
        amount: Money,
        idempotency_key: &str,
    ) -> Result<PayoutId, PayoutError>;
}
