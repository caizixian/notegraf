use notegraf::notestore::BoxedNoteStore;
use notegraf::{InMemoryStore, PostgreSQLStoreBuilder};
use sqlx::postgres::PgConnectOptions;
use sqlx::{ConnectOptions, Connection, Executor, PgConnection};
use tracing::log::LevelFilter;
use uuid::Uuid;

#[derive(serde::Deserialize, Debug)]
pub enum NoteStoreType {
    InMemory,
    PostgreSQL,
}

#[derive(serde::Deserialize, Debug)]
pub struct Settings {
    database: Option<DatabaseSettings>,
    pub host: String,
    pub port: u16,
    pub debug: bool,
    notestoretype: NoteStoreType,
    populatetestdata: bool,
    pub otlpendpoint: Option<String>,
    pub loglevel: Option<String>,
}

impl Settings {
    pub async fn get_note_store(
        &self,
        random_db: bool,
        log_statement_filter: LevelFilter,
    ) -> BoxedNoteStore<crate::NoteType> {
        let store: BoxedNoteStore<crate::NoteType> = match self.notestoretype {
            NoteStoreType::InMemory => Box::new(InMemoryStore::new()),
            NoteStoreType::PostgreSQL => {
                let database_settings = CONFIGURATION.database
                    .as_ref()
                    .expect("When notestoretype is set to PostgreSQL, you must configure the keys under database");
                let mut db_options = if random_db {
                    let db_name = Uuid::new_v4().to_string();
                    let db_options = database_settings.options_without_db();
                    let mut connection = PgConnection::connect_with(&db_options)
                        .await
                        .expect("Failed to connect to Postgres");
                    connection
                        .execute(&*format!(r#"CREATE DATABASE "{}";"#, db_name))
                        .await
                        .expect("Failed to create database.");
                    db_options.database(&db_name)
                } else {
                    database_settings.options()
                };
                db_options.log_statements(log_statement_filter);
                Box::new(PostgreSQLStoreBuilder::new(db_options).build().await)
            }
        };
        if cfg!(feature = "notetype_markdown") && self.populatetestdata {
            notegraf::notestore::util::populate_test_data(&store).await;
        }
        store
    }
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct DatabaseSettings {
    pub port: String,
    pub host: String,
    pub name: String,
    pub username: Option<String>,
    pub password: Option<String>,
}

impl DatabaseSettings {
    pub fn options(&self) -> PgConnectOptions {
        self.options_without_db().database(&self.name)
    }

    pub fn options_without_db(&self) -> PgConnectOptions {
        let options = PgConnectOptions::new()
            .host(&self.host)
            .port(self.port.parse().expect("Failed to parse port number"));
        if let Some(ref username) = self.username {
            let password = self
                .password
                .as_ref()
                .expect("Password expected when a username is set");
            options.username(username).password(password)
        } else {
            options
        }
    }
}

lazy_static! {
    pub static ref CONFIGURATION: Settings =
        get_configuration().expect("Failed to read configuration.yml.");
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let config = config::Config::builder()
        .set_default("debug", false)?
        .set_default("host", "localhost")?
        .set_default("populatetestdata", false)?
        .add_source(config::File::with_name("configuration").required(false))
        .add_source(
            config::Environment::default()
                .prefix("notegraf")
                .separator("_"),
        )
        .build()?;
    config.try_deserialize()
}
