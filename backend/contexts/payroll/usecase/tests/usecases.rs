//! usecase の単体テスト。リポジトリと外部依存はインメモリのフェイクで差し替える

// allow-unwrap-in-tests は #[test] 関数の中にしか効かず、tests/ のフェイクや補助関数は対象外
#![allow(clippy::unwrap_used)]

use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use payroll_domain::payslip::{
    NewPayslip, PayPeriod, Payslip, PayslipEvent, PayslipId, PayslipLine, PayslipStatus,
    WorkMinutes,
};
use payroll_domain::project::ProjectId;
use payroll_domain::staff::{DisplayName, NewStaff, Staff, StaffId};
use payroll_usecase::UseCaseError;
use payroll_usecase::payslip::{FinalizePayslipInput, FinalizePayslipUseCase, GetPayslipUseCase};
use payroll_usecase::ports::database::{Database, Db, DbHandle, Transaction};
use payroll_usecase::ports::events::{EventOutbox, PayrollEvent};
use payroll_usecase::ports::repository::{PayslipRepository, RepositoryError, StaffRepository};
use payroll_usecase::ports::user_directory::{UserDirectory, UserDirectoryError};
use payroll_usecase::staff::{CreateStaffInput, CreateStaffUseCase};
use platform_kernel::{AuthenticatedUser, Email, Money, Role, UserId};

// ---- フェイク ----
// フェイクも「リポジトリ実装」なので reconstruct を呼ぶ必要がある。usecase の clippy.toml は
// crate 全体(tests/ も含む)に効くため、ここだけ明示的に許可する

type PayslipRow = (PayslipId, StaffId, PayPeriod, Vec<PayslipLine>);
type StaffRow = (StaffId, UserId, Email);

/// 確定済みの記録。取り出しはここを見て、トランザクションは commit でここへ反映する
#[derive(Default, Clone)]
struct Records {
    payslips: Vec<PayslipRow>,
    staff: Vec<StaffRow>,
    events: Vec<PayrollEvent>,
}

#[derive(Default)]
struct World {
    committed: Mutex<Records>,
    fail_staff_insert: bool,
    fail_event_append: bool,
}

impl World {
    fn with_staff(rows: &[(i64, &str)]) -> Self {
        let staff = rows
            .iter()
            .map(|(id, user)| {
                (
                    StaffId::from_i64(*id).unwrap(),
                    UserId::parse(*user).unwrap(),
                    Email::parse(format!("{user}@example.com")).unwrap(),
                )
            })
            .collect();
        Self { committed: Mutex::new(Records { staff, ..Records::default() }), ..Self::default() }
    }

    fn records(&self) -> Records {
        self.committed.lock().unwrap().clone()
    }
}

fn next_id(len: usize) -> i64 {
    i64::try_from(len).unwrap() + 1
}

#[allow(clippy::disallowed_methods, reason = "フェイクのリポジトリ実装")]
fn to_payslip(r: &PayslipRow) -> Payslip {
    Payslip::reconstruct(r.0, r.1, r.2, r.3.clone(), PayslipStatus::Finalized).unwrap()
}

#[allow(clippy::disallowed_methods, reason = "フェイクのリポジトリ実装")]
fn to_staff(r: &StaffRow) -> Staff {
    Staff::reconstruct(r.0, r.1.clone(), r.2.clone(), DisplayName::new("x").unwrap())
}

/// フェイクの書き込み先。トランザクションは commit まで記録を手元に溜めて commit で丸ごと反映し、
/// 接続は書いた時点で反映する
enum FakeDb {
    Transaction { world: Arc<World>, pending: Records },
    Connection(Arc<World>),
}

impl FakeDb {
    fn write<R>(&mut self, f: impl FnOnce(&mut Records) -> R) -> R {
        match self {
            Self::Transaction { pending, .. } => f(pending),
            Self::Connection(world) => f(&mut world.committed.lock().unwrap()),
        }
    }
}

#[async_trait]
impl DbHandle for FakeDb {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    async fn commit(self: Box<Self>) -> Result<(), RepositoryError> {
        if let Self::Transaction { world, pending } = *self {
            *world.committed.lock().unwrap() = pending;
        }
        Ok(())
    }
}

struct FakeDatabase(Arc<World>);

#[async_trait]
impl Database for FakeDatabase {
    async fn transaction(&self) -> Result<Transaction, RepositoryError> {
        Ok(Transaction::new(FakeDb::Transaction {
            world: self.0.clone(),
            pending: self.0.records(),
        }))
    }

    async fn connection(&self) -> Result<Db, RepositoryError> {
        Ok(Db::new(FakeDb::Connection(self.0.clone())))
    }
}

fn fake(db: &mut Db) -> &mut FakeDb {
    db.downcast_mut::<FakeDb>().unwrap()
}

struct FakePayslips(Arc<World>);

#[async_trait]
impl PayslipRepository for FakePayslips {
    async fn find(&self, id: PayslipId) -> Result<Option<Payslip>, RepositoryError> {
        Ok(self.0.records().payslips.iter().find(|r| r.0 == id).map(to_payslip))
    }

    async fn list_by_staff(&self, staff_id: StaffId) -> Result<Vec<Payslip>, RepositoryError> {
        Ok(self.0.records().payslips.iter().filter(|r| r.1 == staff_id).map(to_payslip).collect())
    }

