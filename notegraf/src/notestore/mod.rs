//! Storage backends of notes.
use crate::errors::NoteStoreError;
use crate::note::*;
use crate::notemetadata::NoteMetadataEditable;
use crate::notetype::NoteType;
use futures::future::BoxFuture;
use std::path::Path;

mod in_memory;
mod postgresql;
pub mod search;
#[cfg(test)]
mod tests;
pub mod util;

use crate::notestore::search::SearchRequest;
pub use in_memory::InMemoryStore;
pub use postgresql::{PostgreSQLStore, PostgreSQLStoreBuilder};

pub type Revisions<T> = Vec<Box<dyn Note<T>>>;

/// An abstraction for storage backends.
///
/// A [`NoteStore`] can cache the parent-children or reference relationships for performance
/// reasons, e.g., in the form of a SQL table or in a graph database.
pub trait NoteStore<T>
where
    T: NoteType,
{
    /// Create a new note.
    ///
    /// The storage backend assigns a [`NoteID`] and [`Revision`]
    fn new_note(
        &self,
        title: String,
        note_inner: T,
        metadata: NoteMetadataEditable,
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
    /// There is no separate operations for rolling back a note to a specific revision.
    /// The idea is that you can just copy that revision and update the current revision to
    /// that one.
    /// Similarly, you can resurrect a deleted note by updating the note.
    ///
    /// When resurrecting a note, it will become a standalone note and losing any
    /// parent-children or previous-next relationship.
    ///
    /// If a [`NoteStore`] caches the reference-referent relationships,
    /// it should check whether any of the relevant fields of note_inner is changed,
    /// and update the cache accordingly.
    fn update_note<'a>(
        &'a self,
        loc: &'a NoteLocator,
        title: Option<String>,
        note_inner: Option<T>,
        note_metadata: NoteMetadataEditable,
    ) -> BoxFuture<'a, Result<NoteLocator, NoteStoreError>>;
    /// Delete a note.
    ///
    /// If a revision is specified, that revision should be the current revision.
    /// This can be used to prevent racy updates to the same note.
    ///
    /// Deleting a note only marks the current revision as empty.
    /// It is not truly deleted. You can still fetch the note if you know the revision.
    ///
    /// When deleting a note in the middle of a sequence, the next note of the note to be deleted
    /// will become the next note of the parent of the note to be deleted.
    /// It is implementation defined whether the previous note or the next note or both of the note
    /// to be deleted is updated.
    ///
    /// If the note to be deleted has a parent and has a next, then the next note will become
    /// a child of the parent.
    ///
    /// Need to check for dangling references.
    /// A note cannot be deleted if it is the referent of any other note.
    fn delete_note<'a>(&'a self, loc: &'a NoteLocator)
        -> BoxFuture<'a, Result<(), NoteStoreError>>;
    /// Get all revisions of a note, in the order from older (smaller timestamp) to newer (larger
    /// timestamp).
    ///
    /// No matter which variant of [`NoteLocator`] is used, we only care about the [`NoteID`].
    fn get_revisions<'a>(
        &'a self,
        loc: &'a NoteLocator,
    ) -> BoxFuture<'a, Result<Revisions<T>, NoteStoreError>>;
    /// Get the current revision of a note.
    ///
    /// No matter which variant of [`NoteLocator`] is used, we only care about the [`NoteID`].
    fn get_current_revision<'a>(
        &'a self,
        loc: &'a NoteLocator,
    ) -> BoxFuture<'a, Result<Option<Revision>, NoteStoreError>>;
    /// Append a note to the last (or only) note in a sequence
    ///
    /// It is implementation defined whether the previous note is updated.
    /// The only guarantee is that after the operation, the latest revision of the previous note
    /// will have the just added note as the next note (either because the previous note was updated
    /// or it was done during the previous-next relationship computation).
    fn append_note<'a>(
        &'a self,
        last: &'a NoteID,
        title: String,
        note_inner: T,
        metadata: NoteMetadataEditable,
    ) -> BoxFuture<'a, Result<NoteLocator, NoteStoreError>>;
    /// Add a branch to a note
    ///
    /// It is implementation defined whether the parent note is updated.
    /// The only guarantee is that after the operation, the latest revision of the parent note
    /// will have the just added note as a child (either because the parent note was updated or
    /// it was done during the parent-children relationship computation).
    fn add_branch<'a>(
        &'a self,
        parent: &'a NoteID,
        title: String,
        note_inner: T,
        metadata: NoteMetadataEditable,
    ) -> BoxFuture<'a, Result<NoteLocator, NoteStoreError>>;
    /// Search for a note
    fn search<'a>(
        &'a self,
        sr: &'a SearchRequest,
    ) -> BoxFuture<'a, Result<Revisions<T>, NoteStoreError>>;
    /// List all known tags
    fn tags(&self) -> BoxFuture<Result<Vec<String>, NoteStoreError>>;
    /// Backup the storage to a folder on some filesystem.
    fn backup(&self, path: Box<dyn AsRef<Path> + Send>) -> BoxFuture<Result<(), NoteStoreError>>;
    /// Restore the storage from a folder on some filesystem.
    fn restore<P: AsRef<Path>>(path: P) -> Result<Self, NoteStoreError>
    where
        Self: Sized;
}

pub type BoxedNoteStore<T> = Box<dyn NoteStore<T> + Sync + Send>;
