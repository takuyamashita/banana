//! usecase の単体テスト。リポジトリと外部依存はインメモリのフェイクで差し替える

// allow-unwrap-in-tests は #[test] 関数の中にしか効かず、tests/ のフェイクや補助関数は対象外
#![allow(clippy::unwrap_used)]

use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use payroll_domain::payout::{NewPayout, Payout, PayoutId, PayoutOutcome};
use payroll_domain::payslip::{
    HourlyRate, NewPayslip, PayPeriod, Payslip, PayslipEvent, PayslipId, PayslipLine, WorkMinutes,
};
use payroll_domain::project::{NewProject, Project, ProjectId, ProjectName};
use payroll_domain::staff::{DisplayName, NewStaff, Staff, StaffId};
use payroll_usecase::UseCaseError;
use payroll_usecase::payslip::{
    CreatePayslipInput, CreatePayslipLine, CreatePayslipUseCase, FinalizePayslipUseCase,
    GetPayslipUseCase, ListPayslipsUseCase, RequestPayoutInput, RequestPayoutResult,
    RequestPayoutUseCase,
};
use payroll_usecase::ports::clock::Clock;
use payroll_usecase::ports::database::{Database, Db, DbHandle, Transaction};
use payroll_usecase::ports::events::{EventOutbox, PayrollEvent};
use payroll_usecase::ports::payout_gateway::{PayoutError, PayoutGateway, PayoutReceipt};
use payroll_usecase::ports::repository::{
    PayoutRepository, PayslipRepository, ProjectRepository, RepositoryError, StaffRepository,
};
use payroll_usecase::ports::user_directory::{UserDirectory, UserDirectoryError};
use payroll_usecase::staff::{CreateStaffInput, CreateStaffUseCase};
use platform_kernel::{AuthenticatedUser, Email, Money, Role, UserId};
use time::OffsetDateTime;
use time::macros::datetime;

// ---- フェイク ----
// フェイクも「リポジトリ実装」なので reconstruct を呼ぶ必要がある。ルートの clippy.toml の禁止は
// tests/ にも効くため、呼ぶ箇所だけ明示的に許可する

/// 番号・派遣社員・対象月・明細行・確定日時(作成中なら None)
#[derive(Clone)]
struct PayslipRow {
    id: PayslipId,
    staff_id: StaffId,
    period: PayPeriod,
    lines: Vec<PayslipLine>,
    /// 確定日時。作成中なら None
    finalized_at: Option<OffsetDateTime>,
}

#[derive(Clone)]
struct StaffRow {
    id: StaffId,
    user_id: UserId,
    email: Email,
}

#[derive(Clone)]
struct PayoutRow {
    payslip_id: PayslipId,
    staff_id: StaffId,
    amount: Money,
    outcome: PayoutOutcome,
}

/// 確定済みの記録。取り出しはここを見て、トランザクションは commit でここへ反映する
#[derive(Default, Clone)]
struct Records {
    payslips: Vec<PayslipRow>,
    staff: Vec<StaffRow>,
    projects: Vec<ProjectId>,
    payouts: Vec<PayoutRow>,
    events: Vec<PayrollEvent>,
}

#[derive(Default)]
struct World {
    committed: Mutex<Records>,
    fail_staff_insert: bool,
    fail_event_append: bool,
}

impl World {
    /// 派遣社員と、案件1件(案件番号 1)が登録済みの状態
    fn with_staff(rows: &[(i64, &str)]) -> Self {
        let staff = rows
            .iter()
            .map(|(id, user)| StaffRow {
                id: StaffId::from_i64(*id).unwrap(),
                user_id: UserId::parse(*user).unwrap(),
                email: Email::parse(format!("{user}@example.com")).unwrap(),
            })
            .collect();
        let projects = vec![ProjectId::from_i64(1).unwrap()];
        Self {
            committed: Mutex::new(Records { staff, projects, ..Records::default() }),
            ..Self::default()
        }
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
    match r.finalized_at {
        Some(at) => Payslip::reconstruct_finalized(r.id, r.staff_id, r.period, r.lines.clone(), at),
        None => Payslip::reconstruct_draft(r.id, r.staff_id, r.period, r.lines.clone()),
    }
    .unwrap()
}

#[allow(clippy::disallowed_methods, reason = "フェイクのリポジトリ実装")]
fn to_staff(r: &StaffRow) -> Staff {
    Staff::reconstruct(r.id, r.user_id.clone(), r.email.clone(), DisplayName::new("x").unwrap())
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
        Ok(self.0.records().payslips.iter().find(|r| r.id == id).map(to_payslip))
    }

