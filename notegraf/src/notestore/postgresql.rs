use crate::errors::NoteStoreError;
use crate::notemetadata::NoteMetadata;
use crate::{Note, NoteID, NoteLocator, NoteStore, NoteType, Revision};
use futures::future::BoxFuture;
use sqlx::PgPool;
use std::marker::PhantomData;
use std::path::Path;

pub struct PostgreSQLStore<T> {
    db_pool: PgPool,
    _phantom: PhantomData<T>,
}

impl<T: NoteType> PostgreSQLStore<T> {
    fn new(db_pool: &PgPool) -> Self {
        PostgreSQLStore {
            db_pool: db_pool.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<T: NoteType> NoteStore<T> for PostgreSQLStore<T> {
    fn new_note(
        &self,
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
