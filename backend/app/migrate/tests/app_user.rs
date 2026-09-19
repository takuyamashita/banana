//! アプリ用の DB ユーザーの権限を、使い捨ての MySQL で確かめる

// allow-unwrap-in-tests は #[test] 関数の中にしか効かず、補助関数は対象外
#![allow(clippy::unwrap_used)]

use migrate::{Credentials, ensure_app_user};
use sqlx::mysql::MySqlPool;
use testcontainers_modules::mysql::Mysql;
use testcontainers_modules::testcontainers::runners::AsyncRunner;
use testcontainers_modules::testcontainers::{ContainerAsync, ImageExt};

async fn mysql() -> (ContainerAsync<Mysql>, u16, MySqlPool) {
    let container = Mysql::default()
        .with_tag("8.4")
        .with_cmd(["--innodb-use-native-aio=0"])
        .start()
        .await
        .unwrap();
    let port = container.get_host_port_ipv4(3306).await.unwrap();
    // testcontainers の MySQL は root・パスワードなし・DB "test"
    let admin = MySqlPool::connect(&format!("mysql://root@127.0.0.1:{port}/test")).await.unwrap();
    payroll_infrastructure::MIGRATOR.run(&admin).await.unwrap();
    (container, port, admin)
}

fn app(password: &str) -> Credentials {
    Credentials { username: "app".into(), password: password.into() }
}

async fn connect_as(port: u16, password: &str) -> Result<MySqlPool, sqlx::Error> {
    MySqlPool::connect(&format!("mysql://app:{password}@127.0.0.1:{port}/test")).await
}

#[tokio::test]
async fn the_app_user_can_read_and_write_but_cannot_change_tables() {
    let (_container, port, admin) = mysql().await;

    ensure_app_user(&admin, "test", &app("AppPassword0123456789")).await.unwrap();

    let pool = connect_as(port, "AppPassword0123456789").await.unwrap();
    sqlx::query("insert into projects (name) values ('案件A')").execute(&pool).await.unwrap();
    let count: i64 =
        sqlx::query_scalar("select count(*) from projects").fetch_one(&pool).await.unwrap();
    assert_eq!(count, 1);

    for ddl in
        ["drop table projects", "create table t (id int)", "alter table projects add column x int"]
    {
        let err = sqlx::query(ddl).execute(&pool).await.unwrap_err();
        assert!(err.to_string().contains("command denied"), "{ddl}: {err}");
    }
}

#[tokio::test]
async fn running_again_with_a_new_password_changes_it() {
    let (_container, port, admin) = mysql().await;
    ensure_app_user(&admin, "test", &app("OldPassword0123456789")).await.unwrap();

    ensure_app_user(&admin, "test", &app("NewPassword0123456789")).await.unwrap();

    assert!(connect_as(port, "OldPassword0123456789").await.is_err());
    connect_as(port, "NewPassword0123456789").await.unwrap();
}

#[tokio::test]
async fn names_that_would_change_the_statement_are_rejected() {
    let (_container, _port, admin) = mysql().await;

    let injected = Credentials {
        username: "app'@'%'; drop table projects; --".into(),
        password: "x".repeat(20),
    };
    assert!(ensure_app_user(&admin, "test", &injected).await.is_err());
    assert!(ensure_app_user(&admin, "test", &app("short")).await.is_err());
    assert!(
        ensure_app_user(&admin, "test`; drop database test; --", &app("AppPassword0123456789"))
            .await
            .is_err()
    );
    // 途中まで流れて表が消えていないこと(エラーになるだけでは確かめたことにならない)
    let tables: i64 = sqlx::query_scalar(
        "select count(*) from information_schema.tables where table_schema = 'test' and table_name = 'projects'",
    )
    .fetch_one(&admin)
    .await
    .unwrap();
    assert_eq!(tables, 1);
}
