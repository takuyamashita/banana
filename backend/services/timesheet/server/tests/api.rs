//! API テスト。本番と同じ組み立てでプロセス内に server を立て、生成された tonic クライアントで直接呼ぶ。
//! 認証だけはテスト用ミドルウェアに差し替え、ヘッダの値から AuthenticatedUser を載せる。
//! 給与からの出来事は、本番と同じ受け手(timesheet_bootstrap::build_event_handler)に封筒の JSON を渡して届ける

// allow-unwrap-in-tests は #[test] 関数の中にしか効かず、補助関数は対象外
#![allow(clippy::unwrap_used)]

use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use platform_gen::acme::timesheet::v1 as proto;
use platform_gen::acme::timesheet::v1::timesheet_service_client::TimesheetServiceClient;
use platform_kernel::{AuthenticatedUser, Role, UserId};
use platform_messaging::consumer::Handler as _;
use sqlx::MySqlPool;
use testcontainers_modules::mysql::Mysql;
use testcontainers_modules::testcontainers::runners::AsyncRunner;
use testcontainers_modules::testcontainers::{ContainerAsync, ImageExt};
use tonic::transport::Channel;
use tonic::{Code, Request as GrpcRequest};

/// x-test-sub / x-test-roles ヘッダから利用者を載せる
async fn test_auth(mut request: Request, next: Next) -> Response {
    let header = |request: &Request, name: &str| {
        request.headers().get(name).and_then(|v| v.to_str().ok()).map(str::to_owned)
    };
    let sub = header(&request, "x-test-sub");
    let roles = header(&request, "x-test-roles").unwrap_or_default();
    if let Some(sub) = sub {
        let roles = roles.split(',').filter_map(Role::from_name).collect();
        request
            .extensions_mut()
            .insert(AuthenticatedUser { user_id: UserId::parse(sub).unwrap(), roles });
    }
    next.run(request).await
}

struct Api {
    channel: Channel,
    pool: MySqlPool,
    _container: ContainerAsync<Mysql>,
}

async fn api() -> Api {
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

    let router =
        tonic::service::Routes::new(proto::timesheet_service_server::TimesheetServiceServer::new(
            timesheet_bootstrap::build_handler(&pool),
        ))
        .into_axum_router()
        .layer(axum::middleware::from_fn(test_auth));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });

    let channel = Channel::from_shared(format!("http://{addr}")).unwrap().connect().await.unwrap();
    Api { channel, pool, _container: container }
}

/// 給与から出来事が届いたことにする(relay が送るのと同じ封筒の JSON)
async fn deliver(api: &Api, event_type: &str, payload: serde_json::Value) {
    let body = serde_json::json!({
        "event_id": 1,
        "event_type": event_type,
        "aggregate_type": "x",
        "aggregate_id": 1,
        "payload": payload,
    });
    timesheet_bootstrap::build_event_handler(&api.pool).handle(&body.to_string()).await.unwrap();
}

/// 派遣社員 3(利用者 sub-taro)と案件 1 が給与から届いた状態にする
async fn announced(api: &Api) {
    deliver(
        api,
        "staff.registered",
        serde_json::json!({ "staffId": "3", "userId": "sub-taro", "displayName": "派遣 太郎" }),
    )
    .await;
    deliver(api, "project.created", serde_json::json!({ "projectId": "1", "name": "案件A" })).await;
}

fn as_user<T>(message: T, sub: &str, roles: &str) -> GrpcRequest<T> {
    let mut request = GrpcRequest::new(message);
    request.metadata_mut().insert("x-test-sub", sub.parse().unwrap());
    request.metadata_mut().insert("x-test-roles", roles.parse().unwrap());
    request
}

fn taro<T>(message: T) -> GrpcRequest<T> {
    as_user(message, "sub-taro", "staff")
}

fn admin<T>(message: T) -> GrpcRequest<T> {
    as_user(message, "admin", "admin")
}

fn september_entries() -> proto::SaveMyTimesheetRequest {
    proto::SaveMyTimesheetRequest {
        year: 2026,
        month: 9,
        entries: vec![
            proto::WorkEntryInput { date: "2026-09-02".into(), project_id: 1, work_minutes: 450 },
            proto::WorkEntryInput { date: "2026-09-01".into(), project_id: 1, work_minutes: 480 },
        ],
    }
}

#[tokio::test]
async fn staff_write_and_submit_and_admin_approves() {
    let api = api().await;
    announced(&api).await;
    let mut client = TimesheetServiceClient::new(api.channel.clone());
    let september = || proto::GetMyTimesheetRequest { year: 2026, month: 9 };

    // まだ書き始めていない月は、稼働のない作成中として見える
    let empty =
        client.get_my_timesheet(taro(september())).await.unwrap().into_inner().timesheet.unwrap();
    assert_eq!(
        (empty.timesheet_id, empty.status(), empty.staff_name.as_str()),
        (0, proto::TimesheetStatus::Draft, "派遣 太郎")
    );

    let saved = client
        .save_my_timesheet(taro(september_entries()))
        .await
        .unwrap()
        .into_inner()
        .timesheet
        .unwrap();
    assert_eq!(saved.total_minutes, 930);
    let dates: Vec<_> =
        saved.entries.iter().map(|e| (e.date.as_str(), e.project_name.as_str())).collect();
    assert_eq!(dates, [("2026-09-01", "案件A"), ("2026-09-02", "案件A")]);

    let submitted = client
        .submit_my_timesheet(taro(proto::SubmitMyTimesheetRequest { year: 2026, month: 9 }))
        .await
        .unwrap()
        .into_inner()
        .timesheet
        .unwrap();
    assert_eq!(submitted.status(), proto::TimesheetStatus::Submitted);

    let list = client
        .list_submitted_timesheets(admin(proto::ListSubmittedTimesheetsRequest::default()))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(list.timesheets.len(), 1);
    let approved = client
        .approve_timesheet(admin(proto::ApproveTimesheetRequest {
            timesheet_id: submitted.timesheet_id,
        }))
        .await
        .unwrap()
        .into_inner()
        .timesheet
        .unwrap();
    assert_eq!(approved.status(), proto::TimesheetStatus::Approved);

    // 承認は給与に知らせる出来事と一緒に記録される
    let (event_type, payload): (String, serde_json::Value) =
        sqlx::query_as("select event_type, payload from outbox")
            .fetch_one(&api.pool)
            .await
            .unwrap();
    assert_eq!(event_type, "timesheet.approved");
    assert_eq!(payload["work"], serde_json::json!([{ "projectId": "1", "workMinutes": 930 }]));
}

