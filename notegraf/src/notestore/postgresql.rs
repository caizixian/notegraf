use crate::errors::NoteStoreError;
use crate::notemetadata::NoteMetadata;
use crate::{Note, NoteID, NoteLocator, NoteStore, NoteType, Revision};
use chrono::{DateTime, Utc};
use futures::future::BoxFuture;
use sqlx::postgres::{PgConnectOptions, PgQueryResult};
use sqlx::{PgPool, Postgres, Transaction};
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

impl PostgreSQLNoteRaw {
    fn into_note<T: NoteType>(self) -> PostgreSQLNote<T> {
        let note_inner: T = T::from(self.note_inner);
        let parent: Option<NoteID> = self.parent.map(|x| x.to_string().into());
        let branches: HashSet<NoteID> = match self.branches {
            Some(b) => HashSet::from_iter(b.iter().map(|x| x.to_string().into())),
            None => HashSet::new(),
        };
        let prev: Option<NoteID> = self.prev.map(|x| x.to_string().into());
        let next: Option<NoteID> = match self.next {
            Some(n) => {
                if n.is_empty() {
                    None
                } else {
                    assert_eq!(n.len(), 1);
                    Some(n[0].to_string().into())
                }
            }
            None => None,
        };
        let referents: HashSet<NoteID> =
            HashSet::from_iter(self.referents.iter().map(|x| x.to_string().into()));
        let references: HashSet<NoteID> = match self.references {
            Some(r) => HashSet::from_iter(r.iter().map(|x| x.to_string().into())),
            None => HashSet::new(),
        };
        let metadata = NoteMetadata {
            schema_version: self.metadata_schema_version as u64,
            created_at: self.metadata_created_at,
            modified_at: self.metadata_modified_at,
            tags: HashSet::from_iter(self.metadata_tags.iter().cloned()),
            custom_metadata: self.metadata_custom_metadata,
        };
        PostgreSQLNote {
            title: self.title,
            note_inner,
            id: self.id.to_string().into(),
            revision: self.revision.to_string().into(),
            parent,
            branches,
            prev,
            next,
            referents,
            references,
            metadata,
        }
    }
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

