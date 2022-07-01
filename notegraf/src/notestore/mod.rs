//! Storage backends of notes.
use crate::errors::NoteStoreError;
use crate::note::*;
use crate::notemetadata::NoteMetadata;
use crate::notetype::NoteType;
use futures::future::BoxFuture;
use std::path::Path;

mod in_memory;
mod postgresql;
#[cfg(test)]
mod tests;
pub mod util;

pub use in_memory::InMemoryStore;
pub use postgresql::{PostgreSQLStore, PostgreSQLStoreBuilder};

/// An abstraction for storage backends.
pub trait NoteStore<T>
where
    T: NoteType,
{
    /// Create a new note.
    ///
    /// The storage backend assigns a [`NoteID`] and [`Revision`]
    ///
    /// A [`NoteStore`] can cache the parent-children or reference relationships for performance
    /// reasons, e.g., in the form of a SQL table or in a graph database.
    fn new_note(
        &self,
        title: String,
        note_inner: T,
        metadata: Option<NoteMetadata>,
    ) -> BoxFuture<Result<NoteLocator, NoteStoreError>>;
    /// Get a note.
    ///
    /// Using different variants of the [`NoteLocator`], one can get a specific revision or
    /// the current revision.
    fn get_note<'a>(
        &'a self,
        loc: &'a NoteLocator,
    ) -> BoxFuture<'a, Result<Box<dyn Note<T>>, NoteStoreError>>;
    /// Update the content and metadata of a note.
    ///
    /// The new content will set to be the current revision.
    ///
    /// If a revision is specified, that revision should be the current revision.
    /// This can be used to prevent racy updates to the same note.
    ///
    /// There is no function for rolling back to a specific revision.
    /// The idea is that you can just copy that revision and update the current revision to
    /// that one.
    /// Similarly, you can resurrect a deleted note by updating the note.
    ///
    /// If a [`NoteStore`] caches the parent-children or reference relationships,
    /// it should check the whether any of the relevant fields of note_inner is changed,
    /// and update the the cache accordingly.
    fn update_note<'a>(
        &'a self,
        loc: &'a NoteLocator,
        title: Option<String>,
        note_inner: Option<T>,
        note_metadata: Option<NoteMetadata>,
    ) -> BoxFuture<'a, Result<NoteLocator, NoteStoreError>>;
    /// Delete a note.
    ///
    /// If a revision is specified, that revision should be the current revision.
    /// This can be used to prevent racy updates to the same note.
    ///
    /// Deleting a note only marks the current revision as empty.
    /// It is not truly deleted. You can still fetch the note if you know the revision.
    /// Note that the parent-child relationship between notes is revisioned.
    /// Therefore, we need to update the parent note of the deleted note.
    ///
    /// Need to check for dangling references.
    fn delete_note<'a>(&'a self, loc: &'a NoteLocator)
        -> BoxFuture<'a, Result<(), NoteStoreError>>;
    /// Get the current revision of a note.
    ///
    /// No matter which variant of [`NoteLocator`] is used, we only care about the [`NoteID`].
    fn get_current_revision<'a>(
        &'a self,
        loc: &'a NoteLocator,
    ) -> BoxFuture<'a, Result<Revision, NoteStoreError>>;
    /// Get all revisions of a note, in the order from newer to older.
    ///
    /// No matter which variant of [`NoteLocator`] is used, we only care about the [`NoteID`].
    fn get_revisions<'a>(
        &'a self,
        loc: &'a NoteLocator,
    ) -> BoxFuture<'a, Result<Vec<Revision>, NoteStoreError>>;
    /// Append a note to the last (or only) note in a sequence
    fn append_note<'a>(
        &'a self,
        last: &'a NoteID,
        next: &'a NoteID,
    ) -> BoxFuture<'a, Result<(), NoteStoreError>>;
    /// Add a branch to a note
    fn add_branch<'a>(
        &'a self,
        parent: &'a NoteID,
        child: &'a NoteID,
    ) -> BoxFuture<'a, Result<(), NoteStoreError>>;
    /// Backup the storage to a folder on some filesystem.
    fn backup(&self, path: Box<dyn AsRef<Path> + Send>) -> BoxFuture<Result<(), NoteStoreError>>;
    /// Restore the storage from a folder on some filesystem.
    fn restore<P: AsRef<Path>>(path: P) -> Result<Self, NoteStoreError>
    where
        Self: Sized;
}

pub type BoxedNoteStore<T> = Box<dyn NoteStore<T> + Sync + Send>;
