//! In-memory storage of notes
use crate::note::NoteLocator;
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
    #[error("note `{0}` is deleted, revision needed if resurrecting a deleted note")]
    NoteDeleted(NoteID),
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
    #[error("attempt to delete non-current revision `{1}` of note `{0}`")]
    DeleteOldRevision(NoteID, Revision),
    #[error("inconsistency detected: note `{1}` is not a child of note `{0}`")]
    NotAChild(NoteID, NoteID),
}

/// In-memory storage.
///
/// This is mostly designed for development use, because there is no persistence layer.
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

    /// Generate a new [`NoteID`].
    ///
    /// We use the UUID V4 scheme.
    fn get_new_noteid(&self) -> NoteID {
        NoteID::new(Uuid::new_v4().to_hyphenated().to_string())
    }

    /// Generate a new [`Revision`].
    ///
    /// We use the UUID V4 scheme.
    fn get_new_revision(&self) -> Revision {
        Revision::new(Uuid::new_v4().to_hyphenated().to_string())
    }

    /// Does the locator points to a current revision
    fn is_current(&self, loc: &NoteLocator) -> Result<bool, InMemoryStoreError> {
        if let Some(r) = loc.get_revision() {
            // If the argument is a specific revision, then compare it with the current revision
            let current_rev = self.get_current_revision(loc)?;
            Ok(current_rev == r)
        } else {
            // Otherwise, it's current as long as the note is not deleted
            Ok(!self.is_deleted(loc)?)
        }
    }

    /// Does the locator points to a revision of deleted note
    fn is_deleted(&self, loc: &NoteLocator) -> Result<bool, InMemoryStoreError> {
        // A note is deleted if it has revisions but not a current revision
        let id = loc.get_id();
        if self.notes.contains_key(id) {
            if self.current_revision.contains_key(id) {
                Ok(false)
            } else {
                Ok(true)
            }
        } else {
            Err(InMemoryStoreError::NoteNotExist(id.clone()))
        }
    }

    /// Update a note, whose content is possibly updated in the process
    ///
    /// Might resurrect a deleted note, as long as the locator points to a valid revision
    ///
    /// The set of children doesn't need to be explicitly changed.
    /// Instead, this set will be maintained to be consistent when the parent is changed.
    fn update_note<F>(
        &mut self,
        loc: &NoteLocator,
        op: F,
    ) -> Result<NoteLocator, InMemoryStoreError>
    where
        F: FnOnce(&Note<T>) -> Result<Note<T>, InMemoryStoreError>,
    {
        let (id, rev) = loc.unpack();
        let old_note = if self.is_deleted(loc)? || self.is_current(loc)? {
            self.get_note(loc)?
        } else {
            return Err(InMemoryStoreError::UpdateOldRevision(
                id.clone(),
                rev.unwrap().clone(),
            ));
        };
        // get new revision number
        let new_revision = self.get_new_revision();
        let note_revisions = self
            .notes
            .get_mut(id)
            .ok_or_else(|| InMemoryStoreError::NoteNotExist(id.clone()))?;
        assert!(!note_revisions.contains_key(&new_revision)); // sanity check
                                                              // update note
        let mut updated_note = op(&old_note)?;
        updated_note.revision = new_revision.clone();
        updated_note.modified_at = SystemTime::now();
        updated_note.created_at = old_note.created_at;
        // don't need to borrow updated_note for the below change
        let new_parent = updated_note.parent.clone();
        note_revisions.insert(new_revision.clone(), updated_note);
        self.current_revision
            .insert(id.clone(), new_revision.clone());
        // propagate changes in parent-children relationships
        if new_parent != old_note.parent {
            if let Some(ref p) = old_note.parent {
                self.remove_child(&NoteLocator::Current(p.clone()), id)
                    .unwrap();
            }
            if let Some(ref p) = new_parent {
                self.add_child(&NoteLocator::Current(p.clone()), id)
                    .unwrap();
            }
        }
        Ok(NoteLocator::Specific(id.clone(), new_revision))
    }

    /// Add a child from a note
    fn add_child(
        &mut self,
        loc: &NoteLocator,
        child: &NoteID,
    ) -> Result<NoteLocator, InMemoryStoreError> {
        self.update_note(loc, |old_note| {
            let mut note = old_note.clone();
            note.children.insert(child.clone());
            Ok(note)
        })
    }

    /// Remove a child from a note
    fn remove_child(
        &mut self,
        loc: &NoteLocator,
        child: &NoteID,
    ) -> Result<NoteLocator, InMemoryStoreError> {
        self.update_note(loc, |old_note| {
            let mut note = old_note.clone();
            let success = note.children.remove(child);
            if success {
                Ok(note)
            } else {
                Err(InMemoryStoreError::NotAChild(
                    loc.get_id().clone(),
                    child.clone(),
                ))
            }
        })
    }

    /// Get all revisions of a note with actual notes
    pub fn get_revisions_with_note(
        &self,
        loc: &NoteLocator,
    ) -> Result<Vec<(Revision, Note<T>)>, InMemoryStoreError> {
        let id = loc.get_id();
        self.notes
            .get(id)
            .ok_or_else(|| InMemoryStoreError::NoteNotExist(id.clone()))
            .map(|rs| {
                let mut v: Vec<(Revision, Note<T>)> =
                    rs.iter().map(|(r, n)| (r.clone(), n.clone())).collect();
                v.sort_by_key(|(_, n)| n.modified_at);
                v
            })
    }
}

