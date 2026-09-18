//! DB 結合テスト。testcontainers で使い捨ての MySQL を立て、マイグレーションを流して検証する

// allow-unwrap-in-tests は #[test] 関数の中にしか効かず、補助関数は対象外
#![allow(clippy::unwrap_used)]

use payroll_domain::payslip::{NewPayslip, PayPeriod, PayslipLine, PayslipStatus, WorkMinutes};
use payroll_domain::project::{NewProject, ProjectName};
use payroll_domain::staff::{DisplayName, Email, NewStaff, StaffId, UserId};
use payroll_infrastructure::repository::{
    MySqlPayslipRepository, MySqlProjectRepository, MySqlStaffRepository,
};
use payroll_usecase::ports::repository::{
    PayslipRepository, ProjectRepository, RepositoryError, StaffRepository,
};
use platform_kernel::Money;
use sqlx::MySqlPool;
use testcontainers_modules::mysql::Mysql;
use testcontainers_modules::testcontainers::runners::AsyncRunner;
use testcontainers_modules::testcontainers::{ContainerAsync, ImageExt};

struct Db {
    pool: MySqlPool,
    _container: ContainerAsync<Mysql>,
}

async fn db() -> Db {
    let container = Mysql::default().with_tag("8.4").start().await.unwrap();
    let port = container.get_host_port_ipv4(3306).await.unwrap();
    // testcontainers の MySQL は root・パスワードなし・DB "test"
    let url = format!("mysql://root@127.0.0.1:{port}/test");
    let pool = payroll_infrastructure::connect(&url, 5).await.unwrap();
    payroll_infrastructure::MIGRATOR.run(&pool).await.unwrap();
    Db { pool, _container: container }
}

async fn seed(pool: &MySqlPool) -> (StaffId, payroll_domain::project::ProjectId) {
    let staff = MySqlStaffRepository::new(pool.clone())
        .insert(&NewStaff::new(
            UserId::parse("sub-1").unwrap(),
            Email::parse("taro@example.com").unwrap(),
            DisplayName::new("派遣 太郎").unwrap(),
        ))
        .await
        .unwrap();
    let project = MySqlProjectRepository::new(pool.clone())
        .insert(&NewProject::new(ProjectName::new("案件A").unwrap()))
        .await
        .unwrap();
    (staff, project)
}

fn finalized(staff: StaffId, project: payroll_domain::project::ProjectId, month: u8) -> NewPayslip {
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
    let mut p = NewPayslip::draft(staff, PayPeriod::new(2026, month).unwrap(), lines).unwrap();
    p.finalize().unwrap();
    p
}

#[tokio::test]
async fn insert_round_trips_aggregate_and_writes_outbox_in_same_tx() {
    let db = db().await;
    let (staff, project) = seed(&db.pool).await;
    let repo = MySqlPayslipRepository::new(db.pool.clone());

    let saved = repo.insert(&mut finalized(staff, project, 9)).await.unwrap();

    let found = repo.find(saved.id()).await.unwrap().unwrap();
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
    assert_eq!(
        (count, event_type.as_str(), aggregate_id),
        (1, "payslip.finalized", saved.id().as_i64())
    );
}

#[tokio::test]
async fn second_active_payslip_for_same_month_violates_unique_key() {
    let db = db().await;
    let (staff, project) = seed(&db.pool).await;
    let repo = MySqlPayslipRepository::new(db.pool.clone());

    repo.insert(&mut finalized(staff, project, 9)).await.unwrap();
    let err = repo.insert(&mut finalized(staff, project, 9)).await.unwrap_err();
    assert!(matches!(err, RepositoryError::Conflict(_)));

    // 失敗したトランザクションの outbox は残らない
    let (count,): (i64,) =
        sqlx::query_as("select count(*) from outbox").fetch_one(&db.pool).await.unwrap();
    assert_eq!(count, 1);

    // 別の月は入る
    repo.insert(&mut finalized(staff, project, 10)).await.unwrap();
    assert_eq!(repo.list_by_staff(staff).await.unwrap().len(), 2);
}

#[tokio::test]
async fn corrupted_row_is_reported_not_panicked() {
    let db = db().await;
    let (staff, project) = seed(&db.pool).await;
    let repo = MySqlPayslipRepository::new(db.pool.clone());
    let saved = repo.insert(&mut finalized(staff, project, 9)).await.unwrap();

    sqlx::query("update payslips set status = 'paid' where id = ?")
        .bind(saved.id().as_i64())
        .execute(&db.pool)
        .await
        .unwrap();

    assert!(matches!(repo.find(saved.id()).await, Err(RepositoryError::CorruptedData(_))));
}
