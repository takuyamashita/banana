//! usecase の単体テスト。リポジトリと外部依存はインメモリのフェイクで差し替える

// allow-unwrap-in-tests は #[test] 関数の中にしか効かず、tests/ のフェイクや補助関数は対象外
#![allow(clippy::unwrap_used)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use payroll_domain::payslip::{
    NewPayslip, PayPeriod, Payslip, PayslipEvent, PayslipId, PayslipLine, WorkMinutes,
};
use payroll_domain::project::ProjectId;
use payroll_domain::staff::{DisplayName, Email, NewStaff, Staff, StaffId, UserId};
use payroll_usecase::UseCaseError;
use payroll_usecase::payslip::{FinalizePayslipInput, FinalizePayslipUseCase, GetPayslipUseCase};
use payroll_usecase::ports::repository::{PayslipRepository, RepositoryError, StaffRepository};
use payroll_usecase::ports::user_directory::{UserDirectory, UserDirectoryError};
use payroll_usecase::staff::{CreateStaffInput, CreateStaffUseCase};
use platform_kernel::{AuthenticatedUser, Money, Role};

// ---- フェイク ----
// フェイクも「リポジトリ実装」なので reconstruct を呼ぶ必要がある。usecase の clippy.toml は
// crate 全体(tests/ も含む)に効くため、ここだけ明示的に許可する

type PayslipRow = (PayslipId, StaffId, PayPeriod, Vec<PayslipLine>);

#[derive(Default)]
struct FakePayslips {
    rows: Mutex<Vec<PayslipRow>>,
    events: Mutex<Vec<PayslipEvent>>,
}

#[async_trait]
impl PayslipRepository for FakePayslips {
    async fn insert(&self, new: &mut NewPayslip) -> Result<Payslip, RepositoryError> {
        let mut rows = self.rows.lock().unwrap();
        let id = PayslipId::from_i64(i64::try_from(rows.len()).unwrap() + 1).unwrap();
        rows.push((id, new.staff_id(), new.period(), new.lines().to_vec()));
        self.events.lock().unwrap().extend(new.take_events());
        #[allow(clippy::disallowed_methods, reason = "フェイクのリポジトリ実装")]
        Ok(Payslip::reconstruct(
            id,
            new.staff_id(),
            new.period(),
            new.lines().to_vec(),
            new.status(),
        )
        .unwrap())
    }

    async fn update(&self, _payslip: &mut Payslip) -> Result<(), RepositoryError> {
        unimplemented!()
    }

    async fn find(&self, id: PayslipId) -> Result<Option<Payslip>, RepositoryError> {
        Ok(self.rows.lock().unwrap().iter().find(|r| r.0 == id).map(to_payslip))
    }

    async fn list_by_staff(&self, staff_id: StaffId) -> Result<Vec<Payslip>, RepositoryError> {
        Ok(self.rows.lock().unwrap().iter().filter(|r| r.1 == staff_id).map(to_payslip).collect())
    }
}

#[allow(clippy::disallowed_methods, reason = "フェイクのリポジトリ実装")]
fn to_payslip(r: &PayslipRow) -> Payslip {
    Payslip::reconstruct(
        r.0,
        r.1,
        r.2,
        r.3.clone(),
        payroll_domain::payslip::PayslipStatus::Finalized,
    )
    .unwrap()
}

#[derive(Default)]
struct FakeStaff {
    rows: Mutex<Vec<(StaffId, UserId, Email)>>,
    fail_insert: bool,
}

impl FakeStaff {
    fn with(rows: &[(i64, &str)]) -> Self {
        let rows = rows
            .iter()
            .map(|(id, user)| {
                (
                    StaffId::from_i64(*id).unwrap(),
                    UserId::parse(*user).unwrap(),
                    Email::parse(format!("{user}@example.com")).unwrap(),
                )
            })
            .collect();
        Self { rows: Mutex::new(rows), fail_insert: false }
    }
}

#[allow(clippy::disallowed_methods, reason = "フェイクのリポジトリ実装")]
fn to_staff(r: &(StaffId, UserId, Email)) -> Staff {
    Staff::reconstruct(r.0, r.1.clone(), r.2.clone(), DisplayName::new("x").unwrap())
}

#[async_trait]
impl StaffRepository for FakeStaff {
    async fn insert(&self, new: &NewStaff) -> Result<StaffId, RepositoryError> {
        if self.fail_insert {
            return Err(RepositoryError::Unavailable("db down".into()));
        }
        let mut rows = self.rows.lock().unwrap();
        let id = StaffId::from_i64(i64::try_from(rows.len()).unwrap() + 1).unwrap();
        rows.push((id, new.user_id().clone(), new.email().clone()));
        Ok(id)
    }

    async fn find(&self, id: StaffId) -> Result<Option<Staff>, RepositoryError> {
        Ok(self.rows.lock().unwrap().iter().find(|r| r.0 == id).map(to_staff))
    }

