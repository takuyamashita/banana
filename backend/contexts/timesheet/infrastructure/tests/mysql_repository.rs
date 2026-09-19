//! DB 結合テスト。testcontainers で使い捨ての MySQL を立て、マイグレーションを流して検証する

// allow-unwrap-in-tests は #[test] 関数の中にしか効かず、補助関数は対象外
#![allow(clippy::unwrap_used)]

use std::sync::Arc;

use platform_gen::acme::timesheet::events::v1 as published;
use platform_kernel::UserId;
use sqlx::MySqlPool;
use testcontainers_modules::mysql::Mysql;
use testcontainers_modules::testcontainers::runners::AsyncRunner;
use testcontainers_modules::testcontainers::{ContainerAsync, ImageExt};
use time::Date;
use time::macros::{date, datetime};
use timesheet_domain::project::{Project, ProjectId, ProjectName};
use timesheet_domain::staff::{Staff, StaffId, StaffName};
use timesheet_domain::timesheet::{
    ReturnReason, Timesheet, TimesheetId, TimesheetStatus, WorkEntry, WorkMinutes, WorkMonth,
};
use timesheet_infrastructure::clock::SystemClock;
use timesheet_infrastructure::database::MySqlDatabase;
use timesheet_infrastructure::messaging::outbox::MySqlEventOutbox;
use timesheet_infrastructure::repository::{
    MySqlProjectRepository, MySqlStaffRepository, MySqlTimesheetRepository,
};
use timesheet_usecase::approval::ApproveTimesheetUseCase;
use timesheet_usecase::ports::database::Database;
use timesheet_usecase::ports::repository::{
    ProjectRepository, RepositoryError, StaffRepository, TimesheetRepository,
};

struct TestDb {
    pool: MySqlPool,
    _container: ContainerAsync<Mysql>,
}

async fn db() -> TestDb {
    // 非同期 I/O を使わない設定で立てる(同時に多数を立てると、カーネルの上限を使い切る)
    let container = Mysql::default()
        .with_tag("8.4")
        .with_cmd(["--innodb-use-native-aio=0"])
        .start()
        .await
        .unwrap();
    let port = container.get_host_port_ipv4(3306).await.unwrap();
    let pool = timesheet_infrastructure::connect(&format!("mysql://root@127.0.0.1:{port}/test"), 5)
        .await
        .unwrap();
    timesheet_infrastructure::MIGRATOR.run(&pool).await.unwrap();
    TestDb { pool, _container: container }
}

/// 派遣社員 3 と案件 1・2 が届いている状態にする
async fn seed(db: &TestDb) {
    let mut conn = MySqlDatabase::new(db.pool.clone()).connection().await.unwrap();
    MySqlStaffRepository::new(db.pool.clone())
        .save(
            &mut conn,
            &Staff::new(
                StaffId::from_i64(3).unwrap(),
                UserId::parse("sub-3").unwrap(),
                StaffName::new("派遣 太郎").unwrap(),
            ),
        )
        .await
        .unwrap();
    let projects = MySqlProjectRepository::new(db.pool.clone());
    for id in [1, 2] {
        let project = Project::new(
            ProjectId::from_i64(id).unwrap(),
            ProjectName::new(format!("案件{id}")).unwrap(),
        );
        projects.save(&mut conn, &project).await.unwrap();
    }
}

fn entry(date: Date, project: i64, minutes: u32) -> WorkEntry {
    WorkEntry::new(
        date,
        ProjectId::from_i64(project).unwrap(),
        WorkMinutes::from_minutes(minutes).unwrap(),
    )
}

fn september() -> WorkMonth {
    WorkMonth::new(2026, 9).unwrap()
}

fn staff() -> StaffId {
    StaffId::from_i64(3).unwrap()
}

async fn insert_draft(
    db: &TestDb,
    entries: Vec<WorkEntry>,
) -> Result<TimesheetId, RepositoryError> {
    let new = Timesheet::start(staff(), september()).record(entries).unwrap();
    let mut conn = MySqlDatabase::new(db.pool.clone()).connection().await.unwrap();
    MySqlTimesheetRepository::new(db.pool.clone()).insert(&mut conn, &new).await
}

