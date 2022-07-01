use crate::errors::NoteStoreError;
use crate::notemetadata::NoteMetadata;
use crate::{Note, NoteID, NoteLocator, NoteStore, NoteType, Revision};
use futures::future::BoxFuture;
use sqlx::postgres::PgConnectOptions;
use sqlx::{query, PgPool};
use std::collections::HashSet;
use std::marker::PhantomData;
use std::path::Path;
use uuid::Uuid;

mod queries;
use queries::*;

#[cfg(test)]
mod tests;

pub struct PostgreSQLStoreBuilder<T> {
    db_options: PgConnectOptions,
    _phantom: PhantomData<T>,
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

    async fn is_deleted(&self, id: Uuid) -> Result<bool, NoteStoreError> {
        let mut transaction = self.db_pool.begin().await?;
        is_deleted(&mut transaction, id).await
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
            query!(r#"INSERT INTO note(id) VALUES ($1)"#, &note_id)
                .execute(&mut transaction)
                .await?;
            let n = PostgreSQLNoteEditable {
                id: note_id,
                revision,
                title,
                note_inner,
                prev: None,
                parent: None,
                metadata: metadata.unwrap_or_default(),
            };
            insert_revision(&mut transaction, n).await?;
            upsert_current_revision(&mut transaction, note_id, revision).await?;
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
                    let id = i.try_to_uuid()?;
                    get_note_current(&mut transaction, id).await?
                }
                NoteLocator::Specific(ref i, ref r) => {
                    let id = i.try_to_uuid()?;
                    let revision = r.try_to_uuid()?;
                    get_note_specific(&mut transaction, id, revision).await?
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
        Box::pin(async move {
            let mut transaction = self.db_pool.begin().await?;
            let (id, rev) = loc.unpack();
            if !is_current(&mut transaction, loc).await? {
                return Err(NoteStoreError::DeleteOldRevision(
                    id.clone(),
                    rev.unwrap().clone(),
                ));
            }
            let note = match loc {
                NoteLocator::Current(n) => {
                    get_note_current(&mut transaction, n.try_to_uuid()?).await?
                }
                NoteLocator::Specific(n, r) => {
                    get_note_specific(&mut transaction, n.try_to_uuid()?, r.try_to_uuid()?).await?
                }
            };

            if !note.branches.unwrap_or_default().is_empty() {
                return Err(NoteStoreError::HasBranches(id.clone()));
            }
            if !note.references.unwrap_or_default().is_empty() {
                return Err(NoteStoreError::HasReferences(id.clone()));
            }
            if note.next.is_some() && note.prev.is_some() {
                // FIXME
            }
            delete_revision(transaction, loc).await
        })
    }

    fn get_current_revision<'a>(
        &'a self,
        loc: &'a NoteLocator,
    ) -> BoxFuture<'a, Result<Revision, NoteStoreError>> {
        Box::pin(async move {
            let id = loc.get_id().try_to_uuid()?;
            let mut transaction = self.db_pool.begin().await?;
            let res = query!(
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
            let ever_existed = noteid_exist(&mut transaction, id).await?;
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
