//! API テスト。本番と同じ組み立てでプロセス内に server を立て、生成された tonic クライアントで直接呼ぶ。
//! 認証だけはテスト用ミドルウェアに差し替え、ヘッダの値から AuthenticatedUser を載せる

// allow-unwrap-in-tests は #[test] 関数の中にしか効かず、補助関数は対象外
#![allow(clippy::unwrap_used)]

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use payroll_usecase::ports::user_directory::{UserDirectory, UserDirectoryError};
use platform_gen::acme::payroll::v1 as proto;
use platform_gen::acme::payroll::v1::payroll_service_client::PayrollServiceClient;
use platform_gen::acme::payroll::v1::project_service_client::ProjectServiceClient;
use platform_gen::acme::payroll::v1::staff_service_client::StaffServiceClient;
use platform_gen::acme::payroll::v1::user_service_client::UserServiceClient;
use platform_kernel::{AuthenticatedUser, Email, Role, UserId};
use testcontainers_modules::mysql::Mysql;
use testcontainers_modules::testcontainers::runners::AsyncRunner;
use testcontainers_modules::testcontainers::{ContainerAsync, ImageExt};
use tonic::transport::Channel;
use tonic::{Code, Request as GrpcRequest, Status};

/// 作ったユーザーの sub をメールアドレスから決める認証基盤のフェイク。
/// 何にどのロールを付けたかを覚えておき、テストから確かめられるようにする
#[derive(Default)]
struct FakeDirectory {
    created: Mutex<Vec<(String, Role)>>,
}

#[async_trait]
impl UserDirectory for FakeDirectory {
    async fn create_user(
        &self,
        email: &Email,
        _pw: &str,
        role: Role,
    ) -> Result<UserId, UserDirectoryError> {
        self.created.lock().unwrap().push((email.as_str().to_owned(), role));
        Ok(UserId::parse(format!("sub-{}", email.as_str())).unwrap())
    }

    async fn delete_user(&self, _id: &UserId) -> Result<(), UserDirectoryError> {
        Ok(())
    }
}

/// x-test-sub / x-test-roles ヘッダから利用者を載せる
async fn test_auth(mut request: Request, next: Next) -> Response {
    // &Request を .await の向こうまで持ち越すと Body が Sync でないため future が Send でなくなる。
    // 先に値を取り出しておく
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
    directory: Arc<FakeDirectory>,
    _container: ContainerAsync<Mysql>,
}

/// 本番と同じ組み立て(bootstrap::build_handlers)で server を立てる。
/// 差し替えるのは認証基盤(FakeDirectory)と、トークン検証の代わりのテスト用ミドルウェアだけ
async fn api() -> Api {
    // テストごとに MySQL を立てるので、同時に多数が起動するとカーネルの非同期 I/O の上限
    // (fs.aio-max-nr)を使い切って起動に失敗する。非同期 I/O を使わない設定で立てる
    let container = Mysql::default()
        .with_tag("8.4")
        .with_cmd(["--innodb-use-native-aio=0"])
        .start()
        .await
        .unwrap();
    let port = container.get_host_port_ipv4(3306).await.unwrap();
    let pool = payroll_infrastructure::connect(&format!("mysql://root@127.0.0.1:{port}/test"), 5)
        .await
        .unwrap();
    payroll_infrastructure::MIGRATOR.run(&pool).await.unwrap();

    let directory = Arc::new(FakeDirectory::default());
    let handlers = bootstrap::build_handlers(&pool, directory.clone());
    let router = tonic::service::Routes::new(
        proto::payroll_service_server::PayrollServiceServer::new(handlers.payroll),
    )
    .add_service(proto::staff_service_server::StaffServiceServer::new(handlers.staff))
    .add_service(proto::project_service_server::ProjectServiceServer::new(handlers.project))
    .add_service(proto::user_service_server::UserServiceServer::new(handlers.user))
    .into_axum_router()
    .layer(axum::middleware::from_fn(test_auth));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });

    let channel = Channel::from_shared(format!("http://{addr}")).unwrap().connect().await.unwrap();
    Api { channel, directory, _container: container }
}

fn as_user<T>(message: T, sub: &str, roles: &str) -> GrpcRequest<T> {
    let mut request = GrpcRequest::new(message);
    request.metadata_mut().insert("x-test-sub", sub.parse().unwrap());
    request.metadata_mut().insert("x-test-roles", roles.parse().unwrap());
    request
}

fn as_staff<T>(message: T) -> GrpcRequest<T> {
    as_user(message, "sub-staff@example.com", "staff")
}

fn admin<T>(message: T) -> GrpcRequest<T> {
    as_user(message, "admin", "admin")
}

