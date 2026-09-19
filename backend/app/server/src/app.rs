use std::sync::{Arc, OnceLock};
use std::time::Duration;

use anyhow::Context as _;
use axum::Router;
use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;
use axum::routing::get;
use bootstrap::AppConfig;
use http::{HeaderName, HeaderValue, Method};
use payroll_handler::authenticate;
use platform_gen::acme::payroll::v1::payroll_service_server::PayrollServiceServer;
use platform_gen::acme::payroll::v1::project_service_server::ProjectServiceServer;
use platform_gen::acme::payroll::v1::staff_service_server::StaffServiceServer;
use sqlx::MySqlPool;
use tokio::time::Instant;
use tokio_util::sync::CancellationToken;
use tonic::Status;
use tonic::service::Routes;
use tonic_web::GrpcWebLayer;
use tower::limit::GlobalConcurrencyLimitLayer;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::trace::{DefaultOnFailure, DefaultOnResponse, TraceLayer};
use tracing::{Level, Span};

/// 1リクエストの処理時間の上限。DB のロック待ちの上限(10秒)より長く、ALB のアイドルタイムアウト(60秒)より短くする
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
/// 同時に処理するリクエストの上限。これを超えた分は、空くまで待たせる
const MAX_CONCURRENT_REQUESTS: usize = 256;
/// 受け取るメッセージの上限。給与明細(明細行100件まで)でも十分に収まる
const MAX_MESSAGE_BYTES: usize = 1024 * 1024;

/// リクエストのスパン。呼び出し元(ブラウザ・他のサービス)の traceparent があれば、その続きにする。
/// ログにもトレース ID を載せ、利用者からの問い合わせとトレースを結びつけられるようにする
fn request_span<B>(request: &http::Request<B>) -> Span {
    let span =
        tracing::info_span!("grpc", rpc = %request.uri().path(), trace_id = tracing::field::Empty);
    platform_telemetry::set_parent(&span, |key| {
        request.headers().get(key).and_then(|v| v.to_str().ok()).map(str::to_owned)
    });
    if let Some(trace_id) = platform_telemetry::trace_id(&span) {
        span.record("trace_id", trace_id);
    }
    span
}

/// 処理が時間内に終わらなければ打ち切る。gRPC の DEADLINE_EXCEEDED で返す
async fn request_timeout(request: Request, next: Next) -> Response {
    if let Ok(response) = tokio::time::timeout(REQUEST_TIMEOUT, next.run(request)).await {
        response
    } else {
        tracing::warn!("request timed out");
        Status::deadline_exceeded("時間内に処理できませんでした").into_http()
    }
}

/// 起動し終えたもの。止めるときに使う
struct Started {
    listener: tokio::net::TcpListener,
    app: Router,
    pool: MySqlPool,
    relay: tokio::task::JoinHandle<()>,
    cancel: CancellationToken,
}

pub async fn run(config: AppConfig) -> anyhow::Result<()> {
    // 止める合図は最初に受け取れるようにする。コンテナでは server が PID 1 で、PID 1 はハンドラのない
    // シグナルを無視するので、起動の途中(設定の確認・DB への接続)で SIGTERM が来ると、SIGKILL まで止まらない。
    // 起動の途中で合図が来たら、起動をやめてすぐに終わる
    let mut shutdown = Box::pin(shutdown_signal()?);
    let Started { listener, app, pool, relay, cancel } = tokio::select! {
        started = start(&config) => started?,
        () = &mut shutdown => {
            tracing::info!("shutdown signal received during startup");
            return Ok(());
        }
    };
    tracing::info!(addr = %config.server.addr, env = %config.env, "server started");

    // SIGTERMを受けたら新規リクエストを止め、処理中のリクエストと relay を終えてから停止する。
    // 待つのはシグナルから shutdown_grace_seconds まで。ECS の stopTimeout(30秒)より短くし、
    // 強制終了される前に自分で止まる
    let grace = Duration::from_secs(config.server.shutdown_grace_seconds);
    let deadline = Arc::new(OnceLock::new());
    let server = axum::serve(listener, app).with_graceful_shutdown({
        let (cancel, deadline) = (cancel.clone(), deadline.clone());
        async move {
            shutdown.await;
            tracing::info!("shutdown signal received");
            let _ = deadline.set(Instant::now() + grace);
            cancel.cancel();
        }
    });
    let drain_limit = {
        let (cancel, deadline) = (cancel.clone(), deadline.clone());
        async move {
            cancel.cancelled().await;
            tokio::time::sleep_until(deadline.get().copied().unwrap_or_else(Instant::now)).await;
        }
    };
    tokio::select! {
        result = server => result?,
        () = drain_limit => tracing::warn!("requests did not finish within the grace period"),
    }

    // relay が今送っている1件を終えるのを待つ。時間内に終わらなければ打ち切る(送れなかった出来事は次の起動で送る)
    let deadline = deadline.get().copied().unwrap_or_else(|| Instant::now() + grace);
    let mut relay = relay;
    if tokio::time::timeout_at(deadline, &mut relay).await.is_err() {
        tracing::warn!("relay did not stop within the grace period, aborting");
        relay.abort();
    }
    // 貸し出し中の接続が返らないと待ち続けるので、閉じるのにも上限を置く
    if tokio::time::timeout(Duration::from_secs(2), pool.close()).await.is_err() {
        tracing::warn!("database pool did not close in time");
    }
    tracing::info!("server stopped");
    Ok(())
}

