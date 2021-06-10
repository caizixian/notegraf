use crate::note::{Note, NoteID, Revision};
use crate::notetype::NoteType;
use std::path::Path;

mod in_memory;
pub use in_memory::InMemoryStore;

pub trait NoteStore<T: NoteType> {
    type Error: Sized;

    fn new_note(&mut self, note_inner: T) -> Result<(NoteID, Revision), Self::Error>;
    fn get_note(&self, id: &NoteID, revision: Option<&Revision>) -> Result<Note<T>, Self::Error>;
    fn update_note(
        &mut self,
        id: &NoteID,
        base_revision: &Revision,
        note_inner: T,
    ) -> Result<Revision, Self::Error>;
    fn get_current_revision(&self, id: &NoteID) -> Result<Revision, Self::Error>;
    fn get_revisions(&self, id: &NoteID) -> Result<Vec<Revision>, Self::Error>;
    fn split_note<F>(&mut self, note: Note<T>, op: F) -> Result<NoteID, Self::Error>
    where
        F: FnOnce(T) -> (T, T);
    fn merge_note<F>(
        &mut self,
        note1: Note<T>,
        note2: Note<T>,
        op: F,
    ) -> Result<NoteID, Self::Error>
    where
        F: FnOnce(T, T) -> T;
    fn get_children(&self, id: NoteID) -> Result<Vec<NoteID>, Self::Error>;
    fn get_references(&self, id: NoteID) -> Result<Vec<NoteID>, Self::Error>;
    fn backup<P: AsRef<Path>>(&self, path: P) -> Result<(), Self::Error>;
    fn restore<P: AsRef<Path>>(path: P) -> Result<Self, Self::Error>
    where
        Self: Sized;
}
