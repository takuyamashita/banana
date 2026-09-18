use async_trait::async_trait;
use payroll_domain::staff::StaffId;
use platform_kernel::Money;
use thiserror::Error;

/// 振込依頼の受付番号。振込先が発行する
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PayoutId(pub String);

/// 振込を依頼できなかった理由
#[derive(Debug, Error)]
pub enum PayoutError {
    /// 口座の不備などで断られた。同じ依頼をやり直しても通らない
    #[error("振込が拒否されました: {0}")]
    Rejected(String),
    /// 振込先に一時的につながらない。時間をおいてやり直せば通りうる
    #[error("振込APIが利用できません: {0}")]
    Unavailable(String),
}

/// 派遣社員の口座への振込を依頼する先(銀行などの振込サービス)
#[async_trait]
pub trait PayoutGateway: Send + Sync {
    /// 派遣社員に金額を振り込むよう依頼する。
    ///
    /// `idempotency_key` は依頼ごとの番号で、同じ番号の依頼は何度届いても1回しか振り込まれない。
    /// 同じ給与確定について依頼をやり直しても、二重に振り込まれることはない
    async fn request_transfer(
        &self,
        staff_id: StaffId,
        amount: Money,
        idempotency_key: &str,
    ) -> Result<PayoutId, PayoutError>;
}
