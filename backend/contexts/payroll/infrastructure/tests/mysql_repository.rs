//! DB 結合テスト。testcontainers で使い捨ての MySQL を立て、マイグレーションを流して検証する

// allow-unwrap-in-tests は #[test] 関数の中にしか効かず、補助関数は対象外
#![allow(clippy::unwrap_used)]

use payroll_domain::payslip::{
    NewPayslip, PayPeriod, PayslipId, PayslipLine, PayslipStatus, WorkMinutes,
};
use payroll_domain::project::{NewProject, ProjectId, ProjectName};
use payroll_domain::staff::{DisplayName, NewStaff, StaffId};
use payroll_infrastructure::database::MySqlDatabase;
use payroll_infrastructure::messaging::outbox::MySqlEventOutbox;
use payroll_infrastructure::repository::{
    MySqlPayslipRepository, MySqlProjectRepository, MySqlStaffRepository,
};
use payroll_usecase::ports::database::Database;
use payroll_usecase::ports::events::{EventOutbox, PayrollEvent};
use payroll_usecase::ports::repository::{
    PayslipRepository, ProjectRepository, RepositoryError, StaffRepository,
};
use platform_kernel::{Email, Money, UserId};
use sqlx::MySqlPool;
use testcontainers_modules::mysql::Mysql;
use testcontainers_modules::testcontainers::runners::AsyncRunner;
use testcontainers_modules::testcontainers::{ContainerAsync, ImageExt};

struct TestDb {
    pool: MySqlPool,
    _container: ContainerAsync<Mysql>,
}

async fn db() -> TestDb {
    let container = Mysql::default().with_tag("8.4").start().await.unwrap();
    let port = container.get_host_port_ipv4(3306).await.unwrap();
    // testcontainers の MySQL は root・パスワードなし・DB "test"
    let url = format!("mysql://root@127.0.0.1:{port}/test");
    let pool = payroll_infrastructure::connect(&url, 5).await.unwrap();
    payroll_infrastructure::MIGRATOR.run(&pool).await.unwrap();
    TestDb { pool, _container: container }
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
    let project = MySqlProjectRepository
        .insert(&mut conn, &NewProject::new(ProjectName::new("案件A").unwrap()))
        .await
        .unwrap();
    (staff, project)
}

fn draft(staff: StaffId, project: ProjectId, month: u8) -> NewPayslip {
    let lines = vec![
        PayslipLine::new(
            project,
            WorkMinutes::from_minutes(600).unwrap(),
            Money::from_yen(1_500).unwrap(),
        ),
        PayslipLine::new(
            project,
            WorkMinutes::from_minutes(45).unwrap(),
            Money::from_yen(1_001).unwrap(),
        ),
    ];
    NewPayslip::draft(staff, PayPeriod::new(2026, month).unwrap(), lines).unwrap()
}

/// 給与確定のユースケースと同じく、給与明細とその出来事を1つのトランザクションで記録する
async fn finalize(db: &TestDb, mut payslip: NewPayslip) -> Result<PayslipId, RepositoryError> {
    let finalized = payslip.finalize().unwrap();
    let mut tx = MySqlDatabase::new(db.pool.clone()).transaction().await?;
    let id = MySqlPayslipRepository::new(db.pool.clone()).insert(&mut tx, &payslip).await?;
    MySqlEventOutbox.append(&mut tx, PayrollEvent::Payslip { id, event: finalized }).await?;
    tx.commit().await?;
    Ok(id)
}

async fn outbox_count(pool: &MySqlPool) -> i64 {
    let (count,): (i64,) =
        sqlx::query_as("select count(*) from outbox").fetch_one(pool).await.unwrap();
    count
}

#[tokio::test]
async fn payslip_and_its_event_are_committed_together() {
    let db = db().await;
    let (staff, project) = seed(&db).await;
    let repo = MySqlPayslipRepository::new(db.pool.clone());

    let id = finalize(&db, draft(staff, project, 9)).await.unwrap();

    let found = repo.find(id).await.unwrap().unwrap();
    assert_eq!(found.status(), PayslipStatus::Finalized);
    assert_eq!(found.lines().len(), 2);
    // 600分×1500/60 = 15,000 と 45分×1001/60 = 750.75 → 750
    assert_eq!(found.total().as_yen(), 15_750);

    let (count, event_type, aggregate_id): (i64, String, i64) = sqlx::query_as(
        "select count(*), max(event_type), max(aggregate_id) from outbox where published_at is null",
    )
    .fetch_one(&db.pool)
    .await
    .unwrap();
    assert_eq!((count, event_type.as_str(), aggregate_id), (1, "payslip.finalized", id.as_i64()));
}

#[tokio::test]
async fn records_are_discarded_when_the_transaction_is_not_committed() {
    let db = db().await;
    let (staff, project) = seed(&db).await;
    let repo = MySqlPayslipRepository::new(db.pool.clone());

    let mut payslip = draft(staff, project, 9);
    let finalized = payslip.finalize().unwrap();
    {
        let mut tx = MySqlDatabase::new(db.pool.clone()).transaction().await.unwrap();
        // 給与明細の insert は内側で SAVEPOINT を張って確定する。外側を捨てればそれも消える
        let id = repo.insert(&mut tx, &payslip).await.unwrap();
        MySqlEventOutbox
            .append(&mut tx, PayrollEvent::Payslip { id, event: finalized })
            .await
            .unwrap();
        // commit せずに捨てる
    }

    assert!(repo.list_by_staff(staff).await.unwrap().is_empty());
    assert_eq!(outbox_count(&db.pool).await, 0);
}

#[tokio::test]
async fn payslip_written_outside_a_transaction_is_kept_whole() {
    let db = db().await;
    let (staff, project) = seed(&db).await;
    let repo = MySqlPayslipRepository::new(db.pool.clone());

    let mut conn = MySqlDatabase::new(db.pool.clone()).connection().await.unwrap();
    let id = repo.insert(&mut conn, &draft(staff, project, 9)).await.unwrap();
    drop(conn);

    // 接続に書いた給与明細は、明細行まで一緒に確定している
    let found = repo.find(id).await.unwrap().unwrap();
    assert_eq!(found.lines().len(), 2);
}

#[tokio::test]
async fn second_active_payslip_for_same_month_violates_unique_key() {
    let db = db().await;
    let (staff, project) = seed(&db).await;
    let repo = MySqlPayslipRepository::new(db.pool.clone());

    finalize(&db, draft(staff, project, 9)).await.unwrap();
    let err = finalize(&db, draft(staff, project, 9)).await.unwrap_err();
    assert!(matches!(err, RepositoryError::Conflict(_)));
    // 失敗したトランザクションの出来事は残らない
    assert_eq!(outbox_count(&db.pool).await, 1);

    // 別の月は入る
    finalize(&db, draft(staff, project, 10)).await.unwrap();
    assert_eq!(repo.list_by_staff(staff).await.unwrap().len(), 2);
}

#[tokio::test]
async fn corrupted_row_is_reported_not_panicked() {
    let db = db().await;
    let (staff, project) = seed(&db).await;
    let repo = MySqlPayslipRepository::new(db.pool.clone());
    let id = finalize(&db, draft(staff, project, 9)).await.unwrap();

    sqlx::query("update payslips set status = 'paid' where id = ?")
        .bind(id.as_i64())
        .execute(&db.pool)
        .await
        .unwrap();

    assert!(matches!(repo.find(id).await, Err(RepositoryError::CorruptedData(_))));
}
