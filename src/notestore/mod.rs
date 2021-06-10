use crate::note::{Note, NoteID, Revision};
use crate::notetype::NoteType;
use std::path::Path;

mod in_memory;
pub use in_memory::InMemoryStore;

pub trait NoteStore<T: NoteType> {
    fn new_note(&mut self, note_inner: T) -> (NoteID, Revision);
    fn get_note(&self, id: NoteID, revision: Option<Revision>) -> Note<T>;
    fn update_note(&mut self, note: Note<T>) -> Revision;
    fn get_current_revision(&self, id: NoteID) -> Revision;
    fn get_revisions(&self, id: NoteID) -> Vec<Revision>;
    fn split_note<F>(&mut self, note: Note<T>, op: F) -> NoteID
    where
        F: FnOnce(T) -> (T, T);
    fn merge_note<F>(&mut self, note1: Note<T>, note2: Note<T>, op: F) -> NoteID
    where
        F: FnOnce(T, T) -> T;
    fn get_children(&self, id: NoteID) -> Vec<NoteID>;
    fn get_references(&self, id: NoteID) -> Vec<NoteID>;
    fn backup<P: AsRef<Path>>(&self, path: P);
    fn restore<P: AsRef<Path>>(path: P) -> Self;
}
