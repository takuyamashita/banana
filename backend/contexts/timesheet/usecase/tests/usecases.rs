//! usecase の単体テスト。リポジトリと出来事の記録先はインメモリのフェイクで差し替える

// allow-unwrap-in-tests は #[test] 関数の中にしか効かず、tests/ のフェイクや補助関数は対象外
#![allow(clippy::unwrap_used)]

use std::any::Any;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use platform_kernel::{AuthenticatedUser, Role, UserId};
use time::macros::{date, datetime};
use time::{Date, OffsetDateTime};
use timesheet_domain::project::{Project, ProjectId, ProjectName};
use timesheet_domain::staff::{Staff, StaffId, StaffName};
use timesheet_domain::timesheet::{
    NewTimesheet, ProjectWork, ReturnReason, Timesheet, TimesheetEvent, TimesheetId,
    TimesheetStatus, WorkEntry, WorkMinutes, WorkMonth,
};
use timesheet_usecase::UseCaseError;
use timesheet_usecase::approval::{ApproveTimesheetUseCase, ReturnTimesheetUseCase};
use timesheet_usecase::copies::RecordStaffUseCase;
use timesheet_usecase::my_timesheet::{
    GetMyTimesheetUseCase, MyTimesheet, SaveMyTimesheetUseCase, SubmitMyTimesheetUseCase,
};
use timesheet_usecase::ports::clock::Clock;
use timesheet_usecase::ports::database::{Database, Db, DbHandle, Transaction};
use timesheet_usecase::ports::events::EventOutbox;
use timesheet_usecase::ports::repository::{
    ProjectRepository, RepositoryError, StaffRepository, TimesheetRepository,
};

// ---- フェイク ----

/// 確定済みの記録。取り出しはここを見て、トランザクションは commit でここへ反映する
#[derive(Default, Clone)]
struct Records {
    timesheets: Vec<Timesheet>,
    staff: Vec<Staff>,
    projects: Vec<Project>,
    events: Vec<TimesheetEvent>,
}

#[derive(Default)]
struct World {
    committed: Mutex<Records>,
    fail_event_append: bool,
}

impl World {
    /// 派遣社員 3(利用者 taro)と案件 1・2 が届いている状態
    fn ready() -> Arc<Self> {
        let world = Self::default();
        {
            let mut records = world.committed.lock().unwrap();
            records.staff.push(Staff::new(
                StaffId::from_i64(3).unwrap(),
                UserId::parse("taro").unwrap(),
                StaffName::new("派遣 太郎").unwrap(),
            ));
            for id in [1, 2] {
                records.projects.push(Project::new(
                    ProjectId::from_i64(id).unwrap(),
                    ProjectName::new(format!("案件{id}")).unwrap(),
                ));
            }
        }
        Arc::new(world)
    }

    fn records(&self) -> Records {
        self.committed.lock().unwrap().clone()
    }
}

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

