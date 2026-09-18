//! ログ(stdout)とトレース(OTLP)の初期化。
//!
//! アプリは常に OTLP で出すだけ。ローカルは Jaeger、AWS では ADOT collector が受ける。

use opentelemetry::trace::TracerProvider as _;
use opentelemetry_otlp::{SpanExporter, WithExportConfig};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::trace::SdkTracerProvider;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub struct TelemetryConfig<'a> {
    pub service_name: &'a str,
    /// None なら OTLP を出さない(テストや一時的な実行向け)
    pub otlp_endpoint: Option<&'a str>,
    /// true なら JSON ログ(CloudWatch Logs で構造化して読める)
    pub json_logs: bool,
    /// `RUST_LOG` がないときのフィルタ
    pub default_filter: &'a str,
}

/// drop 時に未送信のスパンを流し切る
pub struct TelemetryGuard {
    provider: Option<SdkTracerProvider>,
}

impl Drop for TelemetryGuard {
    fn drop(&mut self) {
        if let Some(provider) = self.provider.take()
            && let Err(err) = provider.shutdown()
        {
            eprintln!("failed to shut down tracer provider: {err}");
        }
    }
}

pub fn init(config: &TelemetryConfig<'_>) -> Result<TelemetryGuard, Box<dyn std::error::Error>> {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(config.default_filter));

    let provider = match config.otlp_endpoint {
        Some(endpoint) => {
            let exporter = SpanExporter::builder().with_tonic().with_endpoint(endpoint).build()?;
            Some(
                SdkTracerProvider::builder()
                    .with_batch_exporter(exporter)
                    .with_resource(
                        Resource::builder()
                            .with_service_name(config.service_name.to_owned())
                            .build(),
                    )
                    .build(),
            )
        }
        None => None,
    };

    let otel_layer = provider.as_ref().map(|p| {
        tracing_opentelemetry::layer().with_tracer(p.tracer(config.service_name.to_owned()))
    });

    let registry = tracing_subscriber::registry().with(filter).with(otel_layer);
    if config.json_logs {
        registry.with(tracing_subscriber::fmt::layer().json()).try_init()?;
    } else {
        registry.with(tracing_subscriber::fmt::layer()).try_init()?;
    }

    Ok(TelemetryGuard { provider })
}
