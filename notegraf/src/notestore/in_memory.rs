//! In-memory storage of notes
use crate::errors::NoteStoreError;
use crate::note::NoteLocator;
use crate::notemetadata::NoteMetadata;
use crate::{Note, NoteID, NoteStore, NoteType, Revision};
use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use tokio::sync::RwLock;
use uuid::Uuid;

type RevisionsOfNote<T> = Vec<(Revision, Note<T>)>;

/// In-memory storage.
///
/// This is mostly designed for development use, because there is no persistence layer.
#[derive(Debug, Serialize, Deserialize)]
struct InMemoryStoreInner<T> {
    notes: HashMap<NoteID, HashMap<Revision, Note<T>>>,
    current_revision: HashMap<NoteID, Revision>,
}

impl<T: NoteType> Default for InMemoryStoreInner<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: NoteType> InMemoryStoreInner<T> {
    pub fn new() -> Self {
        InMemoryStoreInner {
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
    fn is_current(&self, loc: &NoteLocator) -> Result<bool, NoteStoreError> {
        if let Some(r) = loc.get_revision() {
            // If the argument is a specific revision, then compare it with the current revision
            let current_rev = self.get_current_revision(loc)?;
            Ok(&current_rev == r)
        } else {
            // Otherwise, it's current as long as the note is not deleted
            Ok(!self.is_deleted(loc)?)
        }
    }

    /// Does the locator points to a revision of deleted note
    fn is_deleted(&self, loc: &NoteLocator) -> Result<bool, NoteStoreError> {
        // A note is deleted if it has revisions but not a current revision
        let id = loc.get_id();
        if self.notes.contains_key(id) {
            if self.current_revision.contains_key(id) {
                Ok(false)
            } else {
                Ok(true)
            }
        } else {
            Err(NoteStoreError::NoteNotExist(id.clone()))
        }
    }

    /// Update a note, whose content is possibly updated in the process
    ///
    /// Might resurrect a deleted note, as long as the locator points to a valid revision
    ///
    /// The set of children doesn't need to be explicitly changed.
    /// Instead, this set will be maintained to be consistent when the parent is changed.
    fn update_note_helper<F>(
        &mut self,
        loc: &NoteLocator,
        op: F,
    ) -> Result<NoteLocator, NoteStoreError>
    where
        F: FnOnce(&Note<T>) -> Result<Note<T>, NoteStoreError>,
    {
        let (id, rev) = loc.unpack();
        let resurrected = self.is_deleted(loc)?;
        let old_note = if resurrected || self.is_current(loc)? {
            self.get_note(loc)?
        } else {
            return Err(NoteStoreError::UpdateOldRevision(
                id.clone(),
                rev.unwrap().clone(),
            ));
        };
        // get new revision number
        let new_revision = self.get_new_revision();
        let note_revisions = self
            .notes
            .get_mut(id)
            .ok_or_else(|| NoteStoreError::NoteNotExist(id.clone()))?;
        // sanity check
        assert!(!note_revisions.contains_key(&new_revision));
        // update note
        let mut updated_note = op(&old_note)?;
        updated_note.revision = new_revision.clone();
        updated_note.metadata = updated_note.metadata.on_update_note();
        // don't need to borrow updated_note for the below change
        let new_parent = updated_note.parent.clone();
        note_revisions.insert(new_revision.clone(), updated_note);
        self.current_revision
            .insert(id.clone(), new_revision.clone());
        // propagate changes in parent-children relationships
        if new_parent != old_note.parent || resurrected {
            if let Some(ref p) = old_note.parent {
                if !resurrected {
                    // The old parent of a resurrected note is like None
                    self.remove_child(&NoteLocator::Current(p.clone()), id)
                        .unwrap();
                }
            }
            if let Some(ref p) = new_parent {
                // TODO check whether p is a descendant of id
                // That is, id transitively reachable by traversing through .parent from p
                self.add_child(&NoteLocator::Current(p.clone()), id)
                    .unwrap();
            }
        }
        Ok(NoteLocator::Specific(id.clone(), new_revision))
    }

    /// Set parent of a note
    fn set_parent(
        &mut self,
        loc: &NoteLocator,
        parent: Option<NoteID>,
    ) -> Result<NoteLocator, NoteStoreError> {
        self.update_note_helper(loc, |old_note| {
            let mut note = old_note.clone();
            note.parent = parent;
            Ok(note)
        })
    }

    /// Add a child from a note
    fn add_child(
        &mut self,
        loc: &NoteLocator,
        child: &NoteID,
    ) -> Result<NoteLocator, NoteStoreError> {
        self.update_note_helper(loc, |old_note| {
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
    ) -> Result<NoteLocator, NoteStoreError> {
        self.update_note_helper(loc, |old_note| {
            let mut note = old_note.clone();
            let success = note.children.remove(child);
            if success {
                Ok(note)
            } else {
                Err(NoteStoreError::NotAChild(
                    loc.get_id().clone(),
                    child.clone(),
                ))
            }
        })
    }

    /// Get all revisions of a note with actual notes
    fn get_revisions_with_note(
        &self,
        loc: &NoteLocator,
    ) -> Result<Vec<(Revision, Note<T>)>, NoteStoreError> {
        let id = loc.get_id();
        self.notes
            .get(id)
            .ok_or_else(|| NoteStoreError::NoteNotExist(id.clone()))
            .map(|rs| {
                let mut v: Vec<(Revision, Note<T>)> =
                    rs.iter().map(|(r, n)| (r.clone(), n.clone())).collect();
                v.sort_by_key(|(_, n)| n.metadata.modified_at);
                v
            })
    }

    fn new_note(&mut self, note_inner: T) -> Result<NoteLocator, NoteStoreError> {
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

    fn get_note_by_revision(&self, id: &NoteID, rev: &Revision) -> Result<Note<T>, NoteStoreError> {
        Ok(self
            .notes
            .get(id)
            .ok_or_else(|| NoteStoreError::NoteNotExist(id.clone()))?
            .get(rev)
            .ok_or_else(|| NoteStoreError::RevisionNotExist(id.clone(), rev.clone()))?
            .clone())
    }

    fn get_note(&self, loc: &NoteLocator) -> Result<Note<T>, NoteStoreError> {
        let (id, rev) = loc.unpack();
        if let Some(r) = rev {
            self.get_note_by_revision(id, r)
        } else {
            self.get_note_by_revision(id, &self.get_current_revision(loc)?)
        }
    }

    fn update_note(
        &mut self,
        loc: &NoteLocator,
        note_inner: Option<T>,
        note_metadata: Option<NoteMetadata>,
    ) -> Result<NoteLocator, NoteStoreError> {
        self.update_note_helper(loc, |old_note| {
            let mut note = old_note.clone();
            if let Some(n) = note_inner {
                note.note_inner = n;
            }
            if let Some(m) = note_metadata {
                note.metadata = m;
            }
            Ok(note)
        })
    }

    fn delete_note(&mut self, loc: &NoteLocator) -> Result<(), NoteStoreError> {
        let (id, rev) = loc.unpack();
        if self.is_current(loc)? {
            let note = self.get_note(loc).unwrap();
            if let Some(p) = &note.parent {
                self.remove_child(&NoteLocator::Current(p.clone()), id)
                    .unwrap();
                for c in &note.children {
                    self.set_parent(&NoteLocator::Current(c.clone()), note.parent.clone())?;
                }
            } else {
                for c in &note.children {
                    self.set_parent(&NoteLocator::Current(c.clone()), None)?;
                }
            }

            // Mark the note as delete at last to avoid the previous steps from referring to
            // a delete note
            self.current_revision.remove(id).unwrap();
            Ok(())
        } else {
            Err(NoteStoreError::DeleteOldRevision(
                id.clone(),
                rev.unwrap().clone(),
            ))
        }
    }

    fn get_current_revision(&self, loc: &NoteLocator) -> Result<Revision, NoteStoreError> {
        let id = loc.get_id();
        if let Some(r) = self.current_revision.get(id) {
            Ok(r.clone())
        } else if self.notes.contains_key(id) {
            Err(NoteStoreError::NoteDeleted(id.clone()))
        } else {
            Err(NoteStoreError::NoteNotExist(id.clone()))
        }
    }

    fn get_revisions(&self, loc: &NoteLocator) -> Result<Vec<Revision>, NoteStoreError> {
        let id = loc.get_id();
        self.notes
            .get(id)
            .ok_or_else(|| NoteStoreError::NoteNotExist(id.clone()))
            .map(|rs| rs.keys().cloned().collect())
    }

    fn split_note<F>(
        &mut self,
        loc: &NoteLocator,
        op: F,
    ) -> Result<(NoteLocator, NoteLocator), NoteStoreError>
    where
        F: FnOnce(T) -> (T, T),
    {
        let note = self.get_note(loc)?;
        let (inner_parent, inner_child) = op(note.note_inner);
        // if loc is not current, the update here will fail, so no need to check
        self.update_note(loc, Some(inner_parent), None)?;
        let loc_child = self.new_note(inner_child)?;
        let loc_child_new = self.set_parent(&loc_child, Some(loc.get_id().clone()))?;
        Ok((loc.current(), loc_child_new))
    }

    fn merge_note<F>(
        &mut self,
        loc1: &NoteLocator,
        loc2: &NoteLocator,
        op: F,
    ) -> Result<NoteLocator, NoteStoreError>
    where
        F: FnOnce(T, T) -> T,
    {
        // Need to check whether both are current for atomicity
        // Otherwise one note might be updated while the other might not
        for loc in &[loc1, loc2] {
            if !self.is_current(loc)? {
                return Err(NoteStoreError::UpdateOldRevision(
                    loc.get_id().clone(),
                    loc.get_revision().unwrap().clone(),
                ));
            }
        }

        let note1 = self.get_note(loc1)?;
        let note2 = self.get_note(loc2)?;
        if note2.parent != Some(note1.id) {
            return Err(NoteStoreError::NotAChild(
                loc1.get_id().clone(),
                loc2.get_id().clone(),
            ));
        }
        let new_inner = op(note1.note_inner, note2.note_inner);
        self.update_note(loc1, Some(new_inner), None)?;
        self.delete_note(loc2)?;
        Ok(loc1.current())
    }

    fn backup<P: AsRef<Path>>(&self, path: P) -> Result<(), NoteStoreError> {
        let p = path.as_ref().join("notegraf_in_memory.json");

        let mut f = File::create(p).map_err(NoteStoreError::IOError)?;
        f.write_all(&serde_json::to_vec(&self).map_err(NoteStoreError::SerdeError)?)
            .map_err(NoteStoreError::IOError)?;
        Ok(())
    }

    fn restore<P: AsRef<Path>>(path: P) -> Result<Self, NoteStoreError>
    where
        Self: Sized,
    {
        let p = path.as_ref().join("notegraf_in_memory.json");
        let contents = fs::read_to_string(p).map_err(NoteStoreError::IOError)?;
        serde_json::from_str(&contents).map_err(NoteStoreError::SerdeError)
    }
}

pub struct InMemoryStore<T> {
    ims: RwLock<InMemoryStoreInner<T>>,
}

impl<T: NoteType> InMemoryStore<T> {
    pub fn new() -> Self {
        InMemoryStore {
            ims: RwLock::new(InMemoryStoreInner::new()),
        }
    }

    pub fn get_revisions_with_note<'a>(
        &'a self,
        loc: &'a NoteLocator,
    ) -> BoxFuture<'a, Result<RevisionsOfNote<T>, NoteStoreError>> {
        Box::pin(async move {
            let ims = self.ims.read().await;
            ims.get_revisions_with_note(loc)
        })
    }
}

impl<T: NoteType> Default for InMemoryStore<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: NoteType> NoteStore<T> for InMemoryStore<T> {
    fn new_note(&self, note_inner: T) -> BoxFuture<Result<NoteLocator, NoteStoreError>> {
        Box::pin(async move {
            let mut ims = self.ims.write().await;
            ims.new_note(note_inner)
        })
    }

    fn get_note<'a>(
        &'a self,
        loc: &'a NoteLocator,
    ) -> BoxFuture<'a, Result<Note<T>, NoteStoreError>> {
        Box::pin(async move {
            let ims = self.ims.read().await;
            ims.get_note(loc)
        })
    }

    fn update_note<'a>(
        &'a self,
        loc: &'a NoteLocator,
        note_inner: Option<T>,
        note_metadata: Option<NoteMetadata>,
    ) -> BoxFuture<'a, Result<NoteLocator, NoteStoreError>> {
        Box::pin(async move {
            let mut ims = self.ims.write().await;
            ims.update_note(loc, note_inner, note_metadata)
        })
    }

    fn delete_note<'a>(
        &'a self,
        loc: &'a NoteLocator,
    ) -> BoxFuture<'a, Result<(), NoteStoreError>> {
        Box::pin(async move {
            let mut ims = self.ims.write().await;
            ims.delete_note(loc)
        })
    }

    fn get_current_revision<'a>(
        &'a self,
        loc: &'a NoteLocator,
    ) -> BoxFuture<'a, Result<Revision, NoteStoreError>> {
        Box::pin(async move {
            let ims = self.ims.read().await;
            ims.get_current_revision(loc)
        })
    }

    fn get_revisions<'a>(
        &'a self,
        loc: &'a NoteLocator,
    ) -> BoxFuture<'a, Result<Vec<Revision>, NoteStoreError>> {
        Box::pin(async move {
            let ims = self.ims.read().await;
            ims.get_revisions(loc)
        })
    }

    fn split_note<'a>(
        &'a self,
        loc: &'a NoteLocator,
        op: Box<dyn FnOnce(T) -> (T, T) + Send>,
    ) -> BoxFuture<'a, Result<(NoteLocator, NoteLocator), NoteStoreError>> {
        Box::pin(async move {
            let mut ims = self.ims.write().await;
            ims.split_note(loc, op)
        })
    }

    fn merge_note<'a>(
        &'a self,
        loc1: &'a NoteLocator,
        loc2: &'a NoteLocator,
        op: Box<dyn FnOnce(T, T) -> T + Send>,
    ) -> BoxFuture<'a, Result<NoteLocator, NoteStoreError>> {
        Box::pin(async move {
            let mut ims = self.ims.write().await;
            ims.merge_note(loc1, loc2, op)
        })
    }

    fn backup(&self, path: Box<dyn AsRef<Path> + Send>) -> BoxFuture<Result<(), NoteStoreError>> {
        Box::pin(async move {
            let ims = self.ims.read().await;
            ims.backup(&*path)
        })
    }

    fn restore<P: AsRef<Path>>(path: P) -> Result<Self, NoteStoreError> {
        Ok(InMemoryStore {
            ims: RwLock::new(InMemoryStoreInner::restore(path)?),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use crate::notetype::PlainNote;

    use super::*;

    #[tokio::test]
    async fn unique_id() {
        let store: InMemoryStore<PlainNote> = InMemoryStore::new();
        let loc1 = store.new_note(PlainNote::new("Foo".into())).await.unwrap();
        let loc2 = store.new_note(PlainNote::new("Bar".into())).await.unwrap();
        assert_ne!(loc1.get_id(), loc2.get_id());
    }

    #[tokio::test]
    async fn new_note_revision() {
        let store: InMemoryStore<PlainNote> = InMemoryStore::new();
        let loc = store.new_note(PlainNote::new("Foo".into())).await.unwrap();
        let rev = loc.get_revision().unwrap();
        assert_eq!(
            &store.get_current_revision(&loc.current()).await.unwrap(),
            rev
        );
    }

    #[tokio::test]
    async fn new_note_retrieve() {
        let store: InMemoryStore<PlainNote> = InMemoryStore::new();
        let note_inner = PlainNote::new("Foo".into());
        let loc = store.new_note(note_inner.clone()).await.unwrap();
        assert_eq!(
            store.get_note(&loc.current()).await.unwrap().note_inner,
            note_inner
        );
        assert_eq!(store.get_note(&loc).await.unwrap().note_inner, note_inner);
    }

    #[tokio::test]
    async fn backup() {
        let store: InMemoryStore<PlainNote> = InMemoryStore::new();
        let loc1 = store.new_note(PlainNote::new("Foo".into())).await.unwrap();
        let loc2 = store.new_note(PlainNote::new("Bar".into())).await.unwrap();

        store.backup(Box::new(env::temp_dir())).await.unwrap();
        let store_restore: InMemoryStore<PlainNote> =
            InMemoryStore::restore(env::temp_dir()).unwrap();
        for loc in vec![loc1, loc2].iter() {
            let note = store.get_note(loc).await.unwrap();
            let note_restore = store_restore.get_note(loc).await.unwrap();
            assert_eq!(note, note_restore);
        }
    }

    #[tokio::test]
    async fn update_note() {
        let store: InMemoryStore<PlainNote> = InMemoryStore::new();
        let loc1 = store.new_note(PlainNote::new("Foo".into())).await.unwrap();
        let rev1 = loc1.get_revision().unwrap();
        let created1 = store
            .get_note(&loc1.current())
            .await
            .unwrap()
            .metadata
            .created_at;
        let modified1 = store
            .get_note(&loc1.current())
            .await
            .unwrap()
            .metadata
            .modified_at;
        let loc2 = store
            .update_note(&loc1, Some(PlainNote::new("Foo1".into())), None)
            .await
            .unwrap();
        let rev2 = loc2.get_revision().unwrap();
        assert_ne!(rev1, rev2);
        assert_eq!(&store.get_current_revision(&loc1).await.unwrap(), rev2);
        assert_eq!(
            store.get_note(&loc1.current()).await.unwrap().note_inner,
            PlainNote::new("Foo1".into())
        );
        assert_eq!(
            store
                .get_note(&loc1.at_revision(rev2))
                .await
                .unwrap()
                .note_inner,
            PlainNote::new("Foo1".into())
        );
        assert_ne!(
            store
                .get_note(&loc1.at_revision(rev2))
                .await
                .unwrap()
                .metadata
                .modified_at,
            modified1
        );
        assert_eq!(
            store
                .get_note(&loc1.at_revision(rev2))
                .await
                .unwrap()
                .metadata
                .created_at,
            created1
        );
    }

    #[tokio::test]
    async fn add_child() {
        let store: InMemoryStore<PlainNote> = InMemoryStore::new();
        let loc1 = store
            .new_note(PlainNote::new("Child".into()))
            .await
            .unwrap();
        let loc2 = store
            .new_note(PlainNote::new("Parent".into()))
            .await
            .unwrap();
        store
            .ims
            .write()
            .await
            .add_child(&loc2, loc1.get_id())
            .unwrap();
        assert!(!store
            .get_note(&loc2) // This points to an old revision
            .await
            .unwrap()
            .children
            .contains(loc1.get_id()));
        assert!(store
            .get_note(&loc2.current())
            .await
            .unwrap()
            .children
            .contains(loc1.get_id()));
    }

    #[tokio::test]
    async fn remove_non_existent_child() {
        let store: InMemoryStore<PlainNote> = InMemoryStore::new();
        let loc1 = store
            .new_note(PlainNote::new("Child".into()))
            .await
            .unwrap();
        let loc2 = store
            .new_note(PlainNote::new("Parent".into()))
            .await
            .unwrap();
        assert!(matches!(
            store
                .ims
                .write()
                .await
                .remove_child(&loc2, loc1.get_id())
                .err()
                .unwrap(),
            NoteStoreError::NotAChild(_, _)
        ));
    }

    #[tokio::test]
    async fn remove_child() {
        let store: InMemoryStore<PlainNote> = InMemoryStore::new();
        let loc1 = store
            .new_note(PlainNote::new("Child".into()))
            .await
            .unwrap();
        let loc2 = store
            .new_note(PlainNote::new("Parent".into()))
            .await
            .unwrap();
        let loc3 = store
            .ims
            .write()
            .await
            .add_child(&loc2, loc1.get_id())
            .unwrap();
        // This points to an old revision
        assert!(matches!(
            store
                .ims
                .write()
                .await
                .remove_child(&loc2, loc1.get_id())
                .err()
                .unwrap(),
            NoteStoreError::UpdateOldRevision(_, _)
        ));
        let loc4 = store
            .ims
            .write()
            .await
            .remove_child(&loc2.current(), loc1.get_id())
            .unwrap();
        assert!(!store
            .get_note(&loc2)
            .await
            .unwrap()
            .children
            .contains(loc1.get_id()));
        assert!(store
            .get_note(&loc3)
            .await
            .unwrap()
            .children
            .contains(loc1.get_id()));
        assert!(!store
            .get_note(&loc4)
            .await
            .unwrap()
            .children
            .contains(loc1.get_id()));
        assert!(!store
            .get_note(&loc2.current())
            .await
            .unwrap()
            .children
            .contains(loc1.get_id()));
    }

    #[tokio::test]
    async fn delete_note_specific() {
        let store: InMemoryStore<PlainNote> = InMemoryStore::new();
        let loc1 = store.new_note(PlainNote::new("Note".into())).await.unwrap();
        store.delete_note(&loc1).await.unwrap();
        assert!(store.ims.read().await.is_deleted(&loc1).unwrap());
    }

    #[tokio::test]
    async fn delete_note_current() {
        let store: InMemoryStore<PlainNote> = InMemoryStore::new();
        let loc1 = store.new_note(PlainNote::new("Note".into())).await.unwrap();
        store.delete_note(&loc1.current()).await.unwrap();
        assert!(store.ims.read().await.is_deleted(&loc1).unwrap());
        assert!(matches!(
            store.ims.read().await.is_current(&loc1).err().unwrap(),
            NoteStoreError::NoteDeleted(_)
        ));
    }

    #[tokio::test]
    async fn test_set_parent() {
        let store: InMemoryStore<PlainNote> = InMemoryStore::new();
        let loc1 = store
            .new_note(PlainNote::new("Child".into()))
            .await
            .unwrap();
        let loc2 = store
            .new_note(PlainNote::new("Parent".into()))
            .await
            .unwrap();
        store
            .ims
            .write()
            .await
            .set_parent(&loc1, Some(loc2.get_id().clone()))
            .unwrap();
        assert!(store
            .get_note(&loc2.current())
            .await
            .unwrap()
            .children
            .contains(loc1.get_id()));
    }

    #[tokio::test]
    async fn delete_note_update_parent() {
        let store: InMemoryStore<PlainNote> = InMemoryStore::new();
        let loc1 = store
            .new_note(PlainNote::new("Child".into()))
            .await
            .unwrap();
        let loc2 = store
            .new_note(PlainNote::new("Parent".into()))
            .await
            .unwrap();
        store
            .ims
            .write()
            .await
            .set_parent(&loc1, Some(loc2.get_id().clone()))
            .unwrap();
        assert!(store
            .get_note(&loc2.current())
            .await
            .unwrap()
            .children
            .contains(loc1.get_id()));
        store.delete_note(&loc1.current()).await.unwrap();
        assert!(!store
            .get_note(&loc2.current())
            .await
            .unwrap()
            .children
            .contains(loc1.get_id()));
    }

    #[tokio::test]
    async fn delete_note_update_child() {
        let store: InMemoryStore<PlainNote> = InMemoryStore::new();
        let loc1 = store
            .new_note(PlainNote::new("Child".into()))
            .await
            .unwrap();
        let loc2 = store
            .new_note(PlainNote::new("Parent".into()))
            .await
            .unwrap();
        store
            .ims
            .write()
            .await
            .set_parent(&loc1, Some(loc2.get_id().clone()))
            .unwrap();
        assert_eq!(
            &store
                .get_note(&loc1.current())
                .await
                .unwrap()
                .parent
                .unwrap(),
            loc2.get_id()
        );
        store.delete_note(&loc2.current()).await.unwrap();
        assert_eq!(store.get_note(&loc1.current()).await.unwrap().parent, None);
    }

    #[tokio::test]
    async fn resurrect_deleted_note() {
        let store: InMemoryStore<PlainNote> = InMemoryStore::new();
        let loc1 = store.new_note(PlainNote::new("Foo".into())).await.unwrap();
        let loc2 = store
            .update_note(&loc1, Some(PlainNote::new("Foo1".into())), None)
            .await
            .unwrap();
        store.delete_note(&loc1.current()).await.unwrap();
        let revisions = store.get_revisions_with_note(&loc1).await.unwrap();
        let (last_revision, last_note) = revisions.last().unwrap();
        assert_eq!(last_note.note_inner, PlainNote::new("Foo1".into()));
        assert_eq!(last_revision, loc2.get_revision().unwrap());
        store
            .update_note(
                &NoteLocator::Specific(loc1.get_id().clone(), last_revision.clone()),
                Some(last_note.note_inner.clone()),
                None,
            )
            .await
            .unwrap();
        assert_eq!(
            store.get_note(&loc1.current()).await.unwrap().note_inner,
            PlainNote::new("Foo1".into())
        );
    }

    #[tokio::test]
    async fn resurrected_note_parent() {
        let store: InMemoryStore<PlainNote> = InMemoryStore::new();
        let loc1 = store
            .new_note(PlainNote::new("Child".into()))
            .await
            .unwrap();
        let loc2 = store
            .new_note(PlainNote::new("Parent".into()))
            .await
            .unwrap();
        store
            .ims
            .write()
            .await
            .set_parent(&loc1, Some(loc2.get_id().clone()))
            .unwrap();
        assert!(store
            .get_note(&loc2.current())
            .await
            .unwrap()
            .children
            .contains(loc1.get_id()));
        store
            .update_note(&loc1.current(), Some(PlainNote::new("Child1".into())), None)
            .await
            .unwrap();
        store.delete_note(&loc1.current()).await.unwrap();
        assert!(!store
            .get_note(&loc2.current())
            .await
            .unwrap()
            .children
            .contains(loc1.get_id()));
        let revisions = store.get_revisions_with_note(&loc1).await.unwrap();
        let (last_revision, last_note) = revisions.last().unwrap();
        store
            .update_note(
                &NoteLocator::Specific(loc1.get_id().clone(), last_revision.clone()),
                Some(last_note.note_inner.clone()),
                None,
            )
            .await
            .unwrap();
        assert!(store
            .get_note(&loc2.current())
            .await
            .unwrap()
            .children
            .contains(loc1.get_id()));
    }

    #[tokio::test]
    async fn inherit_grandchild() {
        let store: InMemoryStore<PlainNote> = InMemoryStore::new();
        let loc1 = store
            .new_note(PlainNote::new("Child".into()))
            .await
            .unwrap();
        let loc2 = store
            .new_note(PlainNote::new("Parent".into()))
            .await
            .unwrap();
        let loc3 = store
            .new_note(PlainNote::new("Grandparent".into()))
            .await
            .unwrap();
        store
            .ims
            .write()
            .await
            .set_parent(&loc1, Some(loc2.get_id().clone()))
            .unwrap();
        store
            .ims
            .write()
            .await
            .set_parent(&loc2.current(), Some(loc3.get_id().clone()))
            .unwrap();
        store.delete_note(&loc2.current()).await.unwrap();
        assert_eq!(
            &store
                .get_note(&loc1.current())
                .await
                .unwrap()
                .parent
                .unwrap(),
            loc3.get_id()
        );
        assert!(&store
            .get_note(&loc3.current())
            .await
            .unwrap()
            .children
            .contains(loc1.get_id()));
    }

    #[tokio::test]
    async fn split_note_empty_child() {
        let store: InMemoryStore<PlainNote> = InMemoryStore::new();
        let loc1 = store.new_note(PlainNote::new("Note".into())).await.unwrap();
        let (loc1_new, loc2) = store
            .split_note(&loc1, Box::new(|x| (x, PlainNote::new("".into()))))
            .await
            .unwrap();
        assert_eq!(
            store.get_note(&loc1_new).await.unwrap().note_inner,
            PlainNote::new("Note".into())
        );
        assert_eq!(
            store.get_note(&loc2).await.unwrap().note_inner,
            PlainNote::new("".into())
        );
        assert_eq!(
            &store.get_note(&loc2).await.unwrap().parent.unwrap(),
            loc1.get_id()
        );
    }

    #[tokio::test]
    async fn split_note() {
        let store: InMemoryStore<PlainNote> = InMemoryStore::new();
        let loc1 = store.new_note(PlainNote::new("Note".into())).await.unwrap();
        let (loc1_new, loc2) = store
            .split_note(&loc1, Box::new(|x| x.split_off(2)))
            .await
            .unwrap();
        assert_eq!(
            store.get_note(&loc1_new).await.unwrap().note_inner,
            PlainNote::new("No".into())
        );
        assert_eq!(
            store.get_note(&loc2).await.unwrap().note_inner,
            PlainNote::new("te".into())
        );
        assert_eq!(
            &store.get_note(&loc2).await.unwrap().parent.unwrap(),
            loc1.get_id()
        );
        assert!(&store
            .get_note(&loc1_new)
            .await
            .unwrap()
            .children
            .contains(loc2.get_id()));
    }

    #[tokio::test]
    async fn merge_note() {
        let store: InMemoryStore<PlainNote> = InMemoryStore::new();
        let loc1 = store.new_note(PlainNote::new("Note".into())).await.unwrap();
        let (loc1_new, loc2) = store
            .split_note(&loc1, Box::new(|x| x.split_off(2)))
            .await
            .unwrap();
        assert_eq!(
            store.get_note(&loc1_new).await.unwrap().note_inner,
            PlainNote::new("No".into())
        );
        assert_eq!(
            store.get_note(&loc2).await.unwrap().note_inner,
            PlainNote::new("te".into())
        );
        let (loc2_new, loc3) = store
            .split_note(&loc2, Box::new(|x| x.split_off(1)))
            .await
            .unwrap();
        assert_eq!(
            store.get_note(&loc2_new).await.unwrap().note_inner,
            PlainNote::new("t".into())
        );
        assert_eq!(
            store.get_note(&loc3).await.unwrap().note_inner,
            PlainNote::new("e".into())
        );
        let loc_merge = store
            .merge_note(&loc1_new, &loc2_new, Box::new(|x, y| x.merge(y)))
            .await
            .unwrap();
        assert_eq!(
            store.get_note(&loc_merge).await.unwrap().note_inner,
            PlainNote::new("Not".into())
        );
    }
}
