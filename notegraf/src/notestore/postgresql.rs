use crate::errors::NoteStoreError;
use crate::notemetadata::NoteMetadata;
use crate::{Note, NoteID, NoteLocator, NoteStore, NoteType, Revision};
use chrono::{DateTime, Utc};
use futures::future::BoxFuture;
use sqlx::postgres::{PgConnectOptions, PgQueryResult};
use sqlx::{Connection, Executor, PgConnection, PgPool, Postgres, Transaction};
use std::collections::HashSet;
use std::marker::PhantomData;
use std::path::Path;
use uuid::Uuid;

pub struct PostgreSQLStoreBuilder<T> {
    db_options: PgConnectOptions,
    _phantom: PhantomData<T>,
}

#[derive(sqlx::FromRow)]
struct PostgreSQLNoteRaw {
    revision: Uuid,
    id: Uuid,
    is_current: Option<bool>,
    title: String,
    note_inner: String,
    parent: Option<Uuid>,
    branches: Option<Vec<Uuid>>,
    prev: Option<Uuid>,
    next: Option<Vec<Uuid>>,
    referents: Vec<Uuid>,
    references: Option<Vec<Uuid>>,
    metadata_schema_version: i64,
    metadata_created_at: DateTime<Utc>,
    metadata_modified_at: DateTime<Utc>,
    metadata_tags: Vec<String>,
    metadata_custom_metadata: serde_json::Value,
}

#[derive(Debug, Clone)]
struct PostgreSQLNote<T> {
    title: String,
    note_inner: T,
    id: NoteID,
    revision: Revision,
    parent: Option<NoteID>,
    branches: HashSet<NoteID>,
    prev: Option<NoteID>,
    next: Option<NoteID>,
    referents: HashSet<NoteID>,
    references: HashSet<NoteID>,
    metadata: NoteMetadata,
}

impl<T> Note<T> for PostgreSQLNote<T>
where
    T: NoteType,
{
    fn get_title(&self) -> String {
        self.title.clone()
    }

    fn get_note_inner(&self) -> T {
        self.note_inner.clone()
    }

    fn get_id(&self) -> NoteID {
        self.id.clone()
    }

    fn get_revision(&self) -> Revision {
        self.revision.clone()
    }

    fn get_parent(&self) -> Option<NoteID> {
        self.parent.clone()
    }

    fn get_branches(&self) -> HashSet<NoteID> {
        self.branches.clone()
    }

    fn get_prev(&self) -> Option<NoteID> {
        self.prev.clone()
    }

    fn get_next(&self) -> Option<NoteID> {
        self.next.clone()
    }

    fn get_references(&self) -> HashSet<NoteID> {
        self.references.clone()
    }

    fn get_referents(&self) -> HashSet<NoteID> {
        self.referents.clone()
    }

    fn get_metadata(&self) -> NoteMetadata {
        self.metadata.clone()
    }
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

    async fn get_note_current(
        transaction: &mut Transaction<'_, Postgres>,
        id: Uuid,
    ) -> Result<PostgreSQLNoteRaw, NoteStoreError> {
        let res = sqlx::query_as!(
            PostgreSQLNoteRaw,
            r#"
            SELECT
                revision.revision,
                revision.id,
                true AS is_current,
                revision.title,
                revision.note_inner,
                revision.parent,
                array_remove(array_agg(revision1.id), NULL) AS branches,
                revision.prev,
                array_remove(array_agg(revision2.id), NULL) AS next,
                revision.referents,
                array_remove(array_agg(revision3.id), NULL) AS "references",
                revision.metadata_schema_version,
                revision.metadata_created_at,
                revision.metadata_modified_at,
                revision.metadata_tags,
                revision.metadata_custom_metadata
            FROM revision
            LEFT JOIN current_revision ON revision.id = current_revision.id
            LEFT JOIN revision AS revision1 on revision1.parent = revision.id
            LEFT JOIN revision AS revision2 on revision2.prev = revision.id
            LEFT JOIN revision AS revision3 on revision.id = ANY(revision3.referents)
            WHERE revision.id = $1 AND current_revision.current_revision IS NOT NULL
            GROUP BY revision.revision
            "#,
            id,
        )
        .fetch_one(transaction)
        .await;
        if let Err(sqlx::Error::RowNotFound) = res {
            return Err(NoteStoreError::NoteNotExist(id.to_string().into()));
        } else {
            res.map_err(NoteStoreError::PostgreSQLError)
        }
    }

    async fn get_note_specific(
        transaction: &mut Transaction<'_, Postgres>,
        id: Uuid,
        revision: Uuid,
    ) -> Result<PostgreSQLNoteRaw, NoteStoreError> {
        let res = sqlx::query_as!(
            PostgreSQLNoteRaw,
            r#"
            SELECT
                revision.revision,
                revision.id,
                every(revision.revision = current_revision.current_revision
                    AND current_revision.current_revision IS NOT NULL) AS is_current,
                revision.title,
                revision.note_inner,
                revision.parent,
                array_remove(array_agg(revision1.id), NULL) AS branches,
                revision.prev,
                array_remove(array_agg(revision2.id), NULL) AS next,
                revision.referents,
                array_remove(array_agg(revision3.id), NULL) AS "references",
                revision.metadata_schema_version,
                revision.metadata_created_at,
                revision.metadata_modified_at,
                revision.metadata_tags,
                revision.metadata_custom_metadata
            FROM revision
            LEFT JOIN current_revision ON revision.id = current_revision.id
            LEFT JOIN revision AS revision1 on revision1.parent = revision.id
            LEFT JOIN revision AS revision2 on revision2.prev = revision.id
            LEFT JOIN revision AS revision3 on revision.id = ANY(revision3.referents)
            WHERE revision.id = $1 AND revision.revision = $2
            GROUP BY revision.revision
            "#,
            id,
            revision
        )
        .fetch_one(transaction)
        .await;
        if let Err(sqlx::Error::RowNotFound) = res {
            return Err(NoteStoreError::RevisionNotExist(
                id.to_string().into(),
                revision.to_string().into(),
            ));
        } else {
            res.map_err(NoteStoreError::PostgreSQLError)
        }
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
        .map_err(NoteStoreError::PostgreSQLError)?;
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
