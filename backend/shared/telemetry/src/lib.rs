//! ログ(stdout)とトレース(OTLP)の初期化と、プロセスをまたぐトレースの受け渡し。
//!
//! アプリは常に OTLP で出すだけ。ローカルは Jaeger、AWS では ADOT collector が受ける。
//! プロセスをまたぐとき(HTTP・outbox → SQS → Lambda)は W3C の traceparent で親を引き継ぐ。

use std::collections::HashMap;

use opentelemetry::KeyValue;
use opentelemetry::propagation::{Extractor, Injector, TextMapPropagator};
use opentelemetry::trace::{TraceContextExt as _, TracerProvider as _};
use opentelemetry_otlp::{SpanExporter, WithExportConfig};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::{Sampler, SdkTracerProvider};
use tracing_opentelemetry::OpenTelemetrySpanExt as _;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub struct TelemetryConfig<'a> {
    pub service_name: &'a str,
    /// どの環境か(deployment.environment)。トレースを環境で絞り込むのに使う
    pub environment: &'a str,
    /// 動いている版(service.version)
    pub service_version: &'a str,
    /// 新しく始まるトレースのうち、送る割合(0.0〜1.0)。親から引き継いだトレースは親に従う
    pub sample_ratio: f64,
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

impl TelemetryGuard {
    /// 溜まっているスパンを今すぐ送る。Lambda は呼び出しの合間に止まるので、呼び出しごとに呼ぶ
    pub fn flush(&self) {
        if let Some(provider) = &self.provider
            && let Err(err) = provider.force_flush()
        {
            tracing::warn!(error = %err, "failed to flush spans");
        }
    }
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
                    .with_sampler(Sampler::ParentBased(Box::new(Sampler::TraceIdRatioBased(
                        config.sample_ratio,
                    ))))
                    .with_resource(
                        Resource::builder()
                            .with_service_name(config.service_name.to_owned())
                            .with_attributes([
                                KeyValue::new(
                                    "deployment.environment",
                                    config.environment.to_owned(),
                                ),
                                KeyValue::new("service.version", config.service_version.to_owned()),
                            ])
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

/// 今のスパンの traceparent(W3C Trace Context)。トレースを出していないときは `None`
#[must_use]
pub fn current_traceparent() -> Option<String> {
    let mut carrier = HashMap::new();
    TraceContextPropagator::new()
        .inject_context(&tracing::Span::current().context(), &mut Carrier(&mut carrier));
    carrier.remove("traceparent")
}

/// 受け取った traceparent・tracestate を `span` の親にする。`get` は名前から値を引く
/// (HTTP ヘッダ・SQS のメッセージ属性など)。なければ何もしない
pub fn set_parent(span: &tracing::Span, get: impl Fn(&str) -> Option<String>) {
    let mut carrier = HashMap::new();
    for key in ["traceparent", "tracestate"] {
        if let Some(value) = get(key) {
            carrier.insert(key.to_owned(), value);
        }
    }
    if carrier.is_empty() {
        return;
    }
    let cx = TraceContextPropagator::new().extract(&Carrier(&mut carrier));
    if let Err(err) = span.set_parent(cx) {
        tracing::debug!(error = ?err, "failed to set the parent span");
    }
}

/// `span` のトレース ID。ログにトレースを結びつけるのに使う。トレースを出していないときは `None`
#[must_use]
pub fn trace_id(span: &tracing::Span) -> Option<String> {
    let cx = span.context();
    let span_context = cx.span().span_context().clone();
    span_context.is_valid().then(|| span_context.trace_id().to_string())
}

struct Carrier<'a>(&'a mut HashMap<String, String>);

impl Injector for Carrier<'_> {
    fn set(&mut self, key: &str, value: String) {
        self.0.insert(key.to_owned(), value);
    }
}

impl Extractor for Carrier<'_> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(String::as_str)
    }

    fn keys(&self) -> Vec<&str> {
        self.0.keys().map(String::as_str).collect()
    }
}