impl<T: NoteType> Default for InMemoryStore<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: NoteType> NoteStore<T> for InMemoryStore<T> {
    type Error = InMemoryStoreError;

    fn new_note(&mut self, note_inner: T) -> Result<NoteLocator, Self::Error> {
        let id = self.get_new_noteid();
        let revision = self.get_new_revision();
        let note = Note::new(note_inner, id.clone(), revision.clone(), None);
        assert!(!self.notes.contains_key(&id));
        self.notes.insert(id.clone(), HashMap::new());
        // unwrap won't fail because we just inserted an entry
        self.notes
            .get_mut(&id)
            .unwrap()
            .insert(revision.clone(), note);
        assert!(!self.current_revision.contains_key(&id));
        self.current_revision.insert(id.clone(), revision.clone());
        Ok(NoteLocator::Specific(id, revision))
    }

    fn get_note(&self, loc: &NoteLocator) -> Result<Note<T>, Self::Error> {
        let (id, rev) = loc.unpack();
        let rev = if let Some(r) = rev {
            r
        } else {
            self.get_current_revision(loc)?
        };
        Ok(self
            .notes
            .get(id)
            .ok_or_else(|| InMemoryStoreError::NoteNotExist(id.clone()))?
            .get(rev)
            .ok_or_else(|| InMemoryStoreError::RevisionNotExist(id.clone(), rev.clone()))?
            .clone())
    }

    fn update_note_content(
        &mut self,
        loc: &NoteLocator,
        note_inner: T,
    ) -> Result<NoteLocator, Self::Error> {
        self.update_note(loc, |old_note| {
            let mut note = old_note.clone();
            note.note_inner = note_inner;
            Ok(note)
        })
    }

    fn delete_note(&mut self, loc: &NoteLocator) -> Result<(), Self::Error> {
        let (id, rev) = loc.unpack();
        if self.is_current(loc)? {
            let parent = self.get_note(loc).unwrap().parent;
            self.current_revision.remove(id).unwrap();
            if let Some(p) = parent {
                self.remove_child(&NoteLocator::Current(p), id).unwrap();
            }
            Ok(())
        } else {
            Err(InMemoryStoreError::DeleteOldRevision(
                id.clone(),
                rev.unwrap().clone(),
            ))
        }
    }

    fn get_current_revision(&self, loc: &NoteLocator) -> Result<&Revision, Self::Error> {
        let id = loc.get_id();
        if let Some(r) = self.current_revision.get(id) {
            Ok(r)
        } else if self.notes.contains_key(id) {
            Err(InMemoryStoreError::NoteDeleted(id.clone()))
        } else {
            Err(InMemoryStoreError::NoteNotExist(id.clone()))
        }
    }

    fn get_revisions(&self, loc: &NoteLocator) -> Result<Vec<Revision>, Self::Error> {
        let id = loc.get_id();
        self.notes
            .get(id)
            .ok_or_else(|| InMemoryStoreError::NoteNotExist(id.clone()))
            .map(|rs| rs.keys().cloned().collect())
    }

    fn split_note<F>(&mut self, _note: &NoteLocator, _op: F) -> Result<NoteLocator, Self::Error>
    where
        F: FnOnce(T) -> (T, T),
    {
        todo!()
    }