    async fn find_by_user_id(&self, user_id: &UserId) -> Result<Option<Staff>, RepositoryError> {
        Ok(self.rows.lock().unwrap().iter().find(|r| &r.1 == user_id).map(to_staff))
    }

    async fn find_by_email(&self, email: &Email) -> Result<Option<Staff>, RepositoryError> {
        Ok(self.rows.lock().unwrap().iter().find(|r| &r.2 == email).map(to_staff))
    }
}

#[derive(Default)]
struct FakeDirectory {
    disabled: Mutex<Vec<UserId>>,
    created: Mutex<HashMap<String, UserId>>,
}

#[async_trait]
impl UserDirectory for FakeDirectory {
    async fn create_user(&self, email: &Email, _pw: &str) -> Result<UserId, UserDirectoryError> {
        let id = UserId::parse(format!("sub-{}", email.as_str())).unwrap();
        self.created.lock().unwrap().insert(email.as_str().to_owned(), id.clone());
        Ok(id)
    }

    async fn disable_user(&self, id: &UserId) -> Result<(), UserDirectoryError> {
        self.disabled.lock().unwrap().push(id.clone());
        Ok(())
    }
}

// ---- ヘルパー ----

fn line() -> PayslipLine {
    PayslipLine::new(
        ProjectId::from_i64(1).unwrap(),
        WorkMinutes::from_minutes(600).unwrap(),
        Money::from_yen(1_200).unwrap(),
    )
}

fn input(staff: i64, month: u8) -> FinalizePayslipInput {
    FinalizePayslipInput {
        staff_id: StaffId::from_i64(staff).unwrap(),
        period: PayPeriod::new(2026, month).unwrap(),
        lines: vec![line()],
    }
}

fn user(sub: &str, roles: &[Role]) -> AuthenticatedUser {
    AuthenticatedUser { user_id: sub.into(), roles: roles.to_vec() }
}

// ---- テスト ----

#[tokio::test]
async fn finalize_records_event_through_repository() {
    let payslips = Arc::new(FakePayslips::default());
    let staff = Arc::new(FakeStaff::with(&[(1, "taro")]));
    let usecase = FinalizePayslipUseCase::new(payslips.clone(), staff);

    usecase.execute(input(1, 9)).await.unwrap();

    let events = payslips.events.lock().unwrap();
    assert!(matches!(
        events.as_slice(),
        [PayslipEvent::Finalized { total, .. }] if total.as_yen() == 12_000
    ));
}

#[tokio::test]
async fn finalize_rejects_unknown_staff() {
    let usecase = FinalizePayslipUseCase::new(
        Arc::new(FakePayslips::default()),
        Arc::new(FakeStaff::default()),
    );
    let err = usecase.execute(input(1, 9)).await.unwrap_err();
    assert!(matches!(err, UseCaseError::InvalidInput(_)));
}

#[tokio::test]
async fn finalize_rejects_same_month_twice() {
    let usecase = FinalizePayslipUseCase::new(
        Arc::new(FakePayslips::default()),
        Arc::new(FakeStaff::with(&[(1, "taro")])),
    );
    usecase.execute(input(1, 9)).await.unwrap();
    let err = usecase.execute(input(1, 9)).await.unwrap_err();
    assert!(matches!(err, UseCaseError::Conflict(_)));
    usecase.execute(input(1, 10)).await.unwrap();
}

#[tokio::test]
async fn only_admin_or_owner_can_view_payslip() {
    let payslips = Arc::new(FakePayslips::default());
    let staff = Arc::new(FakeStaff::with(&[(1, "taro"), (2, "hanako")]));
    let id = FinalizePayslipUseCase::new(payslips.clone(), staff.clone())
        .execute(input(1, 9))
        .await
        .unwrap();
    let get = GetPayslipUseCase::new(payslips, staff);

    assert!(get.execute(&user("admin", &[Role::Admin]), id).await.is_ok());
    assert!(get.execute(&user("taro", &[]), id).await.is_ok());
    assert!(matches!(
        get.execute(&user("hanako", &[Role::Staff]), id).await,
        Err(UseCaseError::NotFound)
    ));
    // staff に紐づかない利用者も見られない
    assert!(matches!(
        get.execute(&user("stranger", &[Role::Staff]), id).await,
        Err(UseCaseError::NotFound)
    ));
}

#[tokio::test]
async fn create_staff_disables_user_when_insert_fails() {
    let directory = Arc::new(FakeDirectory::default());
    let staff = Arc::new(FakeStaff { fail_insert: true, ..FakeStaff::default() });
    let usecase = CreateStaffUseCase::new(staff, directory.clone());

    let err = usecase
        .execute(CreateStaffInput {
            email: Email::parse("new@example.com").unwrap(),
            display_name: DisplayName::new("新人").unwrap(),
            temporary_password: "Temp-pass-1".into(),
        })
        .await
        .unwrap_err();

    assert!(matches!(err, UseCaseError::Unavailable(_)));
    assert_eq!(
        directory.disabled.lock().unwrap().as_slice(),
        [UserId::parse("sub-new@example.com").unwrap()]
    );
}
