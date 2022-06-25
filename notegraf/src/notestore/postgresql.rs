use crate::errors::NoteStoreError;
use crate::notemetadata::NoteMetadata;
use crate::{Note, NoteID, NoteLocator, NoteStore, NoteType, Revision};
use futures::future::BoxFuture;
use sqlx::postgres::{PgConnectOptions, PgQueryResult};
use sqlx::{Connection, Executor, PgConnection, PgPool, Postgres, Transaction};
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

    pub async fn build(mut self) -> PostgreSQLStore<T> {
        let mut connection = PgConnection::connect_with(&self.db_options)
            .await
            .expect("Failed to connect to Postgres");
        let db_name = Uuid::new_v4().to_string();
        connection
            .execute(&*format!(r#"CREATE DATABASE "{}";"#, db_name))
            .await
            .expect("Failed to create database.");
        self.db_options = self.db_options.database(&db_name);
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

impl<T: NoteType> PostgreSQLStore<T> {
    fn get_new_noteid() -> Uuid {
        Uuid::new_v4()
    }

    fn get_new_revision() -> Uuid {
        Uuid::new_v4()
    }

    async fn insert_revision(
        transaction: &mut Transaction<'_, Postgres>,
        id: Uuid,
        revision: Uuid,
        title: String,
        note_inner: T,
        metadata: Option<NoteMetadata>,
    ) -> Result<NoteLocator, NoteStoreError> {
        let referents: Vec<Uuid> = match note_inner.get_referents() {
            Ok(r) => r,
            Err(e) => return Err(NoteStoreError::NoteInnerError(e.to_string())),
        }
        .iter()
        .map(|x| Uuid::parse_str(x.as_ref()).unwrap())
        .collect();
        let metadata = metadata.unwrap_or_default();
        let tags: Vec<String> = metadata.tags.iter().cloned().collect();
        let note_inner: String = note_inner.clone().into();
        sqlx::query!(
            r#"
            INSERT INTO
                revision(
                    revision, id, title, note_inner, parent, prev, referents,
                    metadata_schema_version, metadata_created_at,
                    metadata_modified_at, metadata_tags, metadata_custom_metadata
                )
            VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
            revision,
            id,
            title,
            &note_inner,
            None as Option<Uuid>,
            None as Option<Uuid>,
            &referents,
            metadata.schema_version as i64,
            metadata.created_at,
            metadata.modified_at,
            &tags,
            metadata.custom_metadata
        )
        .execute(transaction)
        .await
        .map_err(|e| NoteStoreError::PostgreSQLError(e))?;
        Ok(NoteLocator::Specific(
            id.to_string().into(),
            revision.to_string().into(),
        ))
    }

    async fn upsert_current_revision(
        transaction: &mut Transaction<'_, Postgres>,
        id: Uuid,
        revision: Uuid,
    ) -> sqlx::Result<PgQueryResult> {
        sqlx::query!(
            r#"
            INSERT INTO current_revision (id, current_revision)
            VALUES ($1, $2)
            ON CONFLICT (id) DO UPDATE
            SET current_revision = EXCLUDED.current_revision
            "#,
            id,
            revision
        )
        .execute(transaction)
        .await
    }
}

impl<T: NoteType> NoteStore<T> for PostgreSQLStore<T> {
    fn new_note(
        &self,
        title: String,
        note_inner: T,
        metadata: Option<NoteMetadata>,
    ) -> BoxFuture<Result<NoteLocator, NoteStoreError>> {
        Box::pin(async move {
            let note_id = Self::get_new_noteid();
            let revision = Self::get_new_revision();
            let mut transaction = self.db_pool.begin().await?;
            sqlx::query!(r#"INSERT INTO note(id) VALUES ($1)"#, &note_id)
                .execute(&mut transaction)
                .await?;
            Self::insert_revision(
                &mut transaction,
                note_id,
                revision,
                title,
                note_inner,
                metadata,
            )
            .await?;
            Self::upsert_current_revision(&mut transaction, note_id, revision).await?;
            transaction.commit().await?;
            Ok(NoteLocator::Specific(
                note_id.to_string().into(),
                revision.to_string().into(),
            ))
        })
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
        let username = env::var("NOTEGRAF_DATABASE_USERNAME");
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
