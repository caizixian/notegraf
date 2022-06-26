use lazy_static::lazy_static;
use notegraf_web::configuration::CONFIGURATION;
use notegraf_web::startup::run;
use notegraf_web::telemetry::{get_otlp_tracer, get_subscriber, init_tracing};
use std::net::TcpListener;
use tracing::log::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;

lazy_static! {
    static ref TRACING: () = {
        let subscriber =
            get_subscriber(&*CONFIGURATION).with(tracing_subscriber::fmt::Layer::default());
        if let Some(tracer) = get_otlp_tracer(&*CONFIGURATION) {
            let subscriber = subscriber.with(tracing_opentelemetry::layer().with_tracer(tracer));
            init_tracing(subscriber);
        } else {
            init_tracing(subscriber);
        }
    };
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    lazy_static::initialize(&TRACING);
    let address = format!("{}:{}", CONFIGURATION.host, CONFIGURATION.port);
    let listener = TcpListener::bind(address)?;
    run(
        listener,
        CONFIGURATION
            .get_note_store(false, LevelFilter::Debug)
            .await,
        CONFIGURATION.debug,
    )?
    .await?;
    opentelemetry::global::shutdown_tracer_provider();
    Ok(())
}