/// 読んだ勤務表を比べやすい形にする(状態・稼働の行・差し戻しの理由)
fn snapshot(t: &Timesheet) -> (TimesheetStatus, Vec<(Date, i64, u32)>, Option<String>) {
    let entries = t
        .content()
        .entries()
        .iter()
        .map(|e| (e.date(), e.project_id().as_i64(), e.minutes().as_minutes()))
        .collect();
    let reason = match t {
        Timesheet::Draft(d) => d.returned_reason().map(|r| r.as_str().to_owned()),
        _ => None,
    };
    (t.status(), entries, reason)
}

async fn update(db: &TestDb, timesheet: &Timesheet) -> Result<(), RepositoryError> {
    let mut conn = MySqlDatabase::new(db.pool.clone()).connection().await.unwrap();
    MySqlTimesheetRepository::new(db.pool.clone()).update(&mut conn, timesheet).await
}

#[tokio::test]
async fn a_timesheet_is_restored_as_written_in_every_state() {
    let db = db().await;
    seed(&db).await;
    let repo = MySqlTimesheetRepository::new(db.pool.clone());
    let id = insert_draft(
        &db,
        vec![entry(date!(2026 - 09 - 02), 2, 450), entry(date!(2026 - 09 - 01), 1, 480)],
    )
    .await
    .unwrap();

    let Some(Timesheet::Draft(draft)) = repo.find(id).await.unwrap() else {
        panic!("作成中のはず")
    };
    assert_eq!(
        snapshot(&draft.clone().into()),
        (
            TimesheetStatus::Draft,
            vec![(date!(2026 - 09 - 01), 1, 480), (date!(2026 - 09 - 02), 2, 450)],
            None
        )
    );

    // 申告 → 差し戻し → 書き直し → 申告 → 承認。どの状態でも、書いたものがそのまま読み戻る
    let submitted = draft.submit(datetime!(2026-09-30 09:00 UTC)).unwrap();
    update(&db, &submitted.clone().into()).await.unwrap();
    assert_eq!(repo.find(id).await.unwrap().unwrap().status(), TimesheetStatus::Submitted);

    let returned = submitted.send_back(ReturnReason::new("9/2 の案件が違います").unwrap());
    update(&db, &returned.clone().into()).await.unwrap();
    let fixed = returned.record(vec![entry(date!(2026 - 09 - 02), 1, 450)]).unwrap();
    update(&db, &fixed.clone().into()).await.unwrap();
    assert_eq!(
        snapshot(&repo.find(id).await.unwrap().unwrap()),
        (
            TimesheetStatus::Draft,
            vec![(date!(2026 - 09 - 02), 1, 450)],
            Some("9/2 の案件が違います".to_owned())
        )
    );

    let (approved, _) = fixed
        .submit(datetime!(2026-09-30 12:00 UTC))
        .unwrap()
        .approve(datetime!(2026-10-01 10:00 UTC));
    update(&db, &approved.into()).await.unwrap();
    let Some(Timesheet::Approved(found)) = repo.find(id).await.unwrap() else {
        panic!("承認済みのはず")
    };
    assert_eq!(found.approved_at(), datetime!(2026-10-01 10:00 UTC));
    assert_eq!(found.submitted_at(), datetime!(2026-09-30 12:00 UTC));
}

#[tokio::test]
async fn one_timesheet_per_staff_and_month() {
    let db = db().await;
    seed(&db).await;
    insert_draft(&db, vec![]).await.unwrap();

    let err = insert_draft(&db, vec![]).await.unwrap_err();

    assert!(matches!(err, RepositoryError::Conflict(_)));
}

#[tokio::test]
async fn work_on_a_project_not_announced_by_payroll_is_not_recorded() {
    let db = db().await;
    seed(&db).await;

    let err = insert_draft(&db, vec![entry(date!(2026 - 09 - 01), 9, 480)]).await.unwrap_err();

    // 外部キーで止まり、勤務表の行も残らない(勤務表と稼働の行は一緒に書く)
    assert!(matches!(err, RepositoryError::Internal(_)), "{err:?}");
    let count: i64 =
        sqlx::query_scalar("select count(*) from timesheets").fetch_one(&db.pool).await.unwrap();
    assert_eq!(count, 0);
}

