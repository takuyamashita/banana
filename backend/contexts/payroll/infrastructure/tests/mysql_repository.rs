//! DB 結合テスト。testcontainers で使い捨ての MySQL を立て、マイグレーションを流して検証する

// allow-unwrap-in-tests は #[test] 関数の中にしか効かず、補助関数は対象外
#![allow(clippy::unwrap_used)]

use std::sync::Arc;

use payroll_domain::payout::{Payout, PayoutOutcome};
use payroll_domain::payslip::{
    HourlyRate, NewPayslip, PayPeriod, Payslip, PayslipId, PayslipLine, PayslipStatus, WorkMinutes,
};
use payroll_domain::project::{NewProject, Project, ProjectId, ProjectName};
use payroll_domain::staff::{DisplayName, NewStaff, Staff, StaffId};
use payroll_infrastructure::database::MySqlDatabase;
use payroll_infrastructure::messaging::outbox::MySqlEventOutbox;
use payroll_infrastructure::repository::{
    MySqlPayoutRepository, MySqlPayslipRepository, MySqlProjectRepository, MySqlStaffRepository,
};
use payroll_usecase::UseCaseError;
use payroll_usecase::payslip::FinalizePayslipUseCase;
use payroll_usecase::ports::clock::Clock;
use payroll_usecase::ports::database::Database;
use payroll_usecase::ports::events::{EventOutbox, PayrollEvent};
use payroll_usecase::ports::repository::{
    PayoutRepository, PayslipRepository, ProjectRepository, RepositoryError, StaffRepository,
};
use platform_kernel::{Email, Money, UserId};
use platform_messaging::publisher::{Outgoing, PublishError, Publisher};
use platform_messaging::relay::{RelayLock, relay_once};
use sqlx::MySqlPool;
use testcontainers_modules::mysql::Mysql;
use testcontainers_modules::testcontainers::runners::AsyncRunner;
use testcontainers_modules::testcontainers::{ContainerAsync, ImageExt};
use time::OffsetDateTime;
use time::macros::datetime;
use tokio::sync::Barrier;
use tokio_util::sync::CancellationToken;

struct TestDb {
    pool: MySqlPool,
    url: String,
    _container: ContainerAsync<Mysql>,
}

async fn db() -> TestDb {
    // テストごとに MySQL を立てるので、同時に多数が起動するとカーネルの非同期 I/O の上限
    // (fs.aio-max-nr)を使い切って起動に失敗する。非同期 I/O を使わない設定で立てる
    let container = Mysql::default()
        .with_tag("8.4")
        .with_cmd(["--innodb-use-native-aio=0"])
        .start()
        .await
        .unwrap();
    let port = container.get_host_port_ipv4(3306).await.unwrap();
    // testcontainers の MySQL は root・パスワードなし・DB "test"
    let url = format!("mysql://root@127.0.0.1:{port}/test");
    let pool = payroll_infrastructure::connect(&url, 5).await.unwrap();
    payroll_infrastructure::MIGRATOR.run(&pool).await.unwrap();
    TestDb { pool, url, _container: container }
}

async fn seed(db: &TestDb) -> (StaffId, ProjectId) {
    let mut conn = MySqlDatabase::new(db.pool.clone()).connection().await.unwrap();
    let staff = MySqlStaffRepository::new(db.pool.clone())
        .insert(
            &mut conn,
            &NewStaff::new(
                UserId::parse("sub-1").unwrap(),
                Email::parse("taro@example.com").unwrap(),
                DisplayName::new("派遣 太郎").unwrap(),
            ),
        )
        .await
        .unwrap();
    let project = MySqlProjectRepository::new(db.pool.clone())
        .insert(&mut conn, &NewProject::new(ProjectName::new("案件A").unwrap()))
        .await
        .unwrap();
    (staff, project)
}

fn line(project: ProjectId, minutes: u32, rate: i64) -> PayslipLine {
    PayslipLine::new(
        project,
        ProjectName::new("案件A").unwrap(),
        WorkMinutes::from_minutes(minutes).unwrap(),
        HourlyRate::from_yen(rate).unwrap(),
    )
    .unwrap()
}

fn draft(staff: StaffId, project: ProjectId, month: u8) -> NewPayslip {
    let lines = vec![line(project, 600, 1_500), line(project, 45, 1_001)];
    Payslip::draft(staff, PayPeriod::new(2026, month).unwrap(), lines).unwrap()
}

/// 確定した日時。DB の datetime(6) はマイクロ秒まで持つ
const FINALIZED_AT: OffsetDateTime = datetime!(2026-09-30 10:00:00.123456 UTC);

struct FixedClock;

impl Clock for FixedClock {
    fn now(&self) -> OffsetDateTime {
        FINALIZED_AT
    }
}

