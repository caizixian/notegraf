//! Storage backends of notes.
use crate::note::*;
use crate::notetype::NoteType;
use std::path::Path;

mod in_memory;
pub use in_memory::InMemoryStore;

/// An abstraction for storage backends.
pub trait NoteStore<T: NoteType> {
    type Error: Sized;

    /// Create a new note.
    ///
    /// The storage backend assigns a [`NoteID`] and [`Revision`]
    ///
    /// A [`NoteStore`] can cache the parent-children or reference relationships for performance
    /// reasons, e.g., in the form of a SQL table or in a graph database.
    fn new_note(&mut self, note_inner: T) -> Result<NoteLocator, Self::Error>;
    /// Get a note.
    ///
    /// Using different variants of the [`NoteLocator`], one can get a specific revision or
    /// the current revision.
    fn get_note(&self, loc: &NoteLocator) -> Result<Note<T>, Self::Error>;
    /// Update a note.
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
    fn update_note(&mut self, loc: &NoteLocator, note_inner: T)
        -> Result<NoteLocator, Self::Error>;
    /// Delete a note.
    ///
    /// If a revision is specified, that revision should be the current revision.
    /// This can be used to prevent racy updates to the same note.
    ///
    /// Deleting a note only marks the current revision as empty.
    /// It is not truly deleted. You can still fetch the note if you know the revision.
    /// Note that the parent-child relationship between notes is revisioned.
    /// Therefore, we need to update the parent note of the deleted note.
    fn delete_note(&mut self, loc: &NoteLocator) -> Result<(), Self::Error>;
    /// Get the current revision of a note.
    ///
    /// No matter which variant of [`NoteLocator`] is used, we only care about the [`NoteID`].
    fn get_current_revision(&self, loc: &NoteLocator) -> Result<&Revision, Self::Error>;
    /// Get all revisions of a note.
    ///
    /// No matter which variant of [`NoteLocator`] is used, we only care about the [`NoteID`].
    fn get_revisions(&self, loc: &NoteLocator) -> Result<Vec<Revision>, Self::Error>;
    /// Split a note into two parts.
    ///
    /// The second part becomes the children of the first part.
    /// The first part is updated in-place, i.e., same [`NoteID`] but a different [`Revision`].
    ///
    /// The closure argument specifies how the inner note can be split.
    ///
    /// If a revision is specified, that revision should be the current revision.
    /// This can be used to prevent racy updates to the same note.
    ///
    /// Note that this function can also be used to create a child note without modifying the
    /// parent.
    fn split_note<F>(&mut self, note: &NoteLocator, op: F) -> Result<NoteLocator, Self::Error>
    where
        F: FnOnce(T) -> (T, T);
    /// Merge two notes.
    ///
    /// The second note must the a child of the first note.
    ///
    /// There is the no guarantee whether the notes will be merged in-place.
    /// The only invariant is that the children of the first and the second notes will become
    /// the children of the new note.
    ///
    /// The closure argument specifies how two inner notes can be joined.
    ///
    /// If a revision is specified, that revision should be the current revision.
    /// This can be used to prevent racy updates to the same note.
    fn merge_note<F>(
        &mut self,
        note1: &NoteLocator,
        note2: &NoteLocator,
        op: F,
    ) -> Result<NoteLocator, Self::Error>
    where
        F: FnOnce(T, T) -> T;
    /// Backup the storage to a folder on some filesystem.
    fn backup<P: AsRef<Path>>(&self, path: P) -> Result<(), Self::Error>;
    /// Restore the storage from a folder on some filesystem.
    fn restore<P: AsRef<Path>>(path: P) -> Result<Self, Self::Error>
    where
        Self: Sized;
}