#[tokio::test]
async fn database_rejects_a_status_that_contradicts_its_times() {
    let db = db().await;
    seed(&db).await;
    let id = insert_draft(&db, vec![entry(date!(2026 - 09 - 01), 1, 480)]).await.unwrap();

    for statement in [
        "update timesheets set status = 'submitted', submitted_at = null where id = ?",
        "update timesheets set status = 'approved', submitted_at = now(6), approved_at = null where id = ?",
        "update timesheets set status = 'unknown' where id = ?",
    ] {
        let err = sqlx::query(statement).bind(id.as_i64()).execute(&db.pool).await.unwrap_err();
        assert!(err.to_string().contains("ck_timesheets_status"), "{statement}: {err}");
    }
    let err = sqlx::query("update timesheet_entries set work_minutes = 470")
        .execute(&db.pool)
        .await
        .unwrap_err();
    assert!(err.to_string().contains("ck_timesheet_entries_minutes"), "{err}");
}

#[tokio::test]
async fn reading_for_update_needs_a_transaction() {
    let db = db().await;
    seed(&db).await;
    let id = insert_draft(&db, vec![]).await.unwrap();
    let mut conn = MySqlDatabase::new(db.pool.clone()).connection().await.unwrap();

    let err = MySqlTimesheetRepository::new(db.pool.clone())
        .find_for_update(&mut conn, id)
        .await
        .unwrap_err();

    assert!(matches!(err, RepositoryError::Internal(_)));
}

#[tokio::test]
async fn copies_from_payroll_are_replaced_by_what_arrives() {
    let db = db().await;
    seed(&db).await;
    let staff_repo = MySqlStaffRepository::new(db.pool.clone());
    let mut conn = MySqlDatabase::new(db.pool.clone()).connection().await.unwrap();
    let renamed = Staff::new(
        staff(),
        UserId::parse("sub-3").unwrap(),
        StaffName::new("派遣 太郎(改)").unwrap(),
    );

    // 同じ派遣社員が2回届いても1件。後から届いた内容に置き換わる
    staff_repo.save(&mut conn, &renamed).await.unwrap();
    staff_repo.save(&mut conn, &renamed).await.unwrap();

    assert_eq!(
        staff_repo.find_by_user_id(&UserId::parse("sub-3").unwrap()).await.unwrap(),
        Some(renamed)
    );
    let count: i64 =
        sqlx::query_scalar("select count(*) from staff").fetch_one(&db.pool).await.unwrap();
    assert_eq!(count, 1);
    assert_eq!(MySqlProjectRepository::new(db.pool.clone()).list().await.unwrap().len(), 2);
}

#[tokio::test]
async fn approval_is_recorded_with_its_event_in_the_published_shape() {
    let db = db().await;
    seed(&db).await;
    let repo = Arc::new(MySqlTimesheetRepository::new(db.pool.clone()));
    let id = insert_draft(
        &db,
        vec![entry(date!(2026 - 09 - 01), 2, 480), entry(date!(2026 - 09 - 02), 1, 450)],
    )
    .await
    .unwrap();
    let Some(Timesheet::Draft(draft)) = repo.find(id).await.unwrap() else { panic!() };
    update(&db, &draft.submit(datetime!(2026-09-30 09:00 UTC)).unwrap().into()).await.unwrap();

    ApproveTimesheetUseCase::new(
        repo.clone(),
        Arc::new(MySqlStaffRepository::new(db.pool.clone())),
        Arc::new(MySqlEventOutbox),
        Arc::new(MySqlDatabase::new(db.pool.clone())),
        Arc::new(SystemClock),
    )
    .execute(id)
    .await
    .unwrap();

    let (group, event_type, payload): (String, String, serde_json::Value) = sqlx::query_as(
        "select concat(aggregate_type, '-', aggregate_id), event_type, payload from outbox",
    )
    .fetch_one(&db.pool)
    .await
    .unwrap();
    assert_eq!(
        (group, event_type),
        (format!("timesheet-{}", id.as_i64()), "timesheet.approved".to_owned())
    );
    // 受け手(給与)は proto の型で読む。int64 は文字列、案件は番号の順
    let approved: published::TimesheetApproved = serde_json::from_value(payload.clone()).unwrap();
    assert_eq!((approved.staff_id, approved.year, approved.month), (3, 2026, 9));
    let work: Vec<_> = approved.work.iter().map(|w| (w.project_id, w.work_minutes)).collect();
    assert_eq!(work, [(1, 450), (2, 480)]);
    assert_eq!(payload["staffId"], "3");
}