async fn setup(api: &Api) -> (i64, i64, i64) {
    let mut staff = StaffServiceClient::new(api.channel.clone());
    let mut projects = ProjectServiceClient::new(api.channel.clone());
    let create = |email: &str| proto::CreateStaffRequest {
        email: email.into(),
        display_name: "テスト".into(),
        temporary_password: "Temp-pass-1".into(),
    };
    let taro = staff.create_staff(admin(create("taro@example.com"))).await.unwrap().into_inner();
    let hanako =
        staff.create_staff(admin(create("hanako@example.com"))).await.unwrap().into_inner();
    let project = projects
        .create_project(admin(proto::CreateProjectRequest { name: "案件A".into() }))
        .await
        .unwrap()
        .into_inner();
    (taro.staff_id, hanako.staff_id, project.project_id)
}

fn create_request(staff_id: i64, project_id: i64, month: i32) -> proto::CreatePayslipRequest {
    proto::CreatePayslipRequest {
        staff_id,
        pay_year: 2026,
        pay_month: month,
        lines: vec![proto::PayslipLineInput { project_id, work_minutes: 480, hourly_rate: 1_200 }],
    }
}

/// 管理者として作成して確定し、給与明細番号を返す
async fn create_and_finalize(
    client: &mut PayrollServiceClient<Channel>,
    req: proto::CreatePayslipRequest,
) -> i64 {
    let id = client.create_payslip(admin(req)).await.unwrap().into_inner().payslip_id;
    client.finalize_payslip(admin(proto::FinalizePayslipRequest { payslip_id: id })).await.unwrap();
    id
}

#[tokio::test]
async fn create_and_finalize_require_admin() {
    let api = api().await;
    let (taro, _, project) = setup(&api).await;
    let mut client = PayrollServiceClient::new(api.channel.clone());

    let unauthenticated =
        client.create_payslip(create_request(taro, project, 9)).await.unwrap_err();
    assert_eq!(unauthenticated.code(), Code::Unauthenticated);

    let staff = client
        .create_payslip(as_user(create_request(taro, project, 9), "sub-taro@example.com", "staff"))
        .await
        .unwrap_err();
    assert_eq!(staff.code(), Code::PermissionDenied);

    let id = client
        .create_payslip(admin(create_request(taro, project, 9)))
        .await
        .unwrap()
        .into_inner()
        .payslip_id;
    let finalize = || proto::FinalizePayslipRequest { payslip_id: id };

    let staff = client
        .finalize_payslip(as_user(finalize(), "sub-taro@example.com", "staff"))
        .await
        .unwrap_err();
    assert_eq!(staff.code(), Code::PermissionDenied);

    client.finalize_payslip(admin(finalize())).await.unwrap();
    // 確定済みはもう一度確定できない
    let again = client.finalize_payslip(admin(finalize())).await.unwrap_err();
    assert_eq!(again.code(), Code::FailedPrecondition);
}

#[tokio::test]
async fn out_of_range_month_is_invalid_argument() {
    let api = api().await;
    let (taro, _, project) = setup(&api).await;
    let mut client = PayrollServiceClient::new(api.channel.clone());

    // as u8 なら 257 → 1 に化けて1月分として作られてしまう値
    let err = client.create_payslip(admin(create_request(taro, project, 257))).await.unwrap_err();
    assert_eq!(err.code(), Code::InvalidArgument);
}

#[tokio::test]
async fn payslip_is_visible_only_to_admin_and_owner() {
    let api = api().await;
    let (taro, _, project) = setup(&api).await;
    let mut client = PayrollServiceClient::new(api.channel.clone());
    let id = create_and_finalize(&mut client, create_request(taro, project, 9)).await;
    let get = || proto::GetPayslipRequest { payslip_id: id };

    let own = client.get_payslip(as_user(get(), "sub-taro@example.com", "staff")).await.unwrap();
    assert_eq!(own.into_inner().payslip.unwrap().total_yen, 9_600);

    let other =
        client.get_payslip(as_user(get(), "sub-hanako@example.com", "staff")).await.unwrap_err();
    assert_eq!(other.code(), Code::NotFound);

    let list = client
        .list_payslips(as_user(
            proto::ListPayslipsRequest { staff_id: taro, ..Default::default() },
            "sub-hanako@example.com",
            "staff",
        ))
        .await
        .unwrap_err();
    assert_eq!(list.code(), Code::NotFound);
}