    async fn list_by_staff(&self, staff_id: StaffId) -> Result<Vec<Payslip>, RepositoryError> {
        Ok(self
            .0
            .records()
            .payslips
            .iter()
            .filter(|r| r.staff_id == staff_id)
            .map(to_payslip)
            .collect())
    }

    async fn find_for_update(
        &self,
        db: &mut Db,
        id: PayslipId,
    ) -> Result<Option<Payslip>, RepositoryError> {
        // 本物と同じく、ロックして読むにはトランザクションが要る
        let db = fake(db);
        if matches!(db, FakeDb::Connection(_)) {
            return Err(RepositoryError::Internal("トランザクションが必要です".into()));
        }
        Ok(db.write(|r| r.payslips.iter().find(|row| row.id == id).map(to_payslip)))
    }

    async fn insert(&self, db: &mut Db, new: &NewPayslip) -> Result<PayslipId, RepositoryError> {
        Ok(fake(db).write(|r| {
            let id = PayslipId::from_i64(next_id(r.payslips.len())).unwrap();
            let c = new.content();
            r.payslips.push(PayslipRow {
                id,
                staff_id: c.staff_id(),
                period: c.period(),
                lines: c.lines().to_vec(),
                finalized_at: None,
            });
            id
        }))
    }

    async fn update(&self, db: &mut Db, payslip: &Payslip) -> Result<(), RepositoryError> {
        fake(db).write(|r| {
            let c = payslip.content();
            let Some(row) = r.payslips.iter_mut().find(|row| row.id == c.id()) else {
                return Err(RepositoryError::Internal("記録されていない給与明細です".into()));
            };
            row.staff_id = c.staff_id();
            row.period = c.period();
            row.lines = c.lines().to_vec();
            row.finalized_at = match payslip {
                Payslip::Draft(_) => None,
                Payslip::Finalized(finalized) => Some(finalized.finalized_at()),
            };
            Ok(())
        })
    }
}

struct FakeStaff(Arc<World>);

#[async_trait]
impl StaffRepository for FakeStaff {
    async fn find(&self, id: StaffId) -> Result<Option<Staff>, RepositoryError> {
        Ok(self.0.records().staff.iter().find(|r| r.id == id).map(to_staff))
    }

    async fn find_by_user_id(&self, user_id: &UserId) -> Result<Option<Staff>, RepositoryError> {
        Ok(self.0.records().staff.iter().find(|r| &r.user_id == user_id).map(to_staff))
    }

    async fn find_by_email(&self, email: &Email) -> Result<Option<Staff>, RepositoryError> {
        Ok(self.0.records().staff.iter().find(|r| &r.email == email).map(to_staff))
    }

    async fn insert(&self, db: &mut Db, new: &NewStaff) -> Result<StaffId, RepositoryError> {
        if self.0.fail_staff_insert {
            return Err(RepositoryError::Unavailable("db down".into()));
        }
        Ok(fake(db).write(|r| {
            let id = StaffId::from_i64(next_id(r.staff.len())).unwrap();
            r.staff.push(StaffRow {
                id,
                user_id: new.user_id().clone(),
                email: new.email().clone(),
            });
            id
        }))
    }

    async fn update(&self, db: &mut Db, staff: &Staff) -> Result<(), RepositoryError> {
        fake(db).write(|r| {
            let Some(row) = r.staff.iter_mut().find(|row| row.id == staff.id()) else {
                return Err(RepositoryError::Internal("記録されていない派遣社員です".into()));
            };
            row.user_id = staff.user_id().clone();
            row.email = staff.email().clone();
            Ok(())
        })
    }
}

struct FakeProjects(Arc<World>);

#[async_trait]
impl ProjectRepository for FakeProjects {
    #[allow(clippy::disallowed_methods, reason = "フェイクのリポジトリ実装")]
    async fn find(&self, id: ProjectId) -> Result<Option<Project>, RepositoryError> {
        Ok(self
            .0
            .records()
            .projects
            .contains(&id)
            .then(|| Project::reconstruct(id, project_name(id))))
    }

