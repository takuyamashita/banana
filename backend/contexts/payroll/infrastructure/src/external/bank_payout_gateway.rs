use async_trait::async_trait;
use payroll_domain::staff::StaffId;
use payroll_usecase::ports::payout_gateway::{PayoutError, PayoutGateway, PayoutReceipt};
use platform_kernel::Money;
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
        if status.is_client_error() {
            let body = res.text().await.unwrap_or_default();
            return Err(PayoutError::Rejected(format!("{status}: {body}")));
        }
        if !status.is_success() {
            return Err(PayoutError::Unavailable(status.to_string()));
        }
        let body: TransferResponse = res.json().await.map_err(unavailable)?;
        Ok(PayoutReceipt(body.id))
    }
}
