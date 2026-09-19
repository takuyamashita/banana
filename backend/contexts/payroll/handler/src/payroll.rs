use payroll_domain::payslip::{
    PayPeriod, Payslip, PayslipId, PayslipLine, PayslipStatus, WorkMinutes,
};
use payroll_domain::project::ProjectId;
use payroll_domain::staff::StaffId;
use payroll_usecase::payslip::{
    CreatePayslipInput, CreatePayslipUseCase, FinalizePayslipUseCase, GetPayslipUseCase,
    ListPayslipsUseCase,
};
use platform_gen::acme::payroll::v1 as proto;
use platform_kernel::Money;
use tonic::{Request, Response, Status};

use crate::auth::{current_user, require_admin};
use crate::error::{invalid_argument, to_status};

pub struct PayrollServiceHandler {
    create_payslip: CreatePayslipUseCase,
    finalize_payslip: FinalizePayslipUseCase,
    get_payslip: GetPayslipUseCase,
    list_payslips: ListPayslipsUseCase,
}

impl PayrollServiceHandler {
    #[must_use]
    pub fn new(
        create_payslip: CreatePayslipUseCase,
        finalize_payslip: FinalizePayslipUseCase,
        get_payslip: GetPayslipUseCase,
        list_payslips: ListPayslipsUseCase,
    ) -> Self {
        Self { create_payslip, finalize_payslip, get_payslip, list_payslips }
    }
}

#[tonic::async_trait]
impl proto::payroll_service_server::PayrollService for PayrollServiceHandler {
    async fn create_payslip(
        &self,
        request: Request<proto::CreatePayslipRequest>,
    ) -> Result<Response<proto::CreatePayslipResponse>, Status> {
        // 給与明細の作成は管理者だけ。認証・認可の判定はhandler層の仕事。
        // AuthenticatedUser は認証ミドルウェアが extensions に載せている
        require_admin(&request)?;

        let req = request.into_inner();

        // protoのint64からdomainの型へ変換する。検証に失敗したらInvalidArgumentで返す
        let staff_id = StaffId::from_i64(req.staff_id).map_err(invalid_argument)?;
        // as キャストは範囲外の値を黙って丸めるので、try_from で検証する
        let pay_year = u16::try_from(req.pay_year).map_err(invalid_argument)?;
        let pay_month = u8::try_from(req.pay_month).map_err(invalid_argument)?;
        let period = PayPeriod::new(pay_year, pay_month).map_err(invalid_argument)?;

        let lines = req
            .lines
            .into_iter()
            .map(|l| -> Result<PayslipLine, Status> {
                let project_id = ProjectId::from_i64(l.project_id).map_err(invalid_argument)?;
                let work_minutes =
                    WorkMinutes::from_minutes(l.work_minutes).map_err(invalid_argument)?;
                let hourly_rate = Money::from_yen(l.hourly_rate).map_err(invalid_argument)?;
                Ok(PayslipLine::new(project_id, work_minutes, hourly_rate))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let payslip_id = self
            .create_payslip
            .execute(CreatePayslipInput { staff_id, period, lines })
            .await
            .map_err(to_status)?;

        Ok(Response::new(proto::CreatePayslipResponse { payslip_id: payslip_id.as_i64() }))
    }

    async fn finalize_payslip(
        &self,
        request: Request<proto::FinalizePayslipRequest>,
    ) -> Result<Response<proto::FinalizePayslipResponse>, Status> {
        // 給与の確定は管理者だけ
        require_admin(&request)?;
        let id = PayslipId::from_i64(request.get_ref().payslip_id).map_err(invalid_argument)?;

        self.finalize_payslip.execute(id).await.map_err(to_status)?;

        Ok(Response::new(proto::FinalizePayslipResponse {}))
    }

    async fn get_payslip(
        &self,
        request: Request<proto::GetPayslipRequest>,
    ) -> Result<Response<proto::GetPayslipResponse>, Status> {
        let user = current_user(&request)?;
        let id = PayslipId::from_i64(request.get_ref().payslip_id).map_err(invalid_argument)?;

        let payslip = self.get_payslip.execute(&user, id).await.map_err(to_status)?;

        Ok(Response::new(proto::GetPayslipResponse { payslip: Some(to_proto(&payslip)) }))
    }

    async fn list_payslips(
        &self,
        request: Request<proto::ListPayslipsRequest>,
    ) -> Result<Response<proto::ListPayslipsResponse>, Status> {
        let user = current_user(&request)?;
        let staff_id = StaffId::from_i64(request.get_ref().staff_id).map_err(invalid_argument)?;

        let payslips = self.list_payslips.execute(&user, staff_id).await.map_err(to_status)?;

        Ok(Response::new(proto::ListPayslipsResponse {
            payslips: payslips.iter().map(to_proto).collect(),
        }))
    }
}

fn to_proto(p: &Payslip) -> proto::Payslip {
    let c = p.content();
    proto::Payslip {
        payslip_id: c.id().as_i64(),
        staff_id: c.staff_id().as_i64(),
        pay_year: i32::from(c.period().year()),
        pay_month: i32::from(c.period().month()),
        status: match p.status() {
            PayslipStatus::Draft => proto::PayslipStatus::Draft,
            PayslipStatus::Finalized => proto::PayslipStatus::Finalized,
        }
        .into(),
        total_yen: c.total().as_yen(),
        lines: c
            .lines()
            .iter()
            .map(|l| proto::PayslipLine {
                project_id: l.project_id().as_i64(),
                work_minutes: l.work_minutes().as_minutes(),
                hourly_rate: l.hourly_rate().as_yen(),
                amount_yen: l.amount().as_yen(),
            })
            .collect(),
    }
}