#[tokio::test]
async fn only_admins_approve_or_return_and_only_logged_in_users_write() {
    let api = api().await;
    announced(&api).await;
    let mut client = TimesheetServiceClient::new(api.channel.clone());
    client.save_my_timesheet(taro(september_entries())).await.unwrap();
    let id = client
        .submit_my_timesheet(taro(proto::SubmitMyTimesheetRequest { year: 2026, month: 9 }))
        .await
        .unwrap()
        .into_inner()
        .timesheet
        .unwrap()
        .timesheet_id;

    let unauthenticated = client.save_my_timesheet(september_entries()).await.unwrap_err();
    assert_eq!(unauthenticated.code(), Code::Unauthenticated);
    let by_staff = client
        .approve_timesheet(taro(proto::ApproveTimesheetRequest { timesheet_id: id }))
        .await
        .unwrap_err();
    assert_eq!(by_staff.code(), Code::PermissionDenied);
    let list = client
        .list_submitted_timesheets(taro(proto::ListSubmittedTimesheetsRequest::default()))
        .await
        .unwrap_err();
    assert_eq!(list.code(), Code::PermissionDenied);

    // 差し戻すと作成中に戻り、本人に理由が見える
    client
        .return_timesheet(admin(proto::ReturnTimesheetRequest {
            timesheet_id: id,
            reason: "9/2 を確かめてください".into(),
        }))
        .await
        .unwrap();
    let mine = client
        .get_my_timesheet(taro(proto::GetMyTimesheetRequest { year: 2026, month: 9 }))
        .await
        .unwrap()
        .into_inner()
        .timesheet
        .unwrap();
    assert_eq!(
        (mine.status(), mine.returned_reason.as_str()),
        (proto::TimesheetStatus::Draft, "9/2 を確かめてください")
    );
}

#[tokio::test]
async fn inputs_that_break_the_rules_are_invalid_arguments() {
    let api = api().await;
    announced(&api).await;
    let mut client = TimesheetServiceClient::new(api.channel.clone());
    let with = |date: &str, project_id: i64, work_minutes: u32| proto::SaveMyTimesheetRequest {
        year: 2026,
        month: 9,
        entries: vec![proto::WorkEntryInput { date: date.into(), project_id, work_minutes }],
    };

    for (request, why) in [
        (with("2026-09-01", 1, 470), "15分単位でない"),
        (with("2026-10-01", 1, 480), "対象月の外の日"),
        (with("2026/09/01", 1, 480), "日付の形が違う"),
        (with("2026-09-01", 9, 480), "届いていない案件"),
        (proto::SaveMyTimesheetRequest { year: 2026, month: 257, entries: vec![] }, "範囲外の月"),
    ] {
        let err = client.save_my_timesheet(taro(request)).await.unwrap_err();
        assert_eq!(err.code(), Code::InvalidArgument, "{why}: {err:?}");
    }
}

#[tokio::test]
async fn staff_not_yet_announced_are_asked_to_wait() {
    let api = api().await;
    let mut client = TimesheetServiceClient::new(api.channel.clone());

    let err = client
        .get_my_timesheet(taro(proto::GetMyTimesheetRequest { year: 2026, month: 9 }))
        .await
        .unwrap_err();

    assert_eq!(err.code(), Code::FailedPrecondition);
    assert!(err.message().contains("まだ勤怠に届いていません"), "{}", err.message());

    // 給与から届けば、開き直したときに見える
    announced(&api).await;
    client
        .get_my_timesheet(taro(proto::GetMyTimesheetRequest { year: 2026, month: 9 }))
        .await
        .unwrap();
}

#[tokio::test]
async fn unknown_or_broken_events_from_payroll() {
    let api = api().await;
    let handler = timesheet_bootstrap::build_event_handler(&api.pool);

    // 知らない種類は読み飛ばす(給与が出来事を増やしても止まらない)
    let unknown = serde_json::json!({ "event_id": 1, "event_type": "payslip.finalized", "aggregate_type": "payslip", "aggregate_id": 1, "payload": {} });
    handler.handle(&unknown.to_string()).await.unwrap();
    // 読めない出来事は失敗にする(キューが再配信し、何度も失敗すれば DLQ に移る)
    let broken = serde_json::json!({ "event_id": 2, "event_type": "staff.registered", "aggregate_type": "staff", "aggregate_id": 1, "payload": { "staffId": "0" } });
    assert!(handler.handle(&broken.to_string()).await.is_err());
}