/// 設定を確かめ、DB に接続し、ルーティングと relay を用意して、待ち受けを始める
async fn start(config: &AppConfig) -> anyhow::Result<Started> {
    config.validate_for_server()?;
    let aws = bootstrap::aws_config().await;
    let pool = bootstrap::connect_db(config, &aws).await?;
    let handlers = bootstrap::build_handlers(&pool, bootstrap::build_user_directory(config, &aws));
    let verifier = bootstrap::build_verifier(config);

    // gRPC-Webを有効にしてブラウザ(Connect-ES)から直接叩けるようにする。
    // 認証ミドルウェアがJWTを検証し、AuthenticatedUser を extensions に載せる
    let grpc = Routes::new(
        PayrollServiceServer::new(handlers.payroll).max_decoding_message_size(MAX_MESSAGE_BYTES),
    )
    .add_service(
        StaffServiceServer::new(handlers.staff).max_decoding_message_size(MAX_MESSAGE_BYTES),
    )
    .add_service(
        ProjectServiceServer::new(handlers.project).max_decoding_message_size(MAX_MESSAGE_BYTES),
    )
    .into_axum_router()
    .layer(axum::middleware::from_fn_with_state(verifier, authenticate))
    .layer(axum::middleware::from_fn(request_timeout))
    .layer(GrpcWebLayer::new())
    .layer(GlobalConcurrencyLimitLayer::new(MAX_CONCURRENT_REQUESTS))
    // リクエストごとのスパンと、完了時の1行のログ(RPC・結果・かかった時間)。認証で断ったものも含める
    .layer(
        TraceLayer::new_for_grpc()
            .make_span_with(request_span)
            .on_response(DefaultOnResponse::new().level(Level::INFO))
            .on_failure(DefaultOnFailure::new().level(Level::WARN)),
    );

    // /health と /ready は認証の外。/health はプロセスが動いているか(ALB の生死判定)、
    // /ready は DB にも届くか(E2E の起動待ち)。DB の一時的な不通(RDS のフェイルオーバーなど)で
    // 全タスクが入れ替えられないよう、ALB には DB を見ない /health を使う
    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/ready", get(ready))
        .with_state(pool.clone())
        .merge(grpc)
        .layer(cors(&config.server.cors_allowed_origins)?);

    // outbox relay を常駐タスクとして動かす。複数インスタンスでも、送るのはロックを取れた1つだけ
    let cancel = CancellationToken::new();
    let relay = tokio::spawn(payroll_infrastructure::messaging::relay::run(
        pool.clone(),
        bootstrap::sqs_client(config, &aws),
        config.messaging.queue_url.clone(),
        Duration::from_millis(config.messaging.relay_interval_ms),
        cancel.clone(),
    ));

    let listener = tokio::net::TcpListener::bind(&config.server.addr)
        .await
        .with_context(|| format!("failed to bind {}", config.server.addr))?;

    Ok(Started { listener, app, pool, relay, cancel })
}

async fn ready(State(pool): State<MySqlPool>) -> (StatusCode, &'static str) {
    let ping = tokio::time::timeout(Duration::from_secs(2), sqlx::query("select 1").execute(&pool));
    match ping.await {
        Ok(Ok(_)) => (StatusCode::OK, "ok"),
        _ => (StatusCode::SERVICE_UNAVAILABLE, "database unavailable"),
    }
}

/// ブラウザから gRPC-Web で叩くための CORS。gRPC-Web・Connect の独自ヘッダを許可し、
/// grpc-status などのトレーラー相当ヘッダを JS から読めるように公開する
fn cors(origins: &[String]) -> anyhow::Result<CorsLayer> {
    let origins = origins
        .iter()
        .map(|o| HeaderValue::from_str(o).with_context(|| format!("invalid CORS origin: {o}")))
        .collect::<anyhow::Result<Vec<_>>>()?;

    Ok(CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods([Method::POST, Method::OPTIONS])
        .allow_headers([
            HeaderName::from_static("authorization"),
            HeaderName::from_static("content-type"),
            HeaderName::from_static("x-grpc-web"),
            HeaderName::from_static("x-user-agent"),
            HeaderName::from_static("grpc-timeout"),
            HeaderName::from_static("connect-protocol-version"),
            HeaderName::from_static("connect-timeout-ms"),
        ])
        .expose_headers([
            HeaderName::from_static("grpc-status"),
            HeaderName::from_static("grpc-message"),
            HeaderName::from_static("grpc-status-details-bin"),
        ])
        .max_age(Duration::from_secs(3600)))
}

/// SIGTERM(ECS が止めるとき)か SIGINT(Ctrl+C)が来たら終わる future を返す。
/// 受け取りは呼んだ時点で登録する(await するまで待たない)
fn shutdown_signal() -> anyhow::Result<impl Future<Output = ()>> {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        let mut terminate = signal(SignalKind::terminate()).context("failed to listen SIGTERM")?;
        let mut interrupt = signal(SignalKind::interrupt()).context("failed to listen SIGINT")?;
        Ok(async move {
            tokio::select! {
                _ = terminate.recv() => {}
                _ = interrupt.recv() => {}
            }
        })
    }
    #[cfg(not(unix))]
    {
        Ok(async {
            let _ = tokio::signal::ctrl_c().await;
        })
    }
}