    fn merge_note<F>(
        &mut self,
        _note1: &NoteLocator,
        _note2: &NoteLocator,
        _op: F,
    ) -> Result<NoteLocator, Self::Error>
    where
        F: FnOnce(T, T) -> T,
    {
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

    fn set_parent<T>(
        store: &mut InMemoryStore<T>,
        loc: &NoteLocator,
        parent: Option<NoteID>,
    ) -> Result<NoteLocator, InMemoryStoreError>
    where
        T: NoteType,
    {
        store.update_note(loc, |old_note| {
            let mut note = old_note.clone();
            note.parent = parent;
            Ok(note)
        })
    }

    #[test]
    fn unique_id() {
        let mut store: InMemoryStore<PlainNote> = InMemoryStore::new();
        let loc1 = store.new_note(PlainNote::new("Foo".into())).unwrap();
        let loc2 = store.new_note(PlainNote::new("Bar".into())).unwrap();
        assert_ne!(loc1.get_id(), loc2.get_id());
    }

    #[test]
    fn new_note_revision() {
        let mut store: InMemoryStore<PlainNote> = InMemoryStore::new();
        let loc = store.new_note(PlainNote::new("Foo".into())).unwrap();
        let rev = loc.get_revision().unwrap();
        assert_eq!(store.get_current_revision(&loc.current()).unwrap(), rev);
    }

    #[test]
    fn new_note_retrieve() {
        let mut store: InMemoryStore<PlainNote> = InMemoryStore::new();
        let note_inner = PlainNote::new("Foo".into());
        let loc = store.new_note(note_inner.clone()).unwrap();
        assert_eq!(
            store.get_note(&loc.current()).unwrap().note_inner,
            note_inner
        );
        assert_eq!(store.get_note(&loc).unwrap().note_inner, note_inner);
    }

    #[test]
    fn backup() {
        let mut store: InMemoryStore<PlainNote> = InMemoryStore::new();
        let loc1 = store.new_note(PlainNote::new("Foo".into())).unwrap();
        let loc2 = store.new_note(PlainNote::new("Bar".into())).unwrap();

        store.backup(env::temp_dir()).unwrap();
        let store_restore: InMemoryStore<PlainNote> =
            InMemoryStore::restore(env::temp_dir()).unwrap();
        for loc in vec![loc1, loc2].iter() {
            let note = store.get_note(loc).unwrap();
            let note_restore = store_restore.get_note(loc).unwrap();
            assert_eq!(note, note_restore);
        }
    }

    #[test]
    fn update_note() {
        let mut store: InMemoryStore<PlainNote> = InMemoryStore::new();
        let loc1 = store.new_note(PlainNote::new("Foo".into())).unwrap();
        let rev1 = loc1.get_revision().unwrap();
        let created1 = store.get_note(&loc1.current()).unwrap().created_at;
        let modified1 = store.get_note(&loc1.current()).unwrap().modified_at;
        let loc2 = store
            .update_note_content(&loc1, PlainNote::new("Foo1".into()))
            .unwrap();
        let rev2 = loc2.get_revision().unwrap();
        assert_ne!(rev1, rev2);
        assert_eq!(store.get_current_revision(&loc1).unwrap(), rev2);
        assert_eq!(
            store.get_note(&loc1.current()).unwrap().note_inner,
            PlainNote::new("Foo1".into())
        );
        assert_eq!(
            store.get_note(&loc1.at_revision(rev2)).unwrap().note_inner,
            PlainNote::new("Foo1".into())
        );
        assert_ne!(
            store.get_note(&loc1.at_revision(rev2)).unwrap().modified_at,
            modified1
        );
        assert_eq!(
            store.get_note(&loc1.at_revision(rev2)).unwrap().created_at,
            created1
        );
    }

    #[test]
    fn add_child() {
        let mut store: InMemoryStore<PlainNote> = InMemoryStore::new();
        let loc1 = store.new_note(PlainNote::new("Child".into())).unwrap();
        let loc2 = store.new_note(PlainNote::new("Parent".into())).unwrap();
        store.add_child(&loc2, loc1.get_id()).unwrap();
        assert!(!store
            .get_note(&loc2) // This points to an old revision
            .unwrap()
            .children
            .contains(loc1.get_id()));
        assert!(store
            .get_note(&loc2.current())
            .unwrap()
            .children
            .contains(loc1.get_id()));
    }

    #[test]
    fn remove_non_existent_child() {
        let mut store: InMemoryStore<PlainNote> = InMemoryStore::new();
        let loc1 = store.new_note(PlainNote::new("Child".into())).unwrap();
        let loc2 = store.new_note(PlainNote::new("Parent".into())).unwrap();
        assert!(matches!(
            store.remove_child(&loc2, loc1.get_id()).err().unwrap(),
            InMemoryStoreError::NotAChild(_, _)
        ));
    }

    #[test]
    fn remove_child() {
        let mut store: InMemoryStore<PlainNote> = InMemoryStore::new();
        let loc1 = store.new_note(PlainNote::new("Child".into())).unwrap();
        let loc2 = store.new_note(PlainNote::new("Parent".into())).unwrap();
        let loc3 = store.add_child(&loc2, loc1.get_id()).unwrap();
        // This points to an old revision
        assert!(matches!(
            store.remove_child(&loc2, loc1.get_id()).err().unwrap(),
            InMemoryStoreError::UpdateOldRevision(_, _)
        ));
        let loc4 = store.remove_child(&loc2.current(), loc1.get_id()).unwrap();
        assert!(!store
            .get_note(&loc2)
            .unwrap()
            .children
            .contains(loc1.get_id()));
        assert!(store
            .get_note(&loc3)
            .unwrap()
            .children
            .contains(loc1.get_id()));
        assert!(!store
            .get_note(&loc4)
            .unwrap()
            .children
            .contains(loc1.get_id()));
        assert!(!store
            .get_note(&loc2.current())
            .unwrap()
            .children
            .contains(loc1.get_id()));
    }

    #[test]
    fn delete_note_specific() {
        let mut store: InMemoryStore<PlainNote> = InMemoryStore::new();
        let loc1 = store.new_note(PlainNote::new("Note".into())).unwrap();
        store.delete_note(&loc1).unwrap();
        assert!(store.is_deleted(&loc1).unwrap());
    }

    #[test]
    fn delete_note_current() {
        let mut store: InMemoryStore<PlainNote> = InMemoryStore::new();
        let loc1 = store.new_note(PlainNote::new("Note".into())).unwrap();
        store.delete_note(&loc1.current()).unwrap();
        assert!(store.is_deleted(&loc1).unwrap());
        assert!(matches!(
            store.is_current(&loc1).err().unwrap(),
            InMemoryStoreError::NoteDeleted(_)
        ));
    }

    #[test]
    fn test_set_parent() {
        let mut store: InMemoryStore<PlainNote> = InMemoryStore::new();
        let loc1 = store.new_note(PlainNote::new("Child".into())).unwrap();
        let loc2 = store.new_note(PlainNote::new("Parent".into())).unwrap();
        set_parent(&mut store, &loc1, Some(loc2.get_id().clone())).unwrap();
        assert!(store
            .get_note(&loc2.current())
            .unwrap()
            .children
            .contains(loc1.get_id()));
    }

    #[test]
    fn delete_note_parent() {
        let mut store: InMemoryStore<PlainNote> = InMemoryStore::new();
        let loc1 = store.new_note(PlainNote::new("Child".into())).unwrap();
        let loc2 = store.new_note(PlainNote::new("Parent".into())).unwrap();
        set_parent(&mut store, &loc1, Some(loc2.get_id().clone())).unwrap();
        assert!(store
            .get_note(&loc2.current())
            .unwrap()
            .children
            .contains(loc1.get_id()));
        store.delete_note(&loc1.current()).unwrap();
        assert!(!store
            .get_note(&loc2.current())
            .unwrap()
            .children
            .contains(loc1.get_id()));
    }

    #[test]
    fn resurrect_deleted_note() {
        let mut store: InMemoryStore<PlainNote> = InMemoryStore::new();
        let loc1 = store.new_note(PlainNote::new("Foo".into())).unwrap();
        let loc2 = store
            .update_note_content(&loc1, PlainNote::new("Foo1".into()))
            .unwrap();
        store.delete_note(&loc1.current()).unwrap();
        let revisions = store.get_revisions_with_note(&loc1).unwrap();
        let (last_revision, last_note) = revisions.last().unwrap();
        assert_eq!(last_note.note_inner, PlainNote::new("Foo1".into()));
        assert_eq!(last_revision, loc2.get_revision().unwrap());
        store
            .update_note_content(
                &NoteLocator::Specific(loc1.get_id().clone(), last_revision.clone()),
                last_note.note_inner.clone(),
            )
            .unwrap();
        assert_eq!(
            store.get_note(&loc1.current()).unwrap().note_inner,
            PlainNote::new("Foo1".into())
        );
    }
}
