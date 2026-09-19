/// 振込先の答え
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PayoutOutcome {
    /// 受け付けられた。受付番号は振込先が発行し、振込先への問い合わせに使う
    Accepted { receipt: String },
    /// 断られた(口座の不備など)。同じ依頼をやり直しても通らないので、理由を確かめて対処する
    Rejected { reason: String },
}
