use lazy_static::lazy_static;
use notegraf_web::configuration::CONFIGURATION;
use notegraf_web::startup::run;
use notegraf_web::telemetry::{get_subscriber, init_tracing};
use std::net::TcpListener;
use tracing::log::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;

lazy_static! {
    static ref TRACING: () = {
        let subscriber = get_subscriber(&CONFIGURATION)
            .with(tracing_subscriber::fmt::Layer::default().with_test_writer());
        init_tracing(subscriber);
    };
}

pub struct TestApp {
    pub address: String,
}

pub async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    // We retrieve the port assigned to us by the OS
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{port}");
    lazy_static::initialize(&TRACING);
    let store = CONFIGURATION.get_note_store(true, LevelFilter::Debug).await;
    let server = run(listener, store, CONFIGURATION.debug).expect("Failed to bind address");
    drop(tokio::spawn(server));
    TestApp { address }
}