    async fn insert(&self, db: &mut Db, _new: &NewProject) -> Result<ProjectId, RepositoryError> {
        Ok(fake(db).write(|r| {
            let id = ProjectId::from_i64(next_id(r.projects.len())).unwrap();
            r.projects.push(id);
            id
        }))
    }

    async fn update(&self, db: &mut Db, project: &Project) -> Result<(), RepositoryError> {
        fake(db).write(|r| {
            // フェイクは案件名を持たない(名前は番号から決まる)ので、登録されているかだけを見る
            if r.projects.contains(&project.id()) {
                Ok(())
            } else {
                Err(RepositoryError::Internal("記録されていない案件です".into()))
            }
        })
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

struct FakePayouts(Arc<World>);

#[async_trait]
impl PayoutRepository for FakePayouts {
    #[allow(clippy::disallowed_methods, reason = "フェイクのリポジトリ実装")]
    async fn find_by_payslip(
        &self,
        payslip_id: PayslipId,
    ) -> Result<Option<Payout>, RepositoryError> {
        let records = self.0.records();
        Ok(records.payouts.iter().enumerate().find(|(_, r)| r.payslip_id == payslip_id).map(
            |(i, r)| {
                let id = PayoutId::from_i64(next_id(i)).unwrap();
                Payout::reconstruct(id, r.payslip_id, r.staff_id, r.amount, r.outcome.clone())
            },
        ))
    }

    async fn insert(&self, db: &mut Db, new: &NewPayout) -> Result<PayoutId, RepositoryError> {
        fake(db).write(|r| {
            // 本物と同じく、1つの給与明細の振込依頼は1つ
            if r.payouts.iter().any(|p| p.payslip_id == new.payslip_id()) {
                return Err(RepositoryError::Conflict("一意制約に違反しました".into()));
            }
            r.payouts.push(PayoutRow {
                payslip_id: new.payslip_id(),
                staff_id: new.staff_id(),
                amount: new.amount(),
                outcome: new.outcome().clone(),
            });
            Ok(PayoutId::from_i64(next_id(r.payouts.len() - 1)).unwrap())
        })
    }

    async fn update(&self, db: &mut Db, payout: &Payout) -> Result<(), RepositoryError> {
        fake(db).write(|r| {
            let index = usize::try_from(payout.id().as_i64() - 1).unwrap();
            let Some(row) = r.payouts.get_mut(index) else {
                return Err(RepositoryError::Internal("記録されていない振込依頼です".into()));
            };
            row.payslip_id = payout.payslip_id();
            row.staff_id = payout.staff_id();
            row.amount = payout.amount();
            row.outcome = payout.outcome().clone();
            Ok(())
        })
    }
}

/// 決めた答えを返す振込先。受けた依頼の冪等キーを覚えておく
struct FakeGateway {
    answer: fn() -> Result<PayoutReceipt, PayoutError>,
    requested: Mutex<Vec<String>>,
}

impl FakeGateway {
    fn answering(answer: fn() -> Result<PayoutReceipt, PayoutError>) -> Arc<Self> {
        Arc::new(Self { answer, requested: Mutex::default() })
    }
}

#[async_trait]
impl PayoutGateway for FakeGateway {
    async fn request_transfer(
        &self,
        _staff_id: StaffId,
        _amount: Money,
        idempotency_key: &str,
    ) -> Result<PayoutReceipt, PayoutError> {
        self.requested.lock().unwrap().push(idempotency_key.to_owned());
        (self.answer)()
    }
}

/// いつ聞いても同じ日時を返す時計
struct FixedClock;

const NOW: OffsetDateTime = datetime!(2026-09-30 10:00 UTC);

impl Clock for FixedClock {
    fn now(&self) -> OffsetDateTime {
        NOW
    }
}

#[derive(Default)]
struct FakeDirectory {
    deleted: Mutex<Vec<UserId>>,
    created: Mutex<HashMap<String, UserId>>,
    fail_delete: bool,
}

#[async_trait]
impl UserDirectory for FakeDirectory {
    async fn create_user(&self, email: &Email, _pw: &str) -> Result<UserId, UserDirectoryError> {
        let id = UserId::parse(format!("sub-{}", email.as_str())).unwrap();
        self.created.lock().unwrap().insert(email.as_str().to_owned(), id.clone());
        Ok(id)
    }

    async fn delete_user(&self, id: &UserId) -> Result<(), UserDirectoryError> {
        if self.fail_delete {
            return Err(UserDirectoryError::Unavailable("idp down".into()));
        }
        self.deleted.lock().unwrap().push(id.clone());
        Ok(())
    }
}

// ---- ヘルパー ----

fn create_usecase(world: &Arc<World>) -> CreatePayslipUseCase {
    CreatePayslipUseCase::new(
        Arc::new(FakePayslips(world.clone())),
        Arc::new(FakeStaff(world.clone())),
        Arc::new(FakeProjects(world.clone())),
        Arc::new(FakeDatabase(world.clone())),
    )
}

fn finalize_usecase(world: &Arc<World>) -> FinalizePayslipUseCase {
    FinalizePayslipUseCase::new(
        Arc::new(FakePayslips(world.clone())),
        Arc::new(FakeOutbox(world.clone())),
        Arc::new(FakeDatabase(world.clone())),
        Arc::new(FixedClock),
    )
}

fn payout_usecase(world: &Arc<World>, gateway: &Arc<FakeGateway>) -> RequestPayoutUseCase {
    RequestPayoutUseCase::new(
        Arc::new(FakePayouts(world.clone())),
        gateway.clone(),
        Arc::new(FakeDatabase(world.clone())),
    )
}

fn payout_input(payslip: i64, key: &str) -> RequestPayoutInput {
    RequestPayoutInput {
        payslip_id: PayslipId::from_i64(payslip).unwrap(),
        staff_id: StaffId::from_i64(1).unwrap(),
        total: Money::from_yen(12_000).unwrap(),
        idempotency_key: key.to_owned(),
    }
}

/// フェイクの案件の名前
fn project_name(id: ProjectId) -> ProjectName {
    ProjectName::new(format!("案件{}", id.as_i64())).unwrap()
}

fn line_for(project: i64) -> CreatePayslipLine {
    CreatePayslipLine {
        project_id: ProjectId::from_i64(project).unwrap(),
        work_minutes: WorkMinutes::from_minutes(600).unwrap(),
        hourly_rate: HourlyRate::from_yen(1_200).unwrap(),
    }
}

fn line() -> CreatePayslipLine {
    line_for(1)
}

fn input(staff: i64, month: u8) -> CreatePayslipInput {
    CreatePayslipInput {
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
async fn create_makes_a_draft_payslip() {
    let world = Arc::new(World::with_staff(&[(1, "taro")]));

    create_usecase(&world).execute(input(1, 9)).await.unwrap();

    let records = world.records();
    assert_eq!(records.payslips.len(), 1);
    // 作成中なので確定日時はなく、出来事もまだない
    assert_eq!(records.payslips[0].finalized_at, None);
    assert!(records.events.is_empty());
}

#[tokio::test]
async fn create_keeps_the_project_name_at_that_time() {
    let world = Arc::new(World::with_staff(&[(1, "taro")]));

    create_usecase(&world).execute(input(1, 9)).await.unwrap();

    let lines = world.records().payslips[0].lines.clone();
    assert_eq!(lines[0].project_name().as_str(), "案件1");
}

#[tokio::test]
async fn create_rejects_unknown_staff() {
    let world = Arc::new(World::default());
    let err = create_usecase(&world).execute(input(1, 9)).await.unwrap_err();
    assert!(matches!(err, UseCaseError::InvalidInput(_)));
}

#[tokio::test]
async fn create_rejects_unknown_project() {
    let world = Arc::new(World::with_staff(&[(1, "taro")]));
    let err = create_usecase(&world)
        .execute(CreatePayslipInput { lines: vec![line(), line_for(99)], ..input(1, 9) })
        .await
        .unwrap_err();

    assert!(matches!(err, UseCaseError::InvalidInput(_)));
    assert!(world.records().payslips.is_empty());
}

#[tokio::test]
async fn create_rejects_same_month_twice() {
    let world = Arc::new(World::with_staff(&[(1, "taro")]));
    let usecase = create_usecase(&world);

    usecase.execute(input(1, 9)).await.unwrap();
    let err = usecase.execute(input(1, 9)).await.unwrap_err();
    assert!(matches!(err, UseCaseError::Conflict(_)));
    usecase.execute(input(1, 10)).await.unwrap();
}

#[tokio::test]
async fn finalize_records_the_finalized_payslip_and_its_event_together() {
    let world = Arc::new(World::with_staff(&[(1, "taro")]));
    let id = create_usecase(&world).execute(input(1, 9)).await.unwrap();

    finalize_usecase(&world).execute(id).await.unwrap();

    let records = world.records();
    // 確定日時は時計の今
    assert_eq!(records.payslips[0].finalized_at, Some(NOW));
    assert!(matches!(
        records.events.as_slice(),
        [PayrollEvent::Payslip(PayslipEvent::Finalized { payslip_id, total, finalized_at, .. })]
            if *payslip_id == id && total.as_yen() == 12_000 && *finalized_at == NOW
    ));
}

#[tokio::test]
async fn finalize_keeps_the_draft_when_the_event_cannot_be_recorded() {
    let world = Arc::new(World { fail_event_append: true, ..World::with_staff(&[(1, "taro")]) });
    let id = create_usecase(&world).execute(input(1, 9)).await.unwrap();

    let err = finalize_usecase(&world).execute(id).await.unwrap_err();

    assert!(matches!(err, UseCaseError::Unavailable(_)));
    let records = world.records();
    assert_eq!(records.payslips[0].finalized_at, None);
    assert!(records.events.is_empty());
}

#[tokio::test]
async fn finalize_rejects_a_finalized_payslip() {
    let world = Arc::new(World::with_staff(&[(1, "taro")]));
    let id = create_usecase(&world).execute(input(1, 9)).await.unwrap();
    let finalize = finalize_usecase(&world);

    finalize.execute(id).await.unwrap();
    let err = finalize.execute(id).await.unwrap_err();

    assert!(matches!(err, UseCaseError::FailedPrecondition(_)));
    assert_eq!(world.records().events.len(), 1);
}

#[tokio::test]
async fn finalize_rejects_unknown_payslip() {
    let world = Arc::new(World::with_staff(&[(1, "taro")]));
    let err = finalize_usecase(&world).execute(PayslipId::from_i64(99).unwrap()).await.unwrap_err();
    assert!(matches!(err, UseCaseError::NotFound));
}

#[tokio::test]
async fn only_admin_or_owner_can_view_payslip() {
    let world = Arc::new(World::with_staff(&[(1, "taro"), (2, "hanako")]));
    let id = create_usecase(&world).execute(input(1, 9)).await.unwrap();
    finalize_usecase(&world).execute(id).await.unwrap();
    let get = GetPayslipUseCase::new(
        Arc::new(FakePayslips(world.clone())),
        Arc::new(FakeStaff(world.clone())),
    );

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
async fn staff_cannot_see_draft_payslips() {
    let world = Arc::new(World::with_staff(&[(1, "taro")]));
    let finalized = create_usecase(&world).execute(input(1, 9)).await.unwrap();
    finalize_usecase(&world).execute(finalized).await.unwrap();
    let draft = create_usecase(&world).execute(input(1, 10)).await.unwrap();
    let get = GetPayslipUseCase::new(
        Arc::new(FakePayslips(world.clone())),
        Arc::new(FakeStaff(world.clone())),
    );
    let list =
        ListPayslipsUseCase::new(Arc::new(FakePayslips(world.clone())), Arc::new(FakeStaff(world)));
    let (admin, taro) = (user("admin", &[Role::Admin]), user("taro", &[]));
    let staff_id = StaffId::from_i64(1).unwrap();

    // 本人には作成中の給与明細は存在しないものとして扱う
    assert!(matches!(get.execute(&taro, draft).await, Err(UseCaseError::NotFound)));
    assert_eq!(list.execute(&taro, staff_id).await.unwrap().len(), 1);
    // 管理者はどちらも見られる
    assert!(get.execute(&admin, draft).await.is_ok());
    assert_eq!(list.execute(&admin, staff_id).await.unwrap().len(), 2);
}

fn staff_input(email: &str) -> CreateStaffInput {
    CreateStaffInput {
        email: Email::parse(email).unwrap(),
        display_name: DisplayName::new("新人").unwrap(),
        temporary_password: "Temp-pass-1".into(),
    }
}

fn create_staff_usecase(world: &Arc<World>, directory: &Arc<FakeDirectory>) -> CreateStaffUseCase {
    CreateStaffUseCase::new(
        Arc::new(FakeStaff(world.clone())),
        Arc::new(FakeDatabase(world.clone())),
        directory.clone(),
    )
}

#[tokio::test]
async fn create_staff_deletes_the_account_when_registration_fails() {
    let world = Arc::new(World { fail_staff_insert: true, ..World::default() });
    let directory = Arc::new(FakeDirectory::default());

    let err = create_staff_usecase(&world, &directory)
        .execute(staff_input("new@example.com"))
        .await
        .unwrap_err();

    // 発行したアカウントを消して、同じメールアドレスで登録し直せるようにする
    assert!(matches!(err, UseCaseError::Unavailable(_)));
    assert_eq!(
        directory.deleted.lock().unwrap().as_slice(),
        [UserId::parse("sub-new@example.com").unwrap()]
    );
}

#[tokio::test]
async fn create_staff_reports_an_account_left_behind() {
    let world = Arc::new(World { fail_staff_insert: true, ..World::default() });
    let directory = Arc::new(FakeDirectory { fail_delete: true, ..FakeDirectory::default() });

    let err = create_staff_usecase(&world, &directory)
        .execute(staff_input("new@example.com"))
        .await
        .unwrap_err();

    // 消せなかったアカウントは管理者が片付けるので、利用者IDとともに内部の異常として知らせる
    assert!(
        matches!(&err, UseCaseError::Internal(msg) if msg.contains("sub-new@example.com")),
        "{err:?}"
    );
}

#[tokio::test]
async fn create_staff_rejects_the_same_email_without_issuing_an_account() {
    let world = Arc::new(World::with_staff(&[(1, "taro")]));
    let directory = Arc::new(FakeDirectory::default());

    let err = create_staff_usecase(&world, &directory)
        .execute(staff_input("taro@example.com"))
        .await
        .unwrap_err();

    assert!(matches!(err, UseCaseError::Conflict(_)));
    assert!(directory.created.lock().unwrap().is_empty());
}

#[tokio::test]
async fn accepted_payout_is_recorded_with_its_receipt() {
    let world = Arc::new(World::default());
    let gateway = FakeGateway::answering(|| Ok(PayoutReceipt("R-1".into())));

    let result =
        payout_usecase(&world, &gateway).execute(payout_input(1, "event-1")).await.unwrap();

    assert_eq!(result, RequestPayoutResult::Accepted);
    assert_eq!(*gateway.requested.lock().unwrap(), ["event-1"]);
    let payouts = world.records().payouts;
    assert_eq!(payouts.len(), 1);
    assert_eq!(payouts[0].outcome, PayoutOutcome::Accepted { receipt: "R-1".into() });
}

#[tokio::test]
async fn rejected_payout_is_recorded_and_not_treated_as_a_failure() {
    let world = Arc::new(World::default());
    let gateway =
        FakeGateway::answering(|| Err(PayoutError::Rejected("口座が見つかりません".into())));

    let result =
        payout_usecase(&world, &gateway).execute(payout_input(1, "event-1")).await.unwrap();

    // やり直しても通らないので、失敗にはせず、理由とともに記録する
    assert_eq!(result, RequestPayoutResult::Rejected { reason: "口座が見つかりません".into() });
    assert_eq!(
        world.records().payouts[0].outcome,
        PayoutOutcome::Rejected { reason: "口座が見つかりません".into() }
    );
}

#[tokio::test]
async fn unavailable_payout_is_not_recorded_so_it_can_be_retried() {
    let world = Arc::new(World::default());
    let gateway = FakeGateway::answering(|| Err(PayoutError::Unavailable("timeout".into())));

    let err =
        payout_usecase(&world, &gateway).execute(payout_input(1, "event-1")).await.unwrap_err();

    assert!(matches!(err, UseCaseError::Unavailable(_)));
    assert!(world.records().payouts.is_empty());
}

#[tokio::test]
async fn payout_is_requested_only_once_per_payslip() {
    let world = Arc::new(World::default());
    let gateway = FakeGateway::answering(|| Ok(PayoutReceipt("R-1".into())));
    let usecase = payout_usecase(&world, &gateway);

    usecase.execute(payout_input(1, "event-1")).await.unwrap();
    // 同じ給与確定のメッセージがもう一度届いても、振込先には依頼しない
    let again = usecase.execute(payout_input(1, "event-1")).await.unwrap();

    assert_eq!(again, RequestPayoutResult::AlreadyRequested);
    assert_eq!(gateway.requested.lock().unwrap().len(), 1);
    assert_eq!(world.records().payouts.len(), 1);
}