#[tokio::test]
async fn admin_user_is_created_with_the_admin_role() {
    let api = api().await;
    let mut users = UserServiceClient::new(api.channel.clone());

    let created = users
        .create_admin_user(admin(proto::CreateAdminUserRequest {
            email: "new-admin@example.com".into(),
            temporary_password: "Temp-pass-1".into(),
        }))
        .await
        .unwrap()
        .into_inner();

    // 認証基盤のアカウントだけを作る。管理者は派遣社員ではないので、こちらの記録は増えない
    assert_eq!(created.user_id, "sub-new-admin@example.com");
    assert_eq!(
        api.directory.created.lock().unwrap().as_slice(),
        [("new-admin@example.com".to_owned(), Role::Admin)]
    );
}

#[tokio::test]
async fn admin_user_with_a_malformed_email_is_invalid_argument() {
    let api = api().await;
    let mut users = UserServiceClient::new(api.channel.clone());

    let err = users
        .create_admin_user(admin(proto::CreateAdminUserRequest {
            email: "not-an-email".into(),
            temporary_password: "Temp-pass-1".into(),
        }))
        .await
        .unwrap_err();

    // 認証基盤に届く前に断る
    assert_eq!(err.code(), Code::InvalidArgument);
    assert!(api.directory.created.lock().unwrap().is_empty());
}

/// RPC ごとに、誰が呼べるか。RPC を足したらここにも足す(足し忘れると、未認証で呼べる RPC ができうる)
#[tokio::test]
#[allow(clippy::too_many_lines, reason = "全 RPC を1つの表に並べて、抜けを見つけやすくする")]
async fn every_rpc_requires_login_and_admin_only_rpcs_reject_staff() {
    let api = api().await;
    let mut payroll = PayrollServiceClient::new(api.channel.clone());
    let mut staff = StaffServiceClient::new(api.channel.clone());
    let mut project = ProjectServiceClient::new(api.channel.clone());
    let mut user = UserServiceClient::new(api.channel.clone());

    // 呼び出し方(利用者なし・派遣社員)ごとに、全 RPC の結果のコードを集める
    macro_rules! codes {
        ($wrap:expr) => {{
            let code = |r: Result<(), Status>| r.err().map(|s| s.code());
            vec![
                (
                    "CreatePayslip",
                    true,
                    code(
                        payroll
                            .create_payslip($wrap(proto::CreatePayslipRequest::default()))
                            .await
                            .map(|_| ()),
                    ),
                ),
                (
                    "FinalizePayslip",
                    true,
                    code(
                        payroll
                            .finalize_payslip($wrap(proto::FinalizePayslipRequest::default()))
                            .await
                            .map(|_| ()),
                    ),
                ),
                (
                    "GetPayslip",
                    false,
                    code(
                        payroll
                            .get_payslip($wrap(proto::GetPayslipRequest::default()))
                            .await
                            .map(|_| ()),
                    ),
                ),
                (
                    "ListPayslips",
                    false,
                    code(
                        payroll
                            .list_payslips($wrap(proto::ListPayslipsRequest::default()))
                            .await
                            .map(|_| ()),
                    ),
                ),
                (
                    "CreateStaff",
                    true,
                    code(
                        staff
                            .create_staff($wrap(proto::CreateStaffRequest::default()))
                            .await
                            .map(|_| ()),
                    ),
                ),
                (
                    "ListStaff",
                    true,
                    code(
                        staff
                            .list_staff($wrap(proto::ListStaffRequest::default()))
                            .await
                            .map(|_| ()),
                    ),
                ),
                (
                    "GetMe",
                    false,
                    code(staff.get_me($wrap(proto::GetMeRequest::default())).await.map(|_| ())),
                ),
                (
                    "CreateProject",
                    true,
                    code(
                        project
                            .create_project($wrap(proto::CreateProjectRequest::default()))
                            .await
                            .map(|_| ()),
                    ),
                ),
                (
                    "ListProjects",
                    true,
                    code(
                        project
                            .list_projects($wrap(proto::ListProjectsRequest::default()))
                            .await
                            .map(|_| ()),
                    ),
                ),
                (
                    "CreateAdminUser",
                    true,
                    code(
                        user.create_admin_user($wrap(proto::CreateAdminUserRequest::default()))
                            .await
                            .map(|_| ()),
                    ),
                ),
            ]
        }};
    }

    for (rpc, _, code) in codes!(GrpcRequest::new) {
        assert_eq!(code, Some(Code::Unauthenticated), "{rpc} はログインしていなければ呼べない");
    }
    for (rpc, admin_only, code) in codes!(as_staff) {
        if admin_only {
            assert_eq!(code, Some(Code::PermissionDenied), "{rpc} は管理者だけが呼べる");
        } else {
            assert_ne!(code, Some(Code::PermissionDenied), "{rpc} は派遣社員も呼べる");
            assert_ne!(code, Some(Code::Unauthenticated), "{rpc} は派遣社員も呼べる");
        }
    }
}
