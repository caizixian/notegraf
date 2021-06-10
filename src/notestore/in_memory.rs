use crate::{Note, NoteID, NoteStore, NoteType};
use std::collections::HashMap;
use std::path::Path;
use uuid::Uuid;

#[derive(Debug)]
pub struct InMemoryStore<T: NoteType> {
    notes: HashMap<NoteID, Note<T>>,
}

impl<T: NoteType> InMemoryStore<T> {
    pub fn new() -> Self {
        InMemoryStore {
            notes: Default::default(),
        }
    }

    fn get_noteid(&self) -> NoteID {
        Uuid::new_v4().to_hyphenated().to_string()
    }
}

impl<T: NoteType> NoteStore<T> for InMemoryStore<T> {
    fn new_note(&mut self, note_inner: T) -> NoteID {
        let id = self.get_noteid();
        self.notes
            .insert(id.clone(), Note::new(note_inner, id.clone()));
        id
    }

    fn get_note(&self, _id: NoteID) -> Note<T> {
        todo!()
    }

    fn update_note(&mut self, _note: Note<T>) {
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
        let id1 = store.new_note(PlainNote::new("Foo".into()));
        let id2 = store.new_note(PlainNote::new("Bar".into()));
        assert_ne!(id1, id2);
    }
}
