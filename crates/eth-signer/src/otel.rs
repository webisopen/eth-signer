use std::sync::OnceLock;

use opentelemetry::global;
use opentelemetry_otlp::{LogExporter, MetricExporter, Protocol, SpanExporter, WithExportConfig};
use opentelemetry_sdk::{
    Resource, logs::SdkLoggerProvider, metrics::SdkMeterProvider, trace::SdkTracerProvider,
};
use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt};

fn get_resource() -> Resource {
    static RESOURCE: OnceLock<Resource> = OnceLock::new();

    RESOURCE
        .get_or_init(|| Resource::builder().with_service_name("eth-signer").build())
        .clone()
}

fn init_logs() -> SdkLoggerProvider {
    let exporter = LogExporter::builder()
        .with_http()
        .with_protocol(Protocol::HttpBinary)
        .build()
        .expect("Failed to create log exporter");

    SdkLoggerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(get_resource())
        .build()
}

pub(super) fn init_metrics() -> SdkMeterProvider {
    let exporter = MetricExporter::builder()
        .with_http()
        .with_protocol(Protocol::HttpBinary)
        .build()
        .expect("Failed to create metric exporter");

    SdkMeterProvider::builder()
        .with_periodic_exporter(exporter)
        .with_resource(get_resource())
        .build()
}

pub(super) fn init_traces() -> SdkTracerProvider {
    let exporter = SpanExporter::builder()
        .with_http()
        // .with_endpoint(endpoint)
        .with_protocol(Protocol::HttpJson)
        .build()
        .expect("failed to create span exporter");

    SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(get_resource())
        .build()
}

pub fn get_env_filter() -> EnvFilter {
    #[cfg(debug_assertions)]
    {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| "debug".into())
    }
    #[cfg(not(debug_assertions))]
    {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into())
    }
}

pub fn init(debug: bool) {
    let tracer_provider = init_traces();
    let meter_provider = init_metrics();
    let logger_provider = init_logs();

    let env_filter = get_env_filter();

    let filter_otel = env_filter
        .clone()
        .add_directive("hyper=off".parse().unwrap())
        .add_directive("tonic=off".parse().unwrap())
        .add_directive("h2=off".parse().unwrap())
        .add_directive("reqwest=off".parse().unwrap());
    let otel_layer =
        opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge::new(&logger_provider)
            .with_filter(filter_otel);

    let fmt_layer = match debug {
        true => tracing_subscriber::fmt::layer()
            .with_thread_names(true)
            .with_file(true)
            .with_line_number(true)
            .boxed(),
        false => tracing_subscriber::fmt::layer()
            .json()
            .flatten_event(true)
            .with_thread_names(true)
            .with_file(true)
            .with_line_number(true)
            .boxed(),
    }
    .with_filter(env_filter);

    tracing_subscriber::registry()
        .with(otel_layer)
        .with(fmt_layer)
        .init();

    global::set_tracer_provider(tracer_provider);
    global::set_meter_provider(meter_provider);
}
