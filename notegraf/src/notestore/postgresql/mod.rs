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

fn get_new_noteid() -> Uuid {
    Uuid::new_v4()
}

fn get_new_revision() -> Uuid {
    Uuid::new_v4()
}

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
    #[cfg(test)]
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
            let note_id = get_new_noteid();
            let revision = get_new_revision();
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
            Ok(NoteLocator::Specific(note_id.into(), revision.into()))
        })
    }

    fn get_note<'a>(
        &'a self,
        loc: &'a NoteLocator,
    ) -> BoxFuture<'a, Result<Box<dyn Note<T>>, NoteStoreError>> {
        Box::pin(async move {
            let mut transaction = self.db_pool.begin().await?;
            let note: PostgreSQLNote<T> = get_note_by_loc(&mut transaction, loc).await?.into_note();
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
        Box::pin(async move {
            let mut transaction = self.db_pool.begin().await?;
            let new_loc = update_note_helper(&mut transaction, loc, |old_note| {
                let mut note = old_note.clone();
                if let Some(t) = title {
                    note.title = t;
                }
                if let Some(n) = note_inner {
                    note.note_inner = n;
                }
                if let Some(m) = note_metadata {
                    note.metadata = m;
                }
                Ok(note)
            })
            .await?;
            transaction.commit().await?;
            Ok(new_loc)
        })
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
            let note: PostgreSQLNote<T> = get_note_by_loc(&mut transaction, loc).await?.into_note();
            if !note.branches.is_empty() {
                return Err(NoteStoreError::HasBranches(id.clone()));
            }
            if !note.references.is_empty() {
                return Err(NoteStoreError::HasReferences(id.clone()));
            }
            if note.next.is_some() && note.prev.is_some() {
                update_note_helper::<_, T>(
                    &mut transaction,
                    &NoteLocator::Current(note.next.unwrap()),
                    |old_note| {
                        let mut new_note = old_note.clone();
                        new_note.prev = Some(note.prev.unwrap().to_uuid().unwrap());
                        Ok(new_note)
                    },
                )
                .await?;
            }
            delete_revision(transaction, loc).await
        })
    }

    fn get_current_revision_to_delete<'a>(
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
                    return Ok(row.current_revision.into());
                }
                Err(e) => {
                    if !matches!(e, sqlx::Error::RowNotFound) {
                        return Err(NoteStoreError::PostgreSQLError(e));
                    }
                }
            }
            let ever_existed = noteid_exist(&mut transaction, id).await?;
            if ever_existed {
                Err(NoteStoreError::NoteDeleted(id.into()))
            } else {
                Err(NoteStoreError::NoteNotExist(id.into()))
            }
        })
    }

    fn get_revisions_to_delete<'a>(
        &'a self,
        loc: &'a NoteLocator,
    ) -> BoxFuture<'a, Result<Vec<Revision>, NoteStoreError>> {
        Box::pin(async move {
            let mut transaction = self.db_pool.begin().await?;
            get_revisions_to_delete(&mut transaction, loc)
                .await
                .map(|x| x.iter().map(|y| y.into()).collect())
        })
    }

    fn get_revisions<'a>(
        &'a self,
        _loc: &'a NoteLocator,
    ) -> BoxFuture<'a, Result<Vec<(Revision, Box<dyn Note<T>>)>, NoteStoreError>> {
        todo!()
    }

    fn append_note<'a>(
        &'a self,
        last: &'a NoteID,
        next: &'a NoteID,
    ) -> BoxFuture<'a, Result<(), NoteStoreError>> {
        Box::pin(async move {
            let mut transaction = self.db_pool.begin().await?;
            let last_note: PostgreSQLNote<T> =
                get_note_by_loc(&mut transaction, &NoteLocator::Current(last.clone()))
                    .await?
                    .into_note();
            let last_note_next = last_note.get_next();
            if let Some(n) = last_note_next {
                transaction.rollback().await?;
                return Err(NoteStoreError::ExistingNext(last.clone(), n));
            }
            update_note_helper::<_, T>(
                &mut transaction,
                &NoteLocator::Current(next.clone()),
                |old_note| {
                    let mut note = old_note.clone();
                    note.prev = Some(last.to_uuid().unwrap());
                    Ok(note)
                },
            )
            .await?;
            transaction.commit().await?;
            Ok(())
        })
    }

    fn add_branch<'a>(
        &'a self,
        parent: &'a NoteID,
        child: &'a NoteID,
    ) -> BoxFuture<'a, Result<(), NoteStoreError>> {
        Box::pin(async move {
            let mut transaction = self.db_pool.begin().await?;
            update_note_helper::<_, T>(
                &mut transaction,
                &NoteLocator::Current(child.clone()),
                |old_note| {
                    let mut note = old_note.clone();
                    note.parent = Some(parent.to_uuid().unwrap());
                    Ok(note)
                },
            )
            .await?;
            transaction.commit().await?;
            Ok(())
        })
    }

    fn backup(&self, _path: Box<dyn AsRef<Path> + Send>) -> BoxFuture<Result<(), NoteStoreError>> {
        unimplemented!("Please use PostgreSQL's own backup utilities.")
    }

    fn restore<P: AsRef<Path>>(_path: P) -> Result<Self, NoteStoreError>
    where
        Self: Sized,
    {
        unimplemented!("Please use PostgreSQL's own restore utilities.")
    }
}
