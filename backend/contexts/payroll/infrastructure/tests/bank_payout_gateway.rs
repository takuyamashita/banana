//! 振込 API のアダプタのテスト。決めた応答を返すだけの HTTP サーバーを立てて、送る依頼と応答の読み方を確かめる

// allow-unwrap-in-tests は #[test] 関数の中にしか効かず、補助関数は対象外
#![allow(clippy::unwrap_used)]

use std::sync::{Arc, Mutex};

use payroll_domain::staff::StaffId;
use payroll_infrastructure::external::BankPayoutGateway;
use payroll_usecase::ports::payout_gateway::{PayoutError, PayoutGateway};
use platform_kernel::Money;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

fn response(status: &str, body: &str) -> String {
    format!(
        "HTTP/1.1 {status}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
        body.len()
    )
}

/// 決めた応答を順に1回ずつ返す振込 API。受けた依頼(ヘッダを含む)を覚えておく
async fn fake_bank(responses: Vec<String>) -> (String, Arc<Mutex<Vec<String>>>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let received = Arc::new(Mutex::new(Vec::new()));
    let log = received.clone();
    tokio::spawn(async move {
        for response in responses {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buf = vec![0; 8192];
            let n = socket.read(&mut buf).await.unwrap();
            log.lock().unwrap().push(String::from_utf8_lossy(&buf[..n]).to_lowercase());
            socket.write_all(response.as_bytes()).await.unwrap();
        }
    });
    (format!("http://{addr}"), received)
}

#[tokio::test]
async fn sends_the_idempotency_key_and_reads_the_receipt_or_the_rejection() {
    let (base_url, received) = fake_bank(vec![
        response("201 Created", r#"{"id":"R-42"}"#),
        response("422 Unprocessable Entity", "account closed"),
        response("503 Service Unavailable", ""),
    ])
    .await;
    let gateway = BankPayoutGateway::new(reqwest::Client::new(), base_url, "key".into());
    let staff = StaffId::from_i64(1).unwrap();
    let amount = Money::from_yen(12_000).unwrap();

    let receipt =
        gateway.request_transfer(staff, amount, "payroll-payslip-1-finalized").await.unwrap();
    assert_eq!(receipt.0, "R-42");
    let request = received.lock().unwrap()[0].clone();
    // gitleaks の generic-api-key が「key と文字列」を API キーと誤検知するので、この行だけ外す
    assert!(request.contains("idempotency-key: payroll-payslip-1-finalized"), "{request}"); // gitleaks:allow
    assert!(request.contains("authorization: bearer key"), "{request}");

    // 依頼の中身が受け付けられないのは、やり直しても通らない
    let rejected = gateway.request_transfer(staff, amount, "k2").await.unwrap_err();
    assert!(matches!(rejected, PayoutError::Rejected(reason) if reason.contains("account closed")));

    // 振込 API が止まっているのは、時間をおけば通りうる
    let unavailable = gateway.request_transfer(staff, amount, "k3").await.unwrap_err();
    assert!(matches!(unavailable, PayoutError::Unavailable(_)));
}
