use crate::{Note, NoteID, NoteStore, NoteType, Revision};
use std::collections::HashMap;
use std::path::Path;
use uuid::Uuid;

#[derive(Debug)]
pub struct InMemoryStore<T: NoteType> {
    notes: HashMap<NoteID, HashMap<Revision, Note<T>>>,
    current_revision: HashMap<NoteID, Revision>,
}

impl<T: NoteType> InMemoryStore<T> {
    pub fn new() -> Self {
        InMemoryStore {
            notes: Default::default(),
            current_revision: Default::default(),
        }
    }

    fn get_noteid(&self) -> NoteID {
        NoteID::new(Uuid::new_v4().to_hyphenated().to_string())
    }

    fn get_revision(&self) -> Revision {
        Revision::new(Uuid::new_v4().to_hyphenated().to_string())
    }
}

impl<T: NoteType> NoteStore<T> for InMemoryStore<T> {
    fn new_note(&mut self, note_inner: T) -> (NoteID, Revision) {
        let id = self.get_noteid();
        let revision = self.get_revision();
        let note = Note::new(note_inner, id.clone(), revision.clone());
        (*self.notes.entry(id.clone()).or_insert(HashMap::new())).insert(revision.clone(), note);
        self.current_revision.insert(id.clone(), revision.clone());
        (id, revision)
    }

    fn get_note(&self, id: NoteID, revision: Option<Revision>) -> Note<T> {
        let r = revision.unwrap_or(self.current_revision.get(&id).unwrap().clone());
        self.notes.get(&id).unwrap().get(&r).unwrap().clone()
    }

    fn update_note(&mut self, _note: Note<T>) -> Revision {
        todo!()
    }

    fn get_current_revision(&self, id: NoteID) -> Revision {
        self.current_revision.get(&id).unwrap().clone()
    }

    fn get_revisions(&self, _id: NoteID) -> Vec<Revision> {
        todo!()
    }

    fn split_note<F>(&mut self, _note: Note<T>, _op: F) -> NoteID
    where
        F: FnOnce(T) -> (T, T),
    {
        todo!()
    }

    fn merge_note<F>(&mut self, _note1: Note<T>, _note2: Note<T>, _op: F) -> NoteID
    where
        F: FnOnce(T, T) -> T,
    {
        todo!()
    }

    fn get_children(&self, _id: NoteID) -> Vec<NoteID> {
        todo!()
    }

    fn get_references(&self, _id: NoteID) -> Vec<NoteID> {
        todo!()
    }

    fn backup<P: AsRef<Path>>(&self, _path: P) {
        todo!()
    }
    fn restore<P: AsRef<Path>>(_path: P) -> Self {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notetype::PlainNote;
    #[test]
    fn unique_id() {
        let mut store: InMemoryStore<PlainNote> = InMemoryStore::new();
        let (id1, _) = store.new_note(PlainNote::new("Foo".into()));
        let (id2, _) = store.new_note(PlainNote::new("Bar".into()));
        assert_ne!(id1, id2);
    }

    #[test]
    fn new_note_revision() {
        let mut store: InMemoryStore<PlainNote> = InMemoryStore::new();
        let (id1, revision1) = store.new_note(PlainNote::new("Foo".into()));
        assert_eq!(store.get_current_revision(id1), revision1);
    }

    #[test]
    fn new_note_retrieve() {
        let mut store: InMemoryStore<PlainNote> = InMemoryStore::new();
        let note_inner = PlainNote::new("Foo".into());
        let (id1, revision1) = store.new_note(note_inner.clone());
        assert_eq!(store.get_note(id1.clone(), None).note_inner, note_inner);
        assert_eq!(store.get_note(id1, Some(revision1)).note_inner, note_inner);
    }
}
