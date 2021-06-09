use std::collections::HashMap;
use crate::note::{NoteID, Note};
use crate::notetype::NoteType;
use uuid::Uuid;

trait NoteStore<T: NoteType> {
    fn new_note(&mut self, note_inner: T) -> NoteID;
}

#[derive(Debug)]
struct InMemoryStore<T: NoteType> {
    notes: HashMap<NoteID, Note<T>>
}

impl<T: NoteType> InMemoryStore<T> {
    fn get_noteid(&self) -> NoteID {
        Uuid::new_v4().to_hyphenated().to_string()
    }
}

impl<T: NoteType> NoteStore<T> for InMemoryStore<T> {
    fn new_note(&mut self, note_inner: T) ->  NoteID {
        let id = self.get_noteid();
        self.notes.insert(id.clone(), Note::new(note_inner, id.clone()));
        id
    }
}
