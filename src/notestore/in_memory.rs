use crate::{Note, NoteID, NoteStore, NoteType, Revision};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::time::SystemTime;
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
    #[error("attempt to update non-current revision `{1}` of note `{0}`")]
    UpdateOldRevision(NoteID, Revision),
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

    fn get_new_noteid(&self) -> NoteID {
        NoteID::new(Uuid::new_v4().to_hyphenated().to_string())
    }

    fn get_new_revision(&self) -> Revision {
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
        let id = self.get_new_noteid();
        let revision = self.get_new_revision();
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

    fn get_note(&self, id: &NoteID, revision: Option<&Revision>) -> Result<Note<T>, Self::Error> {
        let r: &Revision = match revision {
            Some(v) => v,
            None => self
                .current_revision
                // Option<Revision>
                .get(&id)
                // Result<Revision, Error>
                .ok_or_else(|| InMemoryStoreError::NoteNotExist(id.clone()))?,
        };
        Ok(self
            .notes
            .get(id)
            .ok_or_else(|| InMemoryStoreError::NoteNotExist(id.clone()))?
            .get(r)
            .ok_or_else(|| InMemoryStoreError::RevisionNotExist(id.clone(), r.clone()))?
            .clone())
    }

    fn get_current_revision(&self, id: &NoteID) -> Result<Revision, Self::Error> {
        self.current_revision
            .get(id)
            .ok_or_else(|| InMemoryStoreError::NoteNotExist(id.clone()))
            .map(|x| x.clone())
    }

    fn update_note(
        &mut self,
        id: &NoteID,
        base_revision: &Revision,
        note_inner: T,
    ) -> Result<Revision, Self::Error> {
        let current_note = self.get_note(id, None)?;
        if &current_note.revision != base_revision {
            return Err(InMemoryStoreError::UpdateOldRevision(
                id.clone(),
                base_revision.clone(),
            ));
        }
        // get new revision number
        let new_revision = self.get_new_revision();
        let note_revisions = self.notes.get_mut(id).unwrap();
        // sanity check
        assert!(!note_revisions.contains_key(&new_revision));
        // update note
        let updated_note = Note {
            note_inner,
            revision: new_revision.clone(),
            modified_at: SystemTime::now(),
            ..current_note
        };
        note_revisions.insert(new_revision.clone(), updated_note);
        // update current revision number
        *self.current_revision.get_mut(id).unwrap() = new_revision.clone();
        Ok(new_revision)
    }

    fn get_revisions(&self, id: &NoteID) -> Result<Vec<Revision>, Self::Error> {
        self.notes
            .get(id)
            .ok_or_else(|| InMemoryStoreError::NoteNotExist(id.clone()))
            .map(|rs| rs.keys().cloned().collect())
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
    fn restore<P: AsRef<Path>>(path: P) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let p = path.as_ref().join("notegraf_in_memory.json");
        let contents = fs::read_to_string(p).map_err(InMemoryStoreError::IOError)?;
        serde_json::from_str(&contents).map_err(InMemoryStoreError::SerdeError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notetype::PlainNote;
    use std::env;
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
        assert_eq!(store.get_current_revision(&id1).unwrap(), revision1);
    }

    #[test]
    fn new_note_retrieve() {
        let mut store: InMemoryStore<PlainNote> = InMemoryStore::new();
        let note_inner = PlainNote::new("Foo".into());
        let (id1, revision1) = store.new_note(note_inner.clone()).unwrap();
        assert_eq!(store.get_note(&id1, None).unwrap().note_inner, note_inner);
        assert_eq!(
            store.get_note(&id1, Some(&revision1)).unwrap().note_inner,
            note_inner
        );
    }

    #[test]
    fn backup() {
        let mut store: InMemoryStore<PlainNote> = InMemoryStore::new();
        let (id1, _) = store.new_note(PlainNote::new("Foo".into())).unwrap();
        let (id2, _) = store.new_note(PlainNote::new("Bar".into())).unwrap();

        store.backup(env::temp_dir()).unwrap();
        let store_restore: InMemoryStore<PlainNote> =
            InMemoryStore::restore(env::temp_dir()).unwrap();
        for id in vec![id1, id2].iter() {
            let note = store.get_note(&id, None).unwrap();
            let note_restore = store_restore.get_note(&id, None).unwrap();
            assert_eq!(note, note_restore);
        }
    }

    #[test]
    fn update_note() {
        let mut store: InMemoryStore<PlainNote> = InMemoryStore::new();
        let (id1, rev1) = store.new_note(PlainNote::new("Foo".into())).unwrap();
        let created1 = store.get_note(&id1, None).unwrap().created_at;
        let modified1 = store.get_note(&id1, None).unwrap().modified_at;
        let rev2 = store
            .update_note(&id1, &rev1, PlainNote::new("Foo1".into()))
            .unwrap();
        assert_ne!(store.get_current_revision(&id1).unwrap(), rev1);
        assert_eq!(store.get_current_revision(&id1).unwrap(), rev2);
        assert_eq!(
            store.get_note(&id1, None).unwrap().note_inner,
            PlainNote::new("Foo1".into())
        );
        assert_eq!(
            store.get_note(&id1, Some(&rev2)).unwrap().note_inner,
            PlainNote::new("Foo1".into())
        );
        assert_ne!(
            store.get_note(&id1, Some(&rev2)).unwrap().modified_at,
            modified1
        );
        assert_eq!(
            store.get_note(&id1, Some(&rev2)).unwrap().created_at,
            created1
        );
    }
}