    async fn insert(&self, db: &mut Db, new: &NewPayslip) -> Result<PayslipId, RepositoryError> {
        Ok(fake(db).write(|r| {
            let id = PayslipId::from_i64(next_id(r.payslips.len())).unwrap();
            r.payslips.push((id, new.staff_id(), new.period(), new.lines().to_vec()));
            id
        }))
    }

    async fn update(&self, _db: &mut Db, _payslip: &Payslip) -> Result<(), RepositoryError> {
        unimplemented!()
    }
}

struct FakeStaff(Arc<World>);

#[async_trait]
impl StaffRepository for FakeStaff {
    async fn find(&self, id: StaffId) -> Result<Option<Staff>, RepositoryError> {
        Ok(self.0.records().staff.iter().find(|r| r.0 == id).map(to_staff))
    }

    async fn find_by_user_id(&self, user_id: &UserId) -> Result<Option<Staff>, RepositoryError> {
        Ok(self.0.records().staff.iter().find(|r| &r.1 == user_id).map(to_staff))
    }

    async fn find_by_email(&self, email: &Email) -> Result<Option<Staff>, RepositoryError> {
        Ok(self.0.records().staff.iter().find(|r| &r.2 == email).map(to_staff))
    }

    async fn insert(&self, db: &mut Db, new: &NewStaff) -> Result<StaffId, RepositoryError> {
        if self.0.fail_staff_insert {
            return Err(RepositoryError::Unavailable("db down".into()));
        }
        Ok(fake(db).write(|r| {
            let id = StaffId::from_i64(next_id(r.staff.len())).unwrap();
            r.staff.push((id, new.user_id().clone(), new.email().clone()));
            id
        }))
    }
}

struct FakeOutbox(Arc<World>);

#[async_trait]
impl EventOutbox for FakeOutbox {
    async fn append(&self, db: &mut Db, event: PayrollEvent) -> Result<(), RepositoryError> {
        if self.0.fail_event_append {
            return Err(RepositoryError::Unavailable("db down".into()));
        }
        fake(db).write(|r| r.events.push(event));
        Ok(())
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

fn finalize_usecase(world: &Arc<World>) -> FinalizePayslipUseCase {
    FinalizePayslipUseCase::new(
        Arc::new(FakePayslips(world.clone())),
        Arc::new(FakeStaff(world.clone())),
        Arc::new(FakeOutbox(world.clone())),
        Arc::new(FakeDatabase(world.clone())),
    )
}

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
    AuthenticatedUser { user_id: UserId::parse(sub).unwrap(), roles: roles.to_vec() }
}

// ---- テスト ----

#[tokio::test]
async fn finalize_records_the_payslip_and_its_event_together() {
    let world = Arc::new(World::with_staff(&[(1, "taro")]));

    let id = finalize_usecase(&world).execute(input(1, 9)).await.unwrap();

    let records = world.records();
    assert_eq!(records.payslips.len(), 1);
    assert!(matches!(
        records.events.as_slice(),
        [PayrollEvent::Payslip { id: event_id, event: PayslipEvent::Finalized { total, .. } }]
            if *event_id == id && total.as_yen() == 12_000
    ));
}

#[tokio::test]
async fn finalize_keeps_nothing_when_the_event_cannot_be_recorded() {
    let world = Arc::new(World { fail_event_append: true, ..World::with_staff(&[(1, "taro")]) });

    let err = finalize_usecase(&world).execute(input(1, 9)).await.unwrap_err();

    assert!(matches!(err, UseCaseError::Unavailable(_)));
    let records = world.records();
    assert!(records.payslips.is_empty());
    assert!(records.events.is_empty());
}

#[tokio::test]
async fn finalize_rejects_unknown_staff() {
    let world = Arc::new(World::default());
    let err = finalize_usecase(&world).execute(input(1, 9)).await.unwrap_err();
    assert!(matches!(err, UseCaseError::InvalidInput(_)));
}

#[tokio::test]
async fn finalize_rejects_same_month_twice() {
    let world = Arc::new(World::with_staff(&[(1, "taro")]));
    let usecase = finalize_usecase(&world);

    usecase.execute(input(1, 9)).await.unwrap();
    let err = usecase.execute(input(1, 9)).await.unwrap_err();
    assert!(matches!(err, UseCaseError::Conflict(_)));
    usecase.execute(input(1, 10)).await.unwrap();
}

#[tokio::test]
async fn only_admin_or_owner_can_view_payslip() {
    let world = Arc::new(World::with_staff(&[(1, "taro"), (2, "hanako")]));
    let id = finalize_usecase(&world).execute(input(1, 9)).await.unwrap();
    let get =
        GetPayslipUseCase::new(Arc::new(FakePayslips(world.clone())), Arc::new(FakeStaff(world)));

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
async fn create_staff_disables_user_when_registration_fails() {
    let world = Arc::new(World { fail_staff_insert: true, ..World::default() });
    let directory = Arc::new(FakeDirectory::default());
    let usecase = CreateStaffUseCase::new(
        Arc::new(FakeStaff(world.clone())),
        Arc::new(FakeDatabase(world)),
        directory.clone(),
    );

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
