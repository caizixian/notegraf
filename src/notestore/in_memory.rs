use crate::{Note, NoteID, NoteStore, NoteType, Revision};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum InMemoryStoreError {
    #[error("note `{0}` doesn't exist")]
    NoteNotExist(NoteID),
    #[error("note `{0}` already exists")]
    NoteIDConflict(NoteID),
    #[error("revision`{1}` of note `{0}` doesn't exist")]
    RevisionNotExist(NoteID, Revision),
    #[error("io error")]
    IOError(#[from] std::io::Error),
    #[error("serde error")]
    SerdeError(#[from] serde_json::Error),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InMemoryStore<T> {
    notes: HashMap<NoteID, HashMap<Revision, Note<T>>>,
    current_revision: HashMap<NoteID, Revision>,
}

impl<T> InMemoryStore<T>
where
    T: NoteType,
{
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

impl<T: NoteType> Default for InMemoryStore<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: NoteType> NoteStore<T> for InMemoryStore<T> {
    type Error = InMemoryStoreError;

    fn new_note(&mut self, note_inner: T) -> Result<(NoteID, Revision), Self::Error> {
        let id = self.get_noteid();
        let revision = self.get_revision();
        let note = Note::new(note_inner, id.clone(), revision.clone());
        assert!(!self.notes.contains_key(&id));
        self.notes.insert(id.clone(), HashMap::new());
        // unwrap won't fail because we just inserted an entry
        self.notes
            .get_mut(&id)
            .unwrap()
            .insert(revision.clone(), note);
        assert!(!self.current_revision.contains_key(&id));
        self.current_revision.insert(id.clone(), revision.clone());
        Ok((id, revision))
    }

    fn get_note(&self, id: NoteID, revision: Option<Revision>) -> Result<Note<T>, Self::Error> {
        let r: Revision = match revision {
            Some(v) => v,
            None => self
                .current_revision
                // Option<Revision>
                .get(&id)
                // Result<Revision, Error>
                .ok_or_else(|| InMemoryStoreError::NoteNotExist(id.clone()))?
                .clone(),
        };
        Ok(self
            .notes
            .get(&id)
            .ok_or_else(|| InMemoryStoreError::NoteNotExist(id.clone()))?
            .get(&r)
            .ok_or_else(|| InMemoryStoreError::RevisionNotExist(id.clone(), r.clone()))?
            .clone())
    }

    fn get_current_revision(&self, id: NoteID) -> Result<Revision, Self::Error> {
        self.current_revision
            .get(&id)
            .ok_or_else(|| InMemoryStoreError::NoteNotExist(id.clone()))
            .map(|x| x.clone())
    }

    fn update_note(&mut self, _note: Note<T>) -> Result<Revision, Self::Error> {
        todo!()
    }
    fn get_revisions(&self, _id: NoteID) -> Result<Vec<Revision>, Self::Error> {
        todo!()
    }
    fn split_note<F>(&mut self, _note: Note<T>, _op: F) -> Result<NoteID, Self::Error>
    where
        F: FnOnce(T) -> (T, T),
    {
        todo!()
    }
    fn merge_note<F>(
        &mut self,
        _note1: Note<T>,
        _note2: Note<T>,
        _op: F,
    ) -> Result<NoteID, Self::Error>
    where
        F: FnOnce(T, T) -> T,
    {
        todo!()
    }
    fn get_children(&self, _id: NoteID) -> Result<Vec<NoteID>, Self::Error> {
        todo!()
    }
    fn get_references(&self, _id: NoteID) -> Result<Vec<NoteID>, Self::Error> {
        todo!()
    }
    fn backup<P: AsRef<Path>>(&self, path: P) -> Result<(), Self::Error> {
        let p = path.as_ref().join("notegraf_in_memory.json");

        let mut f = File::create(p).map_err(InMemoryStoreError::IOError)?;
        f.write_all(&serde_json::to_vec(&self).map_err(InMemoryStoreError::SerdeError)?)
            .map_err(InMemoryStoreError::IOError)?;
        Ok(())
    }
    fn restore<P: AsRef<Path>>(_path: P) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
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
        let (id1, _) = store.new_note(PlainNote::new("Foo".into())).unwrap();
        let (id2, _) = store.new_note(PlainNote::new("Bar".into())).unwrap();
        assert_ne!(id1, id2);
    }

    #[test]
    fn new_note_revision() {
        let mut store: InMemoryStore<PlainNote> = InMemoryStore::new();
        let (id1, revision1) = store.new_note(PlainNote::new("Foo".into())).unwrap();
        assert_eq!(store.get_current_revision(id1).unwrap(), revision1);
    }

    #[test]
    fn new_note_retrieve() {
        let mut store: InMemoryStore<PlainNote> = InMemoryStore::new();
        let note_inner = PlainNote::new("Foo".into());
        let (id1, revision1) = store.new_note(note_inner.clone()).unwrap();
        assert_eq!(
            store.get_note(id1.clone(), None).unwrap().note_inner,
            note_inner
        );
        assert_eq!(
            store.get_note(id1, Some(revision1)).unwrap().note_inner,
            note_inner
        );
    }
}