    pub async fn build(self) -> PostgreSQLStore<T> {
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
            LEFT JOIN revision_only_current AS revision1 on revision1.parent = revision.id
            LEFT JOIN revision_only_current AS revision2 on revision2.prev = revision.id
            -- https://stackoverflow.com/a/29245753
            -- indexes are bound to operators, and the indexed expression must be to the left of
            -- the operator
            LEFT JOIN revision_only_current AS revision3 on revision3.referents @> ARRAY[revision.id]
            WHERE revision.id = $1 AND current_revision.current_revision IS NOT NULL
            GROUP BY revision.revision
            "#,
            id,
        )
        .fetch_one(transaction)
        .await;
        if let Err(sqlx::Error::RowNotFound) = res {
            Err(NoteStoreError::NoteNotExist(id.to_string().into()))
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
            LEFT JOIN revision_only_current AS revision1 on revision1.parent = revision.id
            LEFT JOIN revision_only_current AS revision2 on revision2.prev = revision.id
            -- https://stackoverflow.com/a/29245753
            -- indexes are bound to operators, and the indexed expression must be to the left of
            -- the operator
            LEFT JOIN revision_only_current AS revision3 on revision3.referents @> ARRAY[revision.id]
            WHERE revision.id = $1 AND revision.revision = $2
            GROUP BY revision.revision
            "#,
            id,
            revision
        )
        .fetch_one(transaction)
        .await;
        if let Err(sqlx::Error::RowNotFound) = res {
            Err(NoteStoreError::RevisionNotExist(
                id.to_string().into(),
                revision.to_string().into(),
            ))
        } else {
            res.map_err(NoteStoreError::PostgreSQLError)
        }
    }

    #[allow(clippy::too_many_arguments)]
    async fn insert_revision(
        transaction: &mut Transaction<'_, Postgres>,
        id: Uuid,
        revision: Uuid,
        title: String,
        note_inner: T,
        parent: Option<Uuid>,
        prev: Option<Uuid>,
        metadata: NoteMetadata,
    ) -> Result<NoteLocator, NoteStoreError> {
        let referents: Vec<Uuid> = match note_inner.get_referents() {
            Ok(r) => r,
            Err(e) => return Err(NoteStoreError::NoteInnerError(e.to_string())),
        }
        .iter()
        .map(|x| {
            x.to_uuid()
                .ok_or_else(|| NoteStoreError::NotUuid(x.clone().into()))
        })
        .collect::<Result<Vec<Uuid>, NoteStoreError>>()?;
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
            parent,
            prev,
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

    async fn noteid_exist(
        transaction: &mut Transaction<'_, Postgres>,
        id: Uuid,
    ) -> Result<bool, NoteStoreError> {
        let res = sqlx::query!(
            r#"
            SELECT id
            FROM note
            WHERE id = $1
            "#,
            id
        )
        .fetch_one(transaction)
        .await;
        match res {
            Ok(_) => Ok(true),
            Err(e) => {
                if matches!(e, sqlx::Error::RowNotFound) {
                    Ok(false)
                } else {
                    Err(NoteStoreError::PostgreSQLError(e))
                }
            }
        }
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
                None,
                None,
                metadata.unwrap_or_default(),
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
        Box::pin(async move {
            let mut transaction = self.db_pool.begin().await?;
            let note = match loc {
                NoteLocator::Current(ref i) => {
                    let id = i
                        .to_uuid()
                        .ok_or_else(|| NoteStoreError::NotUuid(i.clone().into()))?;
                    Self::get_note_current(&mut transaction, id).await?
                }
                NoteLocator::Specific(ref i, ref r) => {
                    let id = i
                        .to_uuid()
                        .ok_or_else(|| NoteStoreError::NotUuid(i.clone().into()))?;
                    let revision = r
                        .to_uuid()
                        .ok_or_else(|| NoteStoreError::NotUuid(r.clone().into()))?;
                    Self::get_note_specific(&mut transaction, id, revision).await?
                }
            };
            let note: PostgreSQLNote<T> = note.into_note();
            Ok(Box::new(note) as Box<dyn Note<T>>)
        })
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
        Box::pin(async move {
            let id = loc
                .get_id()
                .to_uuid()
                .ok_or_else(|| NoteStoreError::NotUuid(loc.get_id().clone().into()))?;
            let mut transaction = self.db_pool.begin().await?;
            let res = sqlx::query!(
                r#"
                SELECT current_revision
                FROM current_revision
                WHERE id = $1
                "#,
                id
            )
            .fetch_one(&mut transaction)
            .await;
            match res {
                Ok(row) => {
                    return Ok(row.current_revision.to_string().into());
                }
                Err(e) => {
                    if !matches!(e, sqlx::Error::RowNotFound) {
                        return Err(NoteStoreError::PostgreSQLError(e));
                    }
                }
            }
            let ever_existed = Self::noteid_exist(&mut transaction, id).await?;
            if ever_existed {
                Err(NoteStoreError::NoteDeleted(id.to_string().into()))
            } else {
                Err(NoteStoreError::NoteNotExist(id.to_string().into()))
            }
        })
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
    use sqlx::{Connection, Executor, PgConnection};
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
        let options = get_connect_options();
        let mut connection = PgConnection::connect_with(&options)
            .await
            .expect("Failed to connect to Postgres");
        let db_name = Uuid::new_v4().to_string();
        connection
            .execute(&*format!(r#"CREATE DATABASE "{}";"#, db_name))
            .await
            .expect("Failed to create database.");
        PostgreSQLStoreBuilder::new(options.database(&db_name))
            .build()
            .await
    }

    #[tokio::test]
    async fn unique_id() {
        common_tests::unique_id(get_store().await).await;
    }

    #[tokio::test]
    async fn new_note_revision() {
        common_tests::new_note_revision(get_store().await).await;
    }

    #[tokio::test]
    async fn new_note_retrieve() {
        common_tests::new_note_retrieve(get_store().await).await;
    }
}
