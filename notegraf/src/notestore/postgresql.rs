use crate::errors::NoteStoreError;
use crate::notemetadata::NoteMetadata;
use crate::{Note, NoteID, NoteLocator, NoteStore, NoteType, Revision};
use futures::future::BoxFuture;
use sqlx::postgres::PgConnectOptions;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::marker::PhantomData;
use std::path::Path;
use uuid::Uuid;

pub struct PostgreSQLStoreBuilder<T> {
    db_options: PgConnectOptions,
    _phantom: PhantomData<T>,
}

impl<T: NoteType> PostgreSQLStoreBuilder<T> {
    pub fn new(db_options: PgConnectOptions) -> Self {
        Self {
            db_options,
            _phantom: PhantomData,
        }
    }

    pub async fn build(self) -> PostgreSQLStore<T> {
        let mut connection = PgConnection::connect_with(&self.db_options)
            .await
            .expect("Failed to connect to Postgres");
        let db_name = Uuid::new_v4().to_string();
        connection
            .execute(&*format!(r#"CREATE DATABASE "{}";"#, db_name))
            .await
            .expect("Failed to create database.");

        let connection_pool = PgPool::connect_with(self.db_options)
            .await
            .expect("Failed to connect to Postgres.");
        sqlx::migrate!("./migrations")
            .run(&connection_pool)
            .await
            .expect("Failed to migrate the database");
        PostgreSQLStore {
            db_pool: connection_pool,
            _phantom: PhantomData,
        }
    }
}

pub struct PostgreSQLStore<T> {
    db_pool: PgPool,
    _phantom: PhantomData<T>,
}

impl<T: NoteType> NoteStore<T> for PostgreSQLStore<T> {
    fn new_note(
        &self,
        title: String,
        note_inner: T,
        metadata: Option<NoteMetadata>,
    ) -> BoxFuture<Result<NoteLocator, NoteStoreError>> {
        todo!()
    }

    fn get_note<'a>(
        &'a self,
        loc: &'a NoteLocator,
    ) -> BoxFuture<'a, Result<Box<dyn Note<T>>, NoteStoreError>> {
        todo!()
    }

    fn update_note<'a>(
        &'a self,
        loc: &'a NoteLocator,
        title: Option<String>,
        note_inner: Option<T>,
        note_metadata: Option<NoteMetadata>,
    ) -> BoxFuture<'a, Result<NoteLocator, NoteStoreError>> {
        todo!()
    }

    fn delete_note<'a>(
        &'a self,
        loc: &'a NoteLocator,
    ) -> BoxFuture<'a, Result<(), NoteStoreError>> {
        todo!()
    }

    fn get_current_revision<'a>(
        &'a self,
        loc: &'a NoteLocator,
    ) -> BoxFuture<'a, Result<Revision, NoteStoreError>> {
        todo!()
    }

    fn get_revisions<'a>(
        &'a self,
        loc: &'a NoteLocator,
    ) -> BoxFuture<'a, Result<Vec<Revision>, NoteStoreError>> {
        todo!()
    }

    fn append_note<'a>(
        &'a self,
        last: &'a NoteLocator,
        next: &'a NoteID,
    ) -> BoxFuture<'a, Result<(), NoteStoreError>> {
        todo!()
    }

    fn add_branch<'a>(
        &'a self,
        parent: &'a NoteLocator,
        child: &'a NoteID,
    ) -> BoxFuture<'a, Result<(), NoteStoreError>> {
        todo!()
    }

    fn backup(&self, path: Box<dyn AsRef<Path> + Send>) -> BoxFuture<Result<(), NoteStoreError>> {
        todo!()
    }

    fn restore<P: AsRef<Path>>(path: P) -> Result<Self, NoteStoreError>
    where
        Self: Sized,
    {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notestore::tests as common_tests;
    use crate::notetype::PlainNote;
    use std::env;

    /// Configure the connect options with the following environment variables
    ///
    /// NOTEGRAF_DATABASE_HOST: default "localhost"
    /// NOTEGRAF_DATABASE_PORT: default "5432"
    /// NOTEGRAF_DATABASE_USERNAME: default not set
    /// NOTEGRAF_DATABASE_PASSWORD: default not set
    fn get_connect_options() -> PgConnectOptions {
        let host = env::var("NOTEGRAF_DATABASE_HOST").unwrap_or("localhost".to_owned());
        let port = env::var("NOTEGRAF_DATABASE_PORT").unwrap_or("5432".to_owned());
        let username = env::var("NOTEGRAF_DATABASE_PORT");
        let password = env::var("NOTEGRAF_DATABASE_PASSWORD");
        let options = PgConnectOptions::new()
            .host(&host)
            .port(port.parse().expect("Failed to parse port number"));
        if let Ok(ref u) = username {
            let p = password
                .as_ref()
                .expect("Password expected when a username is set");
            options.username(u).password(p)
        } else {
            options
        }
    }

    async fn get_store() -> PostgreSQLStore<PlainNote> {
        PostgreSQLStoreBuilder::new(get_connect_options())
            .build()
            .await
    }

    #[tokio::test]
    async fn unique_id() {
        common_tests::unique_id(get_store().await).await;
    }
}