/// 給与明細を作成中として登録する(給与明細の作成と同じく、接続に書く)
async fn create(db: &TestDb, draft: &NewPayslip) -> Result<PayslipId, RepositoryError> {
    let mut conn = MySqlDatabase::new(db.pool.clone()).connection().await?;
    MySqlPayslipRepository::new(db.pool.clone()).insert(&mut conn, draft).await
}

/// 本物の部品で組み立てた給与確定のユースケース
fn finalize_usecase(db: &TestDb) -> FinalizePayslipUseCase {
    FinalizePayslipUseCase::new(
        Arc::new(MySqlPayslipRepository::new(db.pool.clone())),
        Arc::new(MySqlEventOutbox),
        Arc::new(MySqlDatabase::new(db.pool.clone())),
        Arc::new(FixedClock),
    )
}

async fn create_and_finalize(
    db: &TestDb,
    staff: StaffId,
    project: ProjectId,
    month: u8,
) -> PayslipId {
    let id = create(db, &draft(staff, project, month)).await.unwrap();
    finalize_usecase(db).execute(id).await.unwrap();
    id
}

async fn outbox_count(pool: &MySqlPool) -> i64 {
    let (count,): (i64,) =
        sqlx::query_as("select count(*) from outbox").fetch_one(pool).await.unwrap();
    count
}

#[tokio::test]
async fn finalized_payslip_and_its_event_are_committed_together() {
    let db = db().await;
    let (staff, project) = seed(&db).await;
    let repo = MySqlPayslipRepository::new(db.pool.clone());

    let id = create_and_finalize(&db, staff, project, 9).await;

    let found = repo.find(id).await.unwrap().unwrap();
    let Payslip::Finalized(finalized) = &found else { panic!("確定済みのはず: {found:?}") };
    assert_eq!(finalized.finalized_at(), FINALIZED_AT);
    assert_eq!(found.content().lines().len(), 2);
    assert_eq!(found.content().lines()[0].project_name().as_str(), "案件A");
    // 600分×1500/60 = 15,000 と 45分×1001/60 = 750.75 → 750
    assert_eq!(found.content().total().as_yen(), 15_750);

    let (count, event_type, aggregate_id, payload): (i64, String, i64, String) = sqlx::query_as(
        "select count(*), max(event_type), max(aggregate_id), cast(max(payload) as char)
         from outbox where published_at is null",
    )
    .fetch_one(&db.pool)
    .await
    .unwrap();
    assert_eq!((count, event_type.as_str(), aggregate_id), (1, "payslip.finalized", id.as_i64()));
    // 受け手が給与明細と確定日時を知れるように、ペイロードにも載せる
    assert!(payload.contains(&format!("\"payslip_id\": {}", id.as_i64())), "{payload}");
    assert!(payload.contains("2026-09-30T10:00:00.123456Z"), "{payload}");
}

