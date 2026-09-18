use std::sync::Arc;

use payroll_domain::payslip::{NewPayslip, PayPeriod, PayslipId, PayslipLine, PayslipRepository};
use payroll_domain::staff::{StaffId, StaffRepository};

use crate::UseCaseError;

pub struct FinalizePayslipInput {
    pub staff_id: StaffId,
    pub period: PayPeriod,
    pub lines: Vec<PayslipLine>, // handlerで検証済みのdomain型
}

pub struct FinalizePayslipUseCase {
    repository: Arc<dyn PayslipRepository>,
    staff_repository: Arc<dyn StaffRepository>,
}

impl FinalizePayslipUseCase {
    #[must_use]
    pub fn new(
        repository: Arc<dyn PayslipRepository>,
        staff_repository: Arc<dyn StaffRepository>,
    ) -> Self {
        Self { repository, staff_repository }
    }

    pub async fn execute(&self, input: FinalizePayslipInput) -> Result<PayslipId, UseCaseError> {
        // 存在しない派遣社員への確定を弾く。集約をまたいだ外部キーは張らないので、アプリ側で確かめる
        if self.staff_repository.find(input.staff_id).await?.is_none() {
            return Err(UseCaseError::InvalidInput("派遣社員が存在しません".into()));
        }

        // 親切なエラーを返すための事前チェック。同時実行の最終防衛線は DB のユニーク制約
        let existing = self.repository.list_by_staff(input.staff_id).await?;
        if existing.iter().any(|p| p.period() == input.period) {
            return Err(UseCaseError::Conflict("この月の給与明細は既に確定しています".into()));
        }

        // IDはまだ存在しないので NewPayslip を組み立てる。
        // draft が不変条件(明細が1件以上)を検証する
        let mut new = NewPayslip::draft(input.staff_id, input.period, input.lines)?;

        // 公開setterではなく finalize で状態遷移ルールを通す。
        // このとき集約が PayslipEvent::Finalized を内部に記録する
        new.finalize()?;

        // insert が採番し、集約とイベント(outbox)を同じトランザクションで書く。
        // usecaseはoutboxもSQSも採番方式も知らない
        let payslip = self.repository.insert(&mut new).await?;

        Ok(payslip.id())
    }
}