fn fake(db: &mut Db) -> &mut FakeDb {
    db.downcast_mut::<FakeDb>().unwrap()
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

struct FakeTimesheets(Arc<World>);

#[async_trait]
impl TimesheetRepository for FakeTimesheets {
    async fn find(&self, id: TimesheetId) -> Result<Option<Timesheet>, RepositoryError> {
        Ok(self.0.records().timesheets.into_iter().find(|t| t.content().id() == id))
    }

    async fn find_by_staff_month(
        &self,
        staff_id: StaffId,
        month: WorkMonth,
    ) -> Result<Option<Timesheet>, RepositoryError> {
        Ok(self
            .0
            .records()
            .timesheets
            .into_iter()
            .find(|t| t.content().staff_id() == staff_id && t.content().month() == month))
    }

    async fn list_submitted(&self) -> Result<Vec<Timesheet>, RepositoryError> {
        Ok(self
            .0
            .records()
            .timesheets
            .into_iter()
            .filter(|t| t.status() == TimesheetStatus::Submitted)
            .collect())
    }

    async fn find_for_update(
        &self,
        db: &mut Db,
        id: TimesheetId,
    ) -> Result<Option<Timesheet>, RepositoryError> {
        Ok(fake(db).write(|r| r.timesheets.iter().find(|t| t.content().id() == id).cloned()))
    }

    #[allow(clippy::disallowed_methods, reason = "フェイクのリポジトリ実装")]
    async fn insert(
        &self,
        db: &mut Db,
        new: &NewTimesheet,
    ) -> Result<TimesheetId, RepositoryError> {
        fake(db).write(|r| {
            let id = TimesheetId::from_i64(i64::try_from(r.timesheets.len()).unwrap() + 1).unwrap();
            let content = new.content();
            r.timesheets.push(
                Timesheet::reconstruct_draft(
                    id,
                    content.staff_id(),
                    content.month(),
                    content.entries().to_vec(),
                    new.returned_reason().cloned(),
                )
                .unwrap(),
            );
            Ok(id)
        })
    }

    async fn update(&self, db: &mut Db, timesheet: &Timesheet) -> Result<(), RepositoryError> {
        fake(db).write(|r| {
            let Some(row) =
                r.timesheets.iter_mut().find(|t| t.content().id() == timesheet.content().id())
            else {
                return Err(RepositoryError::Internal("記録されていない勤務表です".into()));
            };
            *row = timesheet.clone();
            Ok(())
        })
    }
}

struct FakeStaff(Arc<World>);

#[async_trait]
impl StaffRepository for FakeStaff {
    async fn find(&self, id: StaffId) -> Result<Option<Staff>, RepositoryError> {
        Ok(self.0.records().staff.into_iter().find(|s| s.id() == id))
    }

    async fn find_by_user_id(&self, user_id: &UserId) -> Result<Option<Staff>, RepositoryError> {
        Ok(self.0.records().staff.into_iter().find(|s| s.user_id() == user_id))
    }

    async fn save(&self, db: &mut Db, staff: &Staff) -> Result<(), RepositoryError> {
        fake(db).write(|r| {
            r.staff.retain(|s| s.id() != staff.id());
            r.staff.push(staff.clone());
        });
        Ok(())
    }
}

struct FakeProjects(Arc<World>);

#[async_trait]
impl ProjectRepository for FakeProjects {
    async fn find(&self, id: ProjectId) -> Result<Option<Project>, RepositoryError> {
        Ok(self.0.records().projects.into_iter().find(|p| p.id() == id))
    }

    async fn list(&self) -> Result<Vec<Project>, RepositoryError> {
        Ok(self.0.records().projects)
    }

    async fn save(&self, db: &mut Db, project: &Project) -> Result<(), RepositoryError> {
        fake(db).write(|r| {
            r.projects.retain(|p| p.id() != project.id());
            r.projects.push(project.clone());
        });
        Ok(())
    }
}

struct FakeOutbox(Arc<World>);

#[async_trait]
impl EventOutbox for FakeOutbox {
    async fn append(&self, db: &mut Db, event: TimesheetEvent) -> Result<(), RepositoryError> {
        if self.0.fail_event_append {
            return Err(RepositoryError::Unavailable("db down".into()));
        }
        fake(db).write(|r| r.events.push(event));
        Ok(())
    }
}

struct FixedClock(OffsetDateTime);

impl Clock for FixedClock {
    fn now(&self) -> OffsetDateTime {
        self.0
    }
}

// ---- ヘルパー ----

fn taro() -> AuthenticatedUser {
    AuthenticatedUser { user_id: UserId::parse("taro").unwrap(), roles: vec![Role::Staff] }
}

fn september() -> WorkMonth {
    WorkMonth::new(2026, 9).unwrap()
}

fn entry(date: Date, project: i64, minutes: u32) -> WorkEntry {
    WorkEntry::new(
        date,
        ProjectId::from_i64(project).unwrap(),
        WorkMinutes::from_minutes(minutes).unwrap(),
    )
}

fn get(world: &Arc<World>) -> GetMyTimesheetUseCase {
    GetMyTimesheetUseCase::new(
        Arc::new(FakeTimesheets(world.clone())),
        Arc::new(FakeStaff(world.clone())),
    )
}

fn save(world: &Arc<World>) -> SaveMyTimesheetUseCase {
    SaveMyTimesheetUseCase::new(
        Arc::new(FakeTimesheets(world.clone())),
        Arc::new(FakeStaff(world.clone())),
        Arc::new(FakeProjects(world.clone())),
        Arc::new(FakeDatabase(world.clone())),
    )
}

fn submit(world: &Arc<World>) -> SubmitMyTimesheetUseCase {
    SubmitMyTimesheetUseCase::new(
        Arc::new(FakeTimesheets(world.clone())),
        Arc::new(FakeStaff(world.clone())),
        Arc::new(FakeDatabase(world.clone())),
        Arc::new(FixedClock(datetime!(2026-09-30 09:00 UTC))),
    )
}

fn approve(world: &Arc<World>) -> ApproveTimesheetUseCase {
    ApproveTimesheetUseCase::new(
        Arc::new(FakeTimesheets(world.clone())),
        Arc::new(FakeOutbox(world.clone())),
        Arc::new(FakeDatabase(world.clone())),
        Arc::new(FixedClock(datetime!(2026-10-01 10:00 UTC))),
    )
}

fn send_back(world: &Arc<World>) -> ReturnTimesheetUseCase {
    ReturnTimesheetUseCase::new(
        Arc::new(FakeTimesheets(world.clone())),
        Arc::new(FakeDatabase(world.clone())),
    )
}

/// 9月の勤務表を書いて申告し、勤務表番号を返す
async fn submitted(world: &Arc<World>) -> TimesheetId {
    save(world)
        .execute(
            &taro(),
            september(),
            vec![entry(date!(2026 - 09 - 01), 1, 480), entry(date!(2026 - 09 - 02), 2, 450)],
        )
        .await
        .unwrap();
    submit(world).execute(&taro(), september()).await.unwrap().content().id()
}

// ---- テスト ----

#[tokio::test]
async fn a_month_not_started_yet_is_shown_as_an_empty_draft() {
    let world = World::ready();

    let MyTimesheet::NotStarted { staff, month } =
        get(&world).execute(&taro(), september()).await.unwrap()
    else {
        panic!("まだ書き始めていない月のはず");
    };
    assert_eq!((staff.id().as_i64(), month), (3, september()));
}

#[tokio::test]
async fn staff_not_yet_announced_by_payroll_are_told_to_wait() {
    let world = World::ready();
    let hanako =
        AuthenticatedUser { user_id: UserId::parse("hanako").unwrap(), roles: vec![Role::Staff] };

    let err = get(&world).execute(&hanako, september()).await.unwrap_err();

    // 給与で登録された派遣社員が、まだ勤怠に届いていない(しばらくすれば届く)
    assert!(
        matches!(err, UseCaseError::FailedPrecondition(msg) if msg.contains("まだ勤怠に届いていません"))
    );
}

#[tokio::test]
async fn saving_starts_the_timesheet_and_saving_again_rewrites_its_rows() {
    let world = World::ready();

    let first = save(&world)
        .execute(&taro(), september(), vec![entry(date!(2026 - 09 - 01), 1, 480)])
        .await
        .unwrap();
    let second = save(&world)
        .execute(
            &taro(),
            september(),
            vec![entry(date!(2026 - 09 - 02), 2, 60), entry(date!(2026 - 09 - 01), 1, 450)],
        )
        .await
        .unwrap();

    assert_eq!(first.content().id(), second.content().id());
    let records = world.records();
    assert_eq!(records.timesheets.len(), 1);
    assert_eq!(records.timesheets[0].content().total_minutes(), 510);
}

#[tokio::test]
async fn work_can_only_be_recorded_for_projects_announced_by_payroll() {
    let world = World::ready();

    let err = save(&world)
        .execute(&taro(), september(), vec![entry(date!(2026 - 09 - 01), 9, 480)])
        .await
        .unwrap_err();

    assert!(matches!(err, UseCaseError::InvalidInput(msg) if msg.contains("案件 #9")));
    assert!(world.records().timesheets.is_empty());
}

#[tokio::test]
async fn a_submitted_timesheet_cannot_be_rewritten() {
    let world = World::ready();
    submitted(&world).await;

    let err = save(&world)
        .execute(&taro(), september(), vec![entry(date!(2026 - 09 - 03), 1, 60)])
        .await
        .unwrap_err();

    assert!(matches!(err, UseCaseError::FailedPrecondition(_)));
    assert_eq!(world.records().timesheets[0].content().total_minutes(), 930);
}

#[tokio::test]
async fn a_month_without_work_cannot_be_submitted() {
    let world = World::ready();

    let err = submit(&world).execute(&taro(), september()).await.unwrap_err();
    assert!(matches!(err, UseCaseError::FailedPrecondition(_)));

    save(&world).execute(&taro(), september(), vec![]).await.unwrap();
    let err = submit(&world).execute(&taro(), september()).await.unwrap_err();
    assert!(matches!(err, UseCaseError::InvalidInput(_)));
}

#[tokio::test]
async fn approval_records_the_timesheet_and_announces_the_work_together() {
    let world = World::ready();
    let id = submitted(&world).await;

    let approved = approve(&world).execute(id).await.unwrap();

    assert_eq!(approved.status(), TimesheetStatus::Approved);
    let records = world.records();
    assert_eq!(records.timesheets[0].status(), TimesheetStatus::Approved);
    let [TimesheetEvent::Approved { work, .. }] = records.events.as_slice() else {
        panic!("承認したという出来事が1つ記録されるはず");
    };
    assert_eq!(
        work.as_slice(),
        [
            ProjectWork { project_id: ProjectId::from_i64(1).unwrap(), minutes: 480 },
            ProjectWork { project_id: ProjectId::from_i64(2).unwrap(), minutes: 450 },
        ]
    );
}

#[tokio::test]
async fn a_timesheet_is_not_approved_when_the_event_cannot_be_recorded() {
    let world = World::ready();
    let id = submitted(&world).await;
    let failing =
        Arc::new(World { committed: Mutex::new(world.records()), fail_event_append: true });

    let err = approve(&failing).execute(id).await.unwrap_err();

    // 承認だけが記録されて、給与が知らない承認が残ることはない
    assert!(matches!(err, UseCaseError::Unavailable(_)));
    assert_eq!(failing.records().timesheets[0].status(), TimesheetStatus::Submitted);
}

#[tokio::test]
async fn only_submitted_timesheets_can_be_approved_or_returned() {
    let world = World::ready();
    let draft = save(&world)
        .execute(&taro(), september(), vec![entry(date!(2026 - 09 - 01), 1, 480)])
        .await
        .unwrap();
    let id = draft.content().id();

    assert!(matches!(
        approve(&world).execute(id).await.unwrap_err(),
        UseCaseError::FailedPrecondition(_)
    ));
    let reason = ReturnReason::new("直してください").unwrap();
    assert!(matches!(
        send_back(&world).execute(id, reason).await.unwrap_err(),
        UseCaseError::FailedPrecondition(_)
    ));
    assert!(matches!(
        approve(&world).execute(TimesheetId::from_i64(99).unwrap()).await.unwrap_err(),
        UseCaseError::NotFound
    ));
}

#[tokio::test]
async fn a_returned_timesheet_can_be_fixed_and_submitted_again() {
    let world = World::ready();
    let id = submitted(&world).await;

    let returned = send_back(&world)
        .execute(id, ReturnReason::new("9/2 の案件が違います").unwrap())
        .await
        .unwrap();
    let Timesheet::Draft(draft) = &returned else { panic!("作成中に戻るはず") };
    assert_eq!(draft.returned_reason().unwrap().as_str(), "9/2 の案件が違います");

    save(&world)
        .execute(&taro(), september(), vec![entry(date!(2026 - 09 - 02), 1, 450)])
        .await
        .unwrap();
    let resubmitted = submit(&world).execute(&taro(), september()).await.unwrap();
    assert_eq!(resubmitted.status(), TimesheetStatus::Submitted);
    assert!(world.records().events.is_empty());
}

#[tokio::test]
async fn the_same_staff_announced_twice_is_recorded_once() {
    let world = Arc::new(World::default());
    let record = RecordStaffUseCase::new(
        Arc::new(FakeStaff(world.clone())),
        Arc::new(FakeDatabase(world.clone())),
    );
    let staff = Staff::new(
        StaffId::from_i64(5).unwrap(),
        UserId::parse("u5").unwrap(),
        StaffName::new("五郎").unwrap(),
    );

    record.execute(staff.clone()).await.unwrap();
    record.execute(staff.clone()).await.unwrap();

    assert_eq!(world.records().staff, [staff]);
}