#[tokio::test]
async fn finalization_is_kept_as_draft_when_the_event_cannot_be_recorded() {
    let db = db().await;
    let (staff, project) = seed(&db).await;
    let id = create(&db, &draft(staff, project, 9)).await.unwrap();
    sqlx::query("rename table outbox to outbox_unavailable").execute(&db.pool).await.unwrap();

    let err = finalize_usecase(&db).execute(id).await.unwrap_err();

    assert!(matches!(err, UseCaseError::Internal(_)), "{err:?}");
    let found = MySqlPayslipRepository::new(db.pool.clone()).find(id).await.unwrap().unwrap();
    assert_eq!(found.status(), PayslipStatus::Draft);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_finalization_succeeds_once_and_records_one_event() {
    let db = db().await;
    let (staff, project) = seed(&db).await;
    let id = create(&db, &draft(staff, project, 9)).await.unwrap();
    let usecase = Arc::new(finalize_usecase(&db));
    // 接続プールの上限(5)より少ない数を、そろえて同時に確定する
    let barrier = Arc::new(Barrier::new(4));

    let tasks: Vec<_> = (0..4)
        .map(|_| {
            let (usecase, barrier) = (usecase.clone(), barrier.clone());
            tokio::spawn(async move {
                barrier.wait().await;
                usecase.execute(id).await
            })
        })
        .collect();
    let mut results = Vec::new();
    for task in tasks {
        results.push(task.await.unwrap());
    }

    assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 1, "{results:?}");
    assert!(
        results
            .iter()
            .filter(|r| r.is_err())
            .all(|r| matches!(r, Err(UseCaseError::FailedPrecondition(_)))),
        "{results:?}"
    );
    assert_eq!(outbox_count(&db.pool).await, 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn reading_for_update_waits_until_the_other_transaction_ends() {
    let db = db().await;
    let (staff, project) = seed(&db).await;
    let id = create(&db, &draft(staff, project, 9)).await.unwrap();
    let database = Arc::new(MySqlDatabase::new(db.pool.clone()));
    let repo = Arc::new(MySqlPayslipRepository::new(db.pool.clone()));

    let mut first = database.transaction().await.unwrap();
    repo.find_for_update(&mut first, id).await.unwrap().unwrap();

    // もう一方は、先に読んだトランザクションが終わるまで読めない
    let second = tokio::spawn({
        let (database, repo) = (database.clone(), repo.clone());
        async move {
            let mut tx = database.transaction().await.unwrap();
            repo.find_for_update(&mut tx, id).await.unwrap().unwrap().status()
        }
    });
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    assert!(!second.is_finished(), "ロックされていれば、まだ読めていないはず");

    first.commit().await.unwrap();
    assert_eq!(second.await.unwrap(), PayslipStatus::Draft);
}

#[tokio::test]
async fn reading_for_update_needs_a_transaction() {
    let db = db().await;
    let (staff, project) = seed(&db).await;
    let repo = MySqlPayslipRepository::new(db.pool.clone());
    let id = create(&db, &draft(staff, project, 9)).await.unwrap();

    // 接続ではロックが文の終わりで外れるので、張り忘れとして断る
    let mut conn = MySqlDatabase::new(db.pool.clone()).connection().await.unwrap();
    let err = repo.find_for_update(&mut conn, id).await.unwrap_err();
    assert!(matches!(err, RepositoryError::Internal(_)));
}

#[tokio::test]
async fn records_are_discarded_when_the_transaction_is_not_committed() {
    let db = db().await;
    let (staff, project) = seed(&db).await;
    // 接続を1本にして、捨てたトランザクションと同じ接続で読み直す。
    // 取り消されずに開いたままなら、自分が書いた行が見えてしまう
    let pool = payroll_infrastructure::connect(&db.url, 1).await.unwrap();
    let repo = MySqlPayslipRepository::new(pool.clone());
    {
        let mut tx = MySqlDatabase::new(pool.clone()).transaction().await.unwrap();
        // 給与明細の insert は内側で SAVEPOINT を張って確定する。外側を捨てればそれも消える
        let id = repo.insert(&mut tx, &draft(staff, project, 9)).await.unwrap();
        let Some(Payslip::Draft(draft)) = repo.find_for_update(&mut tx, id).await.unwrap() else {
            panic!("作成中のはず");
        };
        let (finalized, event) = draft.finalize(FINALIZED_AT);
        repo.update(&mut tx, &finalized.into()).await.unwrap();
        MySqlEventOutbox.append(&mut tx, PayrollEvent::Payslip(event)).await.unwrap();
        // commit せずに捨てる
    }

    assert!(repo.list_by_staff(staff).await.unwrap().is_empty());
    assert_eq!(outbox_count(&pool).await, 0);
}

#[tokio::test]
async fn payslip_written_outside_a_transaction_is_kept_whole() {
    let db = db().await;
    let (staff, project) = seed(&db).await;
    let repo = MySqlPayslipRepository::new(db.pool.clone());

    let id = create(&db, &draft(staff, project, 9)).await.unwrap();

    // 接続に書いた給与明細は、明細行まで一緒に確定している
    let found = repo.find(id).await.unwrap().unwrap();
    assert_eq!(found.status(), PayslipStatus::Draft);
    assert_eq!(found.content().lines().len(), 2);
}

#[tokio::test]
async fn second_payslip_for_the_same_month_violates_unique_key() {
    let db = db().await;
    let (staff, project) = seed(&db).await;
    let repo = MySqlPayslipRepository::new(db.pool.clone());

    create(&db, &draft(staff, project, 9)).await.unwrap();
    let err = create(&db, &draft(staff, project, 9)).await.unwrap_err();
    assert!(matches!(err, RepositoryError::Conflict(_)));

    // 別の月は入る
    create(&db, &draft(staff, project, 10)).await.unwrap();
    assert_eq!(repo.list_by_staff(staff).await.unwrap().len(), 2);
}

#[tokio::test]
async fn database_rejects_status_and_finalized_at_that_do_not_match() {
    let db = db().await;
    let (staff, project) = seed(&db).await;
    let id = create(&db, &draft(staff, project, 9)).await.unwrap();

    for sql in [
        "update payslips set status = 'paid' where id = ?",
        "update payslips set finalized_at = now(6) where id = ?",
        "update payslips set status = 'finalized' where id = ?",
    ] {
        let result = sqlx::query(sql).bind(id.as_i64()).execute(&db.pool).await;
        assert!(result.is_err(), "CHECK 制約で拒否されるはず: {sql}");
    }
}

#[tokio::test]
async fn corrupted_row_is_reported_not_panicked() {
    let db = db().await;
    let (staff, project) = seed(&db).await;
    let repo = MySqlPayslipRepository::new(db.pool.clone());
    let id = create_and_finalize(&db, staff, project, 9).await;
    // DB の制約をすり抜けた記録(制約を張る前のデータ・手作業など)も、組み立て直しで弾く
    sqlx::query(
        "alter table payslips alter check ck_payslip_status not enforced,
                              alter check ck_payslip_finalized_at not enforced",
    )
    .execute(&db.pool)
    .await
    .unwrap();
    let corrupt = |sql: &'static str| {
        let pool = db.pool.clone();
        async move { sqlx::query(sql).bind(id.as_i64()).execute(&pool).await.unwrap() }
    };

    corrupt("update payslips set status = 'paid' where id = ?").await;
    assert!(matches!(repo.find(id).await, Err(RepositoryError::CorruptedData(_))));

    // 確定済みなのに確定日時がない行も壊れたデータ
    corrupt("update payslips set status = 'finalized', finalized_at = null where id = ?").await;
    assert!(matches!(repo.find(id).await, Err(RepositoryError::CorruptedData(_))));

    // 15分単位でない稼働時間も、丸めずに壊れたデータとして報告する
    corrupt("update payslips set finalized_at = now(6) where id = ?").await;
    corrupt("update payslip_lines set work_minutes = 100 where payslip_id = ?").await;
    assert!(matches!(repo.find(id).await, Err(RepositoryError::CorruptedData(_))));
}

#[tokio::test]
async fn registered_project_can_be_found() {
    let db = db().await;
    let (_, project) = seed(&db).await;
    let repo = MySqlProjectRepository::new(db.pool.clone());

    let found = repo.find(project).await.unwrap().unwrap();
    assert_eq!(found.name().as_str(), "案件A");
    assert!(repo.find(ProjectId::from_i64(999).unwrap()).await.unwrap().is_none());
}

// ---- 記録と組み立て直し ----
// リポジトリの責務は「集約の今の状態をそのまま記録し、記録からそのまま組み立て直す」こと。
// どの状態・どの形の集約でも、書いたものが読み戻ることを確かめる

/// 給与明細の中身を比べられる形にする(状態・派遣社員・対象月・明細行・支給額・確定日時)
fn snapshot<Id>(
    p: &Payslip<Id>,
) -> (PayslipStatus, StaffId, PayPeriod, Vec<PayslipLine>, Money, Option<OffsetDateTime>) {
    let c = p.content();
    let finalized_at = match p {
        Payslip::Draft(_) => None,
        Payslip::Finalized(f) => Some(f.finalized_at()),
    };
    (p.status(), c.staff_id(), c.period(), c.lines().to_vec(), c.total(), finalized_at)
}

/// 登録済みの作成中の給与明細を、指定した内容で組み立てる(update に渡す形を自由に作るため)
#[allow(clippy::disallowed_methods, reason = "記録の読み書きを確かめるテスト")]
fn draft_with(id: PayslipId, staff: StaffId, month: u8, lines: Vec<PayslipLine>) -> Payslip {
    Payslip::reconstruct_draft(id, staff, PayPeriod::new(2026, month).unwrap(), lines).unwrap()
}

async fn line_rows(pool: &MySqlPool, id: PayslipId) -> i64 {
    sqlx::query_scalar("select count(*) from payslip_lines where payslip_id = ?")
        .bind(id.as_i64())
        .fetch_one(pool)
        .await
        .unwrap()
}

/// 1件だけの書き込み(接続に書く)
async fn update_payslip(db: &TestDb, payslip: &Payslip) -> Result<(), RepositoryError> {
    let mut conn = MySqlDatabase::new(db.pool.clone()).connection().await?;
    MySqlPayslipRepository::new(db.pool.clone()).update(&mut conn, payslip).await
}

#[tokio::test]
async fn inserted_payslip_is_restored_as_it_was() {
    let db = db().await;
    let (staff, project) = seed(&db).await;
    let repo = MySqlPayslipRepository::new(db.pool.clone());
    let new = draft(staff, project, 9);

    let id = create(&db, &new).await.unwrap();

    let found = repo.find(id).await.unwrap().unwrap();
    assert_eq!(snapshot(&found), snapshot(&Payslip::from(new)));
}

#[tokio::test]
async fn updated_payslip_is_restored_as_it_was_in_every_state() {
    let db = db().await;
    let (staff, project) = seed(&db).await;
    let repo = MySqlPayslipRepository::new(db.pool.clone());
    let id = create(&db, &draft(staff, project, 9)).await.unwrap();
    let Some(Payslip::Draft(draft)) = repo.find(id).await.unwrap() else {
        panic!("作成中のはず")
    };

    // 作成中のまま書き戻す
    let as_draft = Payslip::from(draft);
    update_payslip(&db, &as_draft).await.unwrap();
    assert_eq!(snapshot(&repo.find(id).await.unwrap().unwrap()), snapshot(&as_draft));

    // 確定して書き戻す
    let Payslip::Draft(draft) = as_draft else { unreachable!() };
    let (finalized, _event) = draft.finalize(FINALIZED_AT);
    let finalized = Payslip::from(finalized);
    update_payslip(&db, &finalized).await.unwrap();
    assert_eq!(snapshot(&repo.find(id).await.unwrap().unwrap()), snapshot(&finalized));
}

#[tokio::test]
async fn updating_with_the_same_content_again_changes_nothing() {
    let db = db().await;
    let (staff, project) = seed(&db).await;
    let repo = MySqlPayslipRepository::new(db.pool.clone());
    let id = create(&db, &draft(staff, project, 9)).await.unwrap();
    let before = repo.find(id).await.unwrap().unwrap();

    // 値の変わらない更新も、記録がある限り成功する(何度書いても同じ状態になる)
    update_payslip(&db, &before).await.unwrap();
    update_payslip(&db, &before).await.unwrap();

    assert_eq!(snapshot(&repo.find(id).await.unwrap().unwrap()), snapshot(&before));
    assert_eq!(line_rows(&db.pool, id).await, 2, "明細行が重ならない");
}

#[tokio::test]
async fn update_replaces_the_lines_with_the_current_ones() {
    let db = db().await;
    let (staff, project) = seed(&db).await;
    let repo = MySqlPayslipRepository::new(db.pool.clone());
    let id = create(&db, &draft(staff, project, 9)).await.unwrap();
    let other = PayslipLine::new(
        project,
        ProjectName::new("案件B").unwrap(),
        WorkMinutes::from_minutes(30).unwrap(),
        HourlyRate::from_yen(2_000).unwrap(),
    )
    .unwrap();

    // 2行 → 別の内容の1行
    let changed = draft_with(id, staff, 9, vec![other.clone()]);
    update_payslip(&db, &changed).await.unwrap();

    let found = repo.find(id).await.unwrap().unwrap();
    assert_eq!(found.content().lines(), [other]);
    assert_eq!(found.content().total().as_yen(), 1_000);
    assert_eq!(line_rows(&db.pool, id).await, 1);
}

#[tokio::test]
async fn updating_an_unrecorded_payslip_is_an_internal_error() {
    let db = db().await;
    let (staff, project) = seed(&db).await;

    let unrecorded =
        draft_with(PayslipId::from_i64(999).unwrap(), staff, 9, vec![line(project, 60, 1_000)]);
    let err = update_payslip(&db, &unrecorded).await.unwrap_err();

    assert!(matches!(err, RepositoryError::Internal(_)));
    assert_eq!(line_rows(&db.pool, PayslipId::from_i64(999).unwrap()).await, 0);
}

#[tokio::test]
async fn failed_update_leaves_the_payslip_untouched() {
    let db = db().await;
    let (staff, project) = seed(&db).await;
    let repo = MySqlPayslipRepository::new(db.pool.clone());
    create(&db, &draft(staff, project, 9)).await.unwrap();
    let id = create(&db, &draft(staff, project, 10)).await.unwrap();
    let before = repo.find(id).await.unwrap().unwrap();

    // 10月分を、既にある9月分と同じ月にしようとする
    let clash = draft_with(id, staff, 9, vec![line(project, 60, 1_000)]);
    let err = update_payslip(&db, &clash).await.unwrap_err();

    assert!(matches!(err, RepositoryError::Conflict(_)));
    assert_eq!(snapshot(&repo.find(id).await.unwrap().unwrap()), snapshot(&before));
    assert_eq!(line_rows(&db.pool, id).await, 2);
}

#[tokio::test]
#[allow(clippy::disallowed_methods, reason = "記録の読み書きを確かめるテスト")]
async fn staff_and_project_are_restored_as_updated() {
    let db = db().await;
    let (staff, project) = seed(&db).await;
    let staff_repo = MySqlStaffRepository::new(db.pool.clone());
    let project_repo = MySqlProjectRepository::new(db.pool.clone());
    let mut conn = MySqlDatabase::new(db.pool.clone()).connection().await.unwrap();

    let renamed = Staff::reconstruct(
        staff,
        UserId::parse("sub-1").unwrap(),
        Email::parse("taro.new@example.com").unwrap(),
        DisplayName::new("派遣 太郎(新)").unwrap(),
    );
    staff_repo.update(&mut conn, &renamed).await.unwrap();
    staff_repo.update(&mut conn, &renamed).await.unwrap();
    let found = staff_repo.find(staff).await.unwrap().unwrap();
    assert_eq!(found.email().as_str(), "taro.new@example.com");
    assert_eq!(found.display_name().as_str(), "派遣 太郎(新)");

    let renamed = Project::reconstruct(project, ProjectName::new("案件A(改)").unwrap());
    project_repo.update(&mut conn, &renamed).await.unwrap();
    assert_eq!(project_repo.find(project).await.unwrap().unwrap().name().as_str(), "案件A(改)");

    // 記録されていないものは更新できない
    let unrecorded =
        Project::reconstruct(ProjectId::from_i64(999).unwrap(), ProjectName::new("x").unwrap());
    let err = project_repo.update(&mut conn, &unrecorded).await.unwrap_err();
    assert!(matches!(err, RepositoryError::Internal(_)));
}

#[tokio::test]
#[allow(clippy::disallowed_methods, reason = "記録の読み書きを確かめるテスト")]
async fn staff_update_that_takes_another_email_is_a_conflict() {
    let db = db().await;
    let (staff, _) = seed(&db).await;
    let repo = MySqlStaffRepository::new(db.pool.clone());
    let mut conn = MySqlDatabase::new(db.pool.clone()).connection().await.unwrap();
    repo.insert(
        &mut conn,
        &NewStaff::new(
            UserId::parse("sub-2").unwrap(),
            Email::parse("hanako@example.com").unwrap(),
            DisplayName::new("派遣 花子").unwrap(),
        ),
    )
    .await
    .unwrap();

    let taken = Staff::reconstruct(
        staff,
        UserId::parse("sub-1").unwrap(),
        Email::parse("hanako@example.com").unwrap(),
        DisplayName::new("派遣 太郎").unwrap(),
    );
    let err = repo.update(&mut conn, &taken).await.unwrap_err();
    assert!(matches!(err, RepositoryError::Conflict(_)));
    assert_eq!(repo.find(staff).await.unwrap().unwrap().email().as_str(), "taro@example.com");
}

#[tokio::test]
#[allow(clippy::disallowed_methods, reason = "記録の読み書きを確かめるテスト")]
async fn payout_is_restored_as_updated() {
    let db = db().await;
    let (staff, project) = seed(&db).await;
    let payslip = create_and_finalize(&db, staff, project, 9).await;
    let repo = MySqlPayoutRepository::new(db.pool.clone());
    let mut conn = MySqlDatabase::new(db.pool.clone()).connection().await.unwrap();
    let amount = Money::from_yen(15_750).unwrap();
    let accepted = PayoutOutcome::Accepted { receipt: "R-1".into() };
    let id = repo.insert(&mut conn, &Payout::new(payslip, staff, amount, accepted)).await.unwrap();

    let rejected = PayoutOutcome::Rejected { reason: "口座不備".into() };
    repo.update(&mut conn, &Payout::reconstruct(id, payslip, staff, amount, rejected.clone()))
        .await
        .unwrap();

    let found = repo.find_by_payslip(payslip).await.unwrap().unwrap();
    assert_eq!((found.id(), found.amount(), found.outcome()), (id, amount, &rejected));
}

/// 送った出来事を覚えておく送り先。`fail` なら送れなかったことにする
#[derive(Default)]
struct FakePublisher {
    fail: bool,
    sent: std::sync::Mutex<Vec<(String, String, String, String)>>,
}

#[async_trait::async_trait]
impl Publisher for FakePublisher {
    async fn publish(&self, message: &Outgoing<'_>) -> Result<(), PublishError> {
        if self.fail {
            return Err(PublishError("送り先につながりません".into()));
        }
        self.sent.lock().unwrap().push((
            message.group.to_owned(),
            message.deduplication_id.to_owned(),
            message.event_type.to_owned(),
            message.body.to_owned(),
        ));
        Ok(())
    }
}

fn failing() -> FakePublisher {
    FakePublisher { fail: true, ..FakePublisher::default() }
}

fn lock() -> RelayLock {
    RelayLock::for_service("payroll")
}

async fn unpublished_count(pool: &MySqlPool) -> i64 {
    let (count,): (i64,) = sqlx::query_as("select count(*) from outbox where published_at is null")
        .fetch_one(pool)
        .await
        .unwrap();
    count
}

#[tokio::test]
async fn relay_sends_only_while_it_holds_the_lock() {
    let db = db().await;
    let (staff, project) = seed(&db).await;
    create_and_finalize(&db, staff, project, 9).await;
    let publisher = failing();

    // 他のインスタンスがロックを持っている間は、送らずに 0 件で終わる
    let mut other = db.pool.acquire().await.unwrap();
    sqlx::query("select get_lock('payroll_outbox_relay', 0)").execute(&mut *other).await.unwrap();
    assert_eq!(
        relay_once(&db.pool, &publisher, &lock(), &CancellationToken::new()).await.unwrap(),
        0
    );
    sqlx::query("select release_lock('payroll_outbox_relay')").execute(&mut *other).await.unwrap();
    drop(other);

    // ロックが空けば送ろうとする。送れなかった出来事は、試行回数とエラーを残して未送信のまま残る
    assert_eq!(
        relay_once(&db.pool, &publisher, &lock(), &CancellationToken::new()).await.unwrap(),
        0
    );
    assert_eq!(unpublished_count(&db.pool).await, 1);
    let (attempts, has_error): (i32, bool) =
        sqlx::query_as("select attempts, last_error is not null from outbox")
            .fetch_one(&db.pool)
            .await
            .unwrap();
    assert_eq!((attempts, has_error), (1, true));

    // 失敗しても、ロックは外れている
    let mut next = db.pool.acquire().await.unwrap();
    let (locked,): (Option<i64>,) = sqlx::query_as("select get_lock('payroll_outbox_relay', 0)")
        .fetch_one(&mut *next)
        .await
        .unwrap();
    assert_eq!(locked, Some(1));
}

#[tokio::test]
async fn payout_is_recorded_once_per_payslip_with_its_outcome() {
    let db = db().await;
    let (staff, project) = seed(&db).await;
    let payslip = create_and_finalize(&db, staff, project, 9).await;
    let other = create_and_finalize(&db, staff, project, 10).await;
    let repo = MySqlPayoutRepository::new(db.pool.clone());
    let mut conn = MySqlDatabase::new(db.pool.clone()).connection().await.unwrap();
    let amount = Money::from_yen(15_750).unwrap();

    let accepted = PayoutOutcome::Accepted { receipt: "R-1".into() };
    repo.insert(&mut conn, &Payout::new(payslip, staff, amount, accepted.clone())).await.unwrap();
    // 断られた理由は列に収まる長さで切って残す
    let rejected = PayoutOutcome::Rejected { reason: "口座不備".repeat(300) };
    repo.insert(&mut conn, &Payout::new(other, staff, amount, rejected)).await.unwrap();

    let found = repo.find_by_payslip(payslip).await.unwrap().unwrap();
    assert_eq!((found.staff_id(), found.amount(), found.outcome()), (staff, amount, &accepted));
    let PayoutOutcome::Rejected { reason } =
        repo.find_by_payslip(other).await.unwrap().unwrap().outcome().clone()
    else {
        panic!("断られた振込依頼のはず");
    };
    assert_eq!(reason.chars().count(), 1000);

    // 1つの給与明細の振込依頼は1つ
    let again =
        Payout::new(payslip, staff, amount, PayoutOutcome::Accepted { receipt: "R-2".into() });
    let err = repo.insert(&mut conn, &again).await.unwrap_err();
    assert!(matches!(err, RepositoryError::Conflict(_)));
}

#[tokio::test]
async fn relay_stops_before_the_next_row_when_asked_to_stop() {
    let db = db().await;
    let (staff, project) = seed(&db).await;
    create_and_finalize(&db, staff, project, 9).await;
    create_and_finalize(&db, staff, project, 10).await;
    let stop = CancellationToken::new();
    stop.cancel();

    // 止められていれば、1件も送ろうとしない(送れなかった回数も増えない)
    assert_eq!(relay_once(&db.pool, &failing(), &lock(), &stop).await.unwrap(), 0);
    let attempts: i64 = sqlx::query_scalar("select cast(sum(attempts) as signed) from outbox")
        .fetch_one(&db.pool)
        .await
        .unwrap();
    assert_eq!((unpublished_count(&db.pool).await, attempts), (2, 0));
}

#[tokio::test]
async fn relay_sends_events_in_order_with_their_group_and_marks_them_published() {
    let db = db().await;
    let (staff, project) = seed(&db).await;
    let first = create_and_finalize(&db, staff, project, 9).await;
    let second = create_and_finalize(&db, staff, project, 10).await;
    let publisher = FakePublisher::default();

    assert_eq!(
        relay_once(&db.pool, &publisher, &lock(), &CancellationToken::new()).await.unwrap(),
        2
    );

    // 記録した順に、集約ごとのグループと outbox の番号(重複を除く鍵)を付けて送る
    let sent = publisher.sent.lock().unwrap().clone();
    let groups: Vec<_> = sent.iter().map(|(group, ..)| group.clone()).collect();
    assert_eq!(
        groups,
        [format!("payslip-{}", first.as_i64()), format!("payslip-{}", second.as_i64())]
    );
    assert!(sent.iter().all(|(_, _, event_type, _)| event_type == "payslip.finalized"));
    let envelope: serde_json::Value = serde_json::from_str(&sent[0].3).unwrap();
    assert_eq!(envelope["event_id"].to_string(), sent[0].1);
    assert_eq!(envelope["payload"]["payslip_id"], first.as_i64());
    // 送れた出来事は送信済みになり、次は送らない
    assert_eq!(unpublished_count(&db.pool).await, 0);
    assert_eq!(
        relay_once(&db.pool, &publisher, &lock(), &CancellationToken::new()).await.unwrap(),
        0
    );
}

#[tokio::test]
async fn staff_and_projects_registered_before_events_existed_are_announced() {
    use platform_gen::acme::payroll::events::v1 as published;

    let container = Mysql::default()
        .with_tag("8.4")
        .with_cmd(["--innodb-use-native-aio=0"])
        .start()
        .await
        .unwrap();
    let port = container.get_host_port_ipv4(3306).await.unwrap();
    let pool = payroll_infrastructure::connect(&format!("mysql://root@127.0.0.1:{port}/test"), 2)
        .await
        .unwrap();

    // 出来事を記録するようになる前(知らせるマイグレーションの手前)の DB に、派遣社員と案件がある
    let migrations = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations");
    let before = std::env::temp_dir().join(format!("payroll-migrations-before-{port}"));
    std::fs::create_dir_all(&before).unwrap();
    for entry in std::fs::read_dir(&migrations).unwrap() {
        let path = entry.unwrap().path();
        if path.file_name().unwrap().to_str().unwrap() < "20260920000011" {
            std::fs::copy(&path, before.join(path.file_name().unwrap())).unwrap();
        }
    }
    sqlx::migrate::Migrator::new(before.as_path()).await.unwrap().run(&pool).await.unwrap();
    sqlx::query("insert into projects (name) values ('案件A')").execute(&pool).await.unwrap();
    sqlx::query(
        "insert into staff (user_id, email, display_name) values ('sub-1', 'taro@example.com', '派遣 太郎')",
    )
    .execute(&pool)
    .await
    .unwrap();

    payroll_infrastructure::MIGRATOR.run(&pool).await.unwrap();

    // 知らせる出来事が outbox に入り、ペイロードは Rust が送るもの(proto の JSON 形)と同じ形
    let rows: Vec<(String, serde_json::Value)> =
        sqlx::query_as("select event_type, payload from outbox order by id")
            .fetch_all(&pool)
            .await
            .unwrap();
    let project = published::ProjectCreated { project_id: 1, name: "案件A".into() };
    let staff = published::StaffRegistered {
        staff_id: 1,
        user_id: "sub-1".into(),
        display_name: "派遣 太郎".into(),
    };
    assert_eq!(
        rows,
        [
            ("project.created".to_owned(), serde_json::to_value(&project).unwrap()),
            ("staff.registered".to_owned(), serde_json::to_value(&staff).unwrap()),
        ]
    );
    assert_eq!(
        serde_json::from_value::<published::StaffRegistered>(rows[1].1.clone()).unwrap(),
        staff
    );
    std::fs::remove_dir_all(before).unwrap();
}

#[tokio::test]
async fn registering_staff_and_projects_records_events_in_the_published_shape() {
    use payroll_usecase::project::CreateProjectUseCase;
    use platform_gen::acme::payroll::events::v1 as published;

    let db = db().await;
    let outbox: Arc<dyn EventOutbox> = Arc::new(MySqlEventOutbox);
    let id = CreateProjectUseCase::new(
        Arc::new(MySqlProjectRepository::new(db.pool.clone())),
        outbox,
        Arc::new(MySqlDatabase::new(db.pool.clone())),
    )
    .execute(ProjectName::new("案件B").unwrap())
    .await
    .unwrap();

    let (aggregate, event_type, payload): (String, String, serde_json::Value) = sqlx::query_as(
        "select concat(aggregate_type, '-', aggregate_id), event_type, payload from outbox",
    )
    .fetch_one(&db.pool)
    .await
    .unwrap();
    assert_eq!(
        (aggregate, event_type),
        (format!("project-{}", id.as_i64()), "project.created".to_owned())
    );
    // int64 は文字列(proto の JSON の決まり)。他の言語の受け手も同じ形で読める
    assert_eq!(
        payload,
        serde_json::json!({ "projectId": id.as_i64().to_string(), "name": "案件B" })
    );
    assert_eq!(
        serde_json::from_value::<published::ProjectCreated>(payload).unwrap().project_id,
        id.as_i64()
    );
}
