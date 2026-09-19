use async_trait::async_trait;
use payroll_domain::staff::StaffId;
use payroll_usecase::ports::payout_gateway::{PayoutError, PayoutGateway, PayoutReceipt};
use platform_kernel::Money;
use reqwest::StatusCode;
use serde::Deserialize;

pub struct BankPayoutGateway {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
}

impl BankPayoutGateway {
    #[must_use]
    pub fn new(client: reqwest::Client, base_url: String, api_key: String) -> Self {
        Self { client, base_url, api_key }
    }
}

#[derive(Deserialize)]
struct TransferResponse {
    id: String,
}

// reqwest::Error → PayoutError も From では書けない(どちらも外部の型で孤児ルールに触れる)
fn unavailable(err: impl std::fmt::Display) -> PayoutError {
    PayoutError::Unavailable(err.to_string())
}

#[async_trait]
impl PayoutGateway for BankPayoutGateway {
    async fn request_transfer(
        &self,
        staff_id: StaffId,
        amount: Money,
        idempotency_key: &str,
    ) -> Result<PayoutReceipt, PayoutError> {
        let res = self
            .client
            .post(format!("{}/transfers", self.base_url))
            .bearer_auth(&self.api_key)
            .header("Idempotency-Key", idempotency_key)
            .json(&serde_json::json!({
                "staff_id": staff_id.as_i64(),
                "amount": amount.as_yen(),
            }))
            .send()
            .await
            .map_err(unavailable)?;

        let status = res.status();
        if status.is_success() {
            let body: TransferResponse = res.json().await.map_err(unavailable)?;
            return Ok(PayoutReceipt(body.id));
        }
        let body = res.text().await.unwrap_or_default();
        Err(classify(status, &body))
    }
}

/// 振込 API の失敗を、やり直して通りうるかで分ける
fn classify(status: StatusCode, body: &str) -> PayoutError {
    match status {
        // 依頼の中身(口座など)が受け付けられない。やり直しても通らない
        StatusCode::BAD_REQUEST | StatusCode::NOT_FOUND | StatusCode::UNPROCESSABLE_ENTITY => {
            PayoutError::Rejected(format!("{status}: {body}"))
        }
        // キーの誤りなど、こちらの設定の問題。断られた振込として記録せず、再試行と DLQ で気づけるようにする
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
            tracing::error!(%status, "payout API rejected our credentials");
            PayoutError::Unavailable(format!("{status}: 振込 API の認証に失敗しました"))
        }
        // 408・409(同じ冪等キーの依頼を処理中)・429・5xx など。時間をおけば通りうる
        _ => PayoutError::Unavailable(format!("{status}: {body}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_rejections_of_the_request_itself_are_final() {
        for status in
            [StatusCode::BAD_REQUEST, StatusCode::NOT_FOUND, StatusCode::UNPROCESSABLE_ENTITY]
        {
            assert!(matches!(classify(status, ""), PayoutError::Rejected(_)), "{status}");
        }
        for status in [
            StatusCode::UNAUTHORIZED,
            StatusCode::FORBIDDEN,
            StatusCode::REQUEST_TIMEOUT,
            StatusCode::CONFLICT,
            StatusCode::TOO_MANY_REQUESTS,
            StatusCode::INTERNAL_SERVER_ERROR,
            StatusCode::SERVICE_UNAVAILABLE,
        ] {
            assert!(matches!(classify(status, ""), PayoutError::Unavailable(_)), "{status}");
        }
    }
}
