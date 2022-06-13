use crate::configuration::Settings;
use opentelemetry::sdk::trace::{self, Tracer};
use opentelemetry::sdk::Resource;
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use tracing::Subscriber;
use tracing_log::LogTracer;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

pub fn get_otlp_tracer(configuration: &Settings) -> Option<Tracer> {
    let end_point = configuration.otlpendpoint.as_ref()?;
    let otlp_exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(end_point);
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(otlp_exporter)
        .with_trace_config(
            trace::config().with_resource(Resource::new(vec![KeyValue::new(
                "service.name",
                format!(
                    "goxidize{}",
                    if configuration.debug { "_debug" } else { "" }
                ),
            )])),
        )
        // https://github.com/open-telemetry/opentelemetry-rust/issues/536#issuecomment-840197611
        .install_batch(opentelemetry::runtime::TokioCurrentThread)
        .expect("Failed to create an opentelemetry_otlp tracer");
    Some(tracer)
}

pub fn get_subscriber(
    configuration: &Settings,
) -> impl Subscriber + Send + Sync + for<'span> LookupSpan<'span> {
    let default_logging_level = if configuration.debug { "debug" } else { "info" };
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_logging_level));
    Registry::default().with(env_filter)
}

pub fn init_tracing(subscriber: impl Subscriber + Send + Sync + for<'span> LookupSpan<'span>) {
    LogTracer::init().expect("Failed to init LogTracer");
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set the default tracing subscriber");
}
