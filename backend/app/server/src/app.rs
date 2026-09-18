use std::time::Duration;

use anyhow::Context as _;
use axum::Router;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::get;
use bootstrap::AppConfig;
use http::{HeaderName, HeaderValue, Method};
use payroll_handler::authenticate;
use platform_gen::acme::payroll::v1::payroll_service_server::PayrollServiceServer;
use platform_gen::acme::payroll::v1::project_service_server::ProjectServiceServer;
use platform_gen::acme::payroll::v1::staff_service_server::StaffServiceServer;
use sqlx::MySqlPool;
use tokio_util::sync::CancellationToken;
use tonic::service::Routes;
use tonic_web::GrpcWebLayer;
use tower_http::cors::{AllowOrigin, CorsLayer};

pub async fn run(config: AppConfig) -> anyhow::Result<()> {
    let aws = bootstrap::aws_config().await;
    let pool = bootstrap::connect_db(&config, &aws).await?;
    let handlers = bootstrap::build_handlers(&pool, bootstrap::build_user_directory(&config, &aws));
    let verifier = bootstrap::build_verifier(&config);

    // gRPC-Webを有効にしてブラウザ(Connect-ES)から直接叩けるようにする。
    // 認証ミドルウェアがJWTを検証し、AuthenticatedUser を extensions に載せる
    let grpc = Routes::new(PayrollServiceServer::new(handlers.payroll))
        .add_service(StaffServiceServer::new(handlers.staff))
        .add_service(ProjectServiceServer::new(handlers.project))
        .into_axum_router()
        .layer(axum::middleware::from_fn_with_state(verifier, authenticate))
        .layer(GrpcWebLayer::new());

    // /health は認証の外。ロードバランサーと E2E の起動待ちに使う
    let app = Router::new()
        .route("/health", get(health))
        .with_state(pool.clone())
        .merge(grpc)
        .layer(cors(&config.server.cors_allowed_origins)?);

    // outbox relay を常駐タスクとして動かす。SKIP LOCKED なので複数インスタンスでも安全
    let cancel = CancellationToken::new();
    let relay = tokio::spawn(payroll_infrastructure::messaging::relay::run(
        pool.clone(),
        bootstrap::sqs_client(&config, &aws),
        config.messaging.queue_url.clone(),
        Duration::from_millis(config.messaging.relay_interval_ms),
        cancel.clone(),
    ));

    let listener = tokio::net::TcpListener::bind(&config.server.addr)
        .await
        .with_context(|| format!("failed to bind {}", config.server.addr))?;
    tracing::info!(addr = %config.server.addr, env = %config.env, "server started");

    // SIGTERMを受けたら新規リクエストを止め、処理中のリクエストを終えてから停止する
    let grace = Duration::from_secs(config.server.shutdown_grace_seconds);
    let shutdown = cancel.clone();
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            shutdown_signal().await;
            tracing::info!("shutdown signal received");
            shutdown.cancel();
        })
        .await?;

    // relay が今のバッチを送り終えるのを待つ(上限あり)
    if tokio::time::timeout(grace, relay).await.is_err() {
        tracing::warn!("relay did not stop within grace period");
    }
    pool.close().await;
    tracing::info!("server stopped");
    Ok(())
}

async fn health(State(pool): State<MySqlPool>) -> (StatusCode, &'static str) {
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

async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut sig) => {
                sig.recv().await;
            }
            Err(_) => std::future::pending::<()>().await,
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {}
        () = terminate => {}
    }
}
