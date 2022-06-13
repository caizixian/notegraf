//! In-memory storage of notes
use crate::errors::NoteStoreError;
use crate::note::NoteLocator;
use crate::notemetadata::NoteMetadata;
use crate::{Note, NoteID, NoteStore, NoteType, Revision};
use chrono::{DateTime, Utc};
use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InMemoryNoteStored<T> {
    note_inner: T,
    id: NoteID,
    revision: Revision,
    branches: HashSet<NoteID>,
    next: Option<NoteID>,
    metadata: NoteMetadata,
}

type RevisionsOfNote<T> = Vec<(Revision, InMemoryNoteStored<T>)>;

#[derive(Debug, Clone, Serialize)]
struct InMemoryNoteComputed<T> {
    note_inner: T,
    id: NoteID,
    revision: Revision,
    parent: Option<NoteID>,
    branches: HashSet<NoteID>,
    prev: Option<NoteID>,
    next: Option<NoteID>,
    referents: HashSet<NoteID>,
    references: HashSet<NoteID>,
    metadata: NoteMetadata,
}

impl<T> Note<T> for InMemoryNoteComputed<T>
where
    T: NoteType,
{
    fn get_note_inner(&self) -> T {
        self.note_inner.clone()
    }

    fn get_id(&self) -> NoteID {
        self.id.clone()
    }

    fn get_revision(&self) -> Revision {
        self.revision.clone()
    }

    fn get_parent(&self) -> Option<NoteID> {
        self.parent.clone()
    }

    fn get_branches(&self) -> HashSet<NoteID> {
        self.branches.clone()
    }

    fn get_prev(&self) -> Option<NoteID> {
        self.prev.clone()
    }

    fn get_next(&self) -> Option<NoteID> {
        self.next.clone()
    }

    fn get_references(&self) -> HashSet<NoteID> {
        self.references.clone()
    }

    fn get_referents(&self) -> HashSet<NoteID> {
        self.referents.clone()
    }

    fn get_metadata(&self) -> NoteMetadata {
        self.metadata.clone()
    }
}

/// In-memory storage.
///
/// This is mostly designed for development use, because there is no persistence layer.
#[derive(Debug, Serialize, Deserialize)]
struct InMemoryStoreInner<T> {
    pub notes: HashMap<NoteID, HashMap<Revision, InMemoryNoteStored<T>>>,
    current_revision: HashMap<NoteID, Revision>,
}

impl<T: NoteType> Default for InMemoryStoreInner<T> {
    fn default() -> Self {
        InMemoryStoreInner {
            notes: Default::default(),
            current_revision: Default::default(),
        }
    }
}

impl<T: NoteType> InMemoryStoreInner<T> {
    pub fn new() -> Self {
        Default::default()
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
        F: FnOnce(&InMemoryNoteStored<T>) -> Result<InMemoryNoteStored<T>, NoteStoreError>,
    {
        let (id, rev) = loc.unpack();
        let is_resurrecting = self.is_deleted(loc)?;
        let old_note = if is_resurrecting || self.is_current(loc)? {
            self.get_note_stored(loc)?
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
        note_revisions.insert(new_revision.clone(), updated_note);
        self.current_revision
            .insert(id.clone(), new_revision.clone());
        Ok(NoteLocator::Specific(id.clone(), new_revision))
    }

    /// Get all revisions of a note with actual notes
    fn get_revisions_with_note(
        &self,
        loc: &NoteLocator,
    ) -> Result<Vec<(Revision, InMemoryNoteStored<T>)>, NoteStoreError> {
        let id = loc.get_id();
        self.notes
            .get(id)
            .ok_or_else(|| NoteStoreError::NoteNotExist(id.clone()))
            .map(|rs| {
                let mut v: Vec<(Revision, InMemoryNoteStored<T>)> =
                    rs.iter().map(|(r, n)| (r.clone(), n.clone())).collect();
                v.sort_by_key(|(_, n)| n.metadata.modified_at);
                v
            })
    }

    fn get_note_by_revision(
        &self,
        id: &NoteID,
        rev: &Revision,
    ) -> Result<InMemoryNoteStored<T>, NoteStoreError> {
        Ok(self
            .notes
            .get(id)
            .ok_or_else(|| NoteStoreError::NoteNotExist(id.clone()))?
            .get(rev)
            .ok_or_else(|| NoteStoreError::RevisionNotExist(id.clone(), rev.clone()))?
            .clone())
    }

    fn get_note_stored(&self, loc: &NoteLocator) -> Result<InMemoryNoteStored<T>, NoteStoreError> {
        let (id, rev) = loc.unpack();
        if let Some(r) = rev {
            self.get_note_by_revision(id, r)
        } else {
            self.get_note_by_revision(id, &self.get_current_revision(loc)?)
        }
    }

    fn get_references(&self, referent: &NoteID) -> HashSet<NoteID> {
        let mut references = HashSet::new();
        for (id, revision) in &self.current_revision {
            let note = self.get_note_by_revision(id, revision).unwrap();
            if note.note_inner.get_referents().unwrap().contains(referent) {
                references.insert(note.id.clone());
            }
        }
        references
    }

    fn get_parent(&self, child: &NoteID) -> Option<NoteID> {
        for (id, revision) in &self.current_revision {
            let note = self.get_note_by_revision(id, revision).unwrap();
            if note.branches.contains(child) {
                return Some(note.id);
            }
        }
        None
    }

    fn get_prev(&self, next: &NoteID) -> Option<NoteID> {
        for (id, revision) in &self.current_revision {
            let note = self.get_note_by_revision(id, revision).unwrap();
            if let Some(ref nn) = note.next {
                if nn == next {
                    return Some(note.id);
                }
            }
        }
        None
    }

    // The methods above are helper methods
    // The methods below are to implement the NoteStore interface
    fn new_note(
        &mut self,
        note_inner: T,
        metadata: Option<NoteMetadata>,
    ) -> Result<NoteLocator, NoteStoreError> {
        let id = self.get_new_noteid();
        let revision = self.get_new_revision();
        let note = InMemoryNoteStored {
            note_inner,
            id: id.clone(),
            revision: revision.clone(),
            branches: Default::default(),
            next: None,
            metadata: metadata.unwrap_or_default(),
        };
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

    fn get_note(&self, loc: &NoteLocator) -> Result<Box<dyn Note<T>>, NoteStoreError> {
        let note_stored = self.get_note_stored(loc)?;
        let referents = note_stored
            .note_inner
            .get_referents()
            .map_err(|e| NoteStoreError::ParseError(format!("{:?}", e)))?;
        let references = self.get_references(&note_stored.id);
        let parent = self.get_parent(&note_stored.id);
        let prev = self.get_prev(&note_stored.id);
        Ok(Box::new(InMemoryNoteComputed {
            note_inner: note_stored.note_inner,
            id: note_stored.id,
            revision: note_stored.revision,
            parent,
            branches: note_stored.branches,
            prev,
            next: note_stored.next,
            referents,
            references,
            metadata: note_stored.metadata,
        }))
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
            let note = self.get_note_stored(loc).unwrap();
            if !note.branches.is_empty() {
                return Err(NoteStoreError::HasBranches(id.clone()));
            }
            // Avoid dangling references
            if !self.get_references(id).is_empty() {
                return Err(NoteStoreError::HasReferences(id.clone()));
            }
            if note.next.is_some() {
                let prev = self.get_prev(id);
                if let Some(prev_id) = prev {
                    self.update_note_helper(&NoteLocator::Current(prev_id), |old_note| {
                        let mut parent_note = old_note.clone();
                        assert_eq!(parent_note.next.as_ref(), Some(id));
                        parent_note.next = note.next;
                        Ok(parent_note)
                    })?;
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
            .map(|rs| {
                let mut v: Vec<(Revision, DateTime<Utc>)> = rs
                    .iter()
                    .map(|(r, n)| (r.clone(), n.metadata.modified_at))
                    .collect();
                // Newer to older. In other words, larger timestamps to smaller timestamps
                v.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
                let revs: Vec<Revision> = v.iter().map(|(r, _)| r.clone()).collect();
                revs
            })
    }

    fn append_note(&mut self, last: &NoteLocator, next: &NoteID) -> Result<(), NoteStoreError> {
        self.update_note_helper(last, |old_note| {
            let mut note = old_note.clone();
            if note.next.is_some() {
                return Err(NoteStoreError::ExistingNext(note.id, next.clone()));
            }
            note.next = Some(next.clone());
            Ok(note)
        })?;
        Ok(())
    }

    fn add_branch(&mut self, parent: &NoteLocator, child: &NoteID) -> Result<(), NoteStoreError> {
        self.update_note_helper(parent, |old_note| {
            let mut note = old_note.clone();
            note.branches.insert(child.clone());
            Ok(note)
        })?;
        Ok(())
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
    fn new_note(
        &self,
        note_inner: T,
        metadata: Option<NoteMetadata>,
    ) -> BoxFuture<Result<NoteLocator, NoteStoreError>> {
        Box::pin(async move {
            let mut ims = self.ims.write().await;
            ims.new_note(note_inner, metadata)
        })
    }

    fn get_note<'a>(
        &'a self,
        loc: &'a NoteLocator,
    ) -> BoxFuture<'a, Result<Box<dyn Note<T>>, NoteStoreError>> {
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

    fn append_note<'a>(
        &'a self,
        last: &'a NoteLocator,
        next: &'a NoteID,
    ) -> BoxFuture<'a, Result<(), NoteStoreError>> {
        Box::pin(async move {
            let mut ims = self.ims.write().await;
            ims.append_note(last, next)
        })
    }

    fn add_branch<'a>(
        &'a self,
        parent: &'a NoteLocator,
        child: &'a NoteID,
    ) -> BoxFuture<'a, Result<(), NoteStoreError>> {
        Box::pin(async move {
            let mut ims = self.ims.write().await;
            ims.add_branch(parent, child)
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
    use super::*;
    use crate::notestore::tests as common_tests;
    use crate::notetype::PlainNote;
    use std::env;

    #[tokio::test]
    async fn unique_id() {
        let store: InMemoryStore<PlainNote> = InMemoryStore::new();
        common_tests::unique_id(store).await;
    }

    #[tokio::test]
    async fn new_note_revision() {
        let store: InMemoryStore<PlainNote> = InMemoryStore::new();
        common_tests::new_note_revision(store).await;
    }

    #[tokio::test]
    async fn new_note_retrieve() {
        let store: InMemoryStore<PlainNote> = InMemoryStore::new();
        common_tests::new_note_retrieve(store).await;
    }

    #[tokio::test]
    async fn backup() {
        let store: InMemoryStore<PlainNote> = InMemoryStore::new();
        let loc1 = store
            .new_note(PlainNote::new("Foo".into()), None)
            .await
            .unwrap();
        let loc2 = store
            .new_note(PlainNote::new("Bar".into()), None)
            .await
            .unwrap();

        store.backup(Box::new(env::temp_dir())).await.unwrap();
        let store_restore: InMemoryStore<PlainNote> =
            InMemoryStore::restore(env::temp_dir()).unwrap();
        for loc in vec![loc1, loc2].iter() {
            let note = store.get_note(loc).await.unwrap();
            let note_restore = store_restore.get_note(loc).await.unwrap();
            assert_eq!(
                serde_json::to_string(&note).unwrap(),
                serde_json::to_string(&note_restore).unwrap()
            );
        }
    }

    #[tokio::test]
    async fn update_note() {
        let store: InMemoryStore<PlainNote> = InMemoryStore::new();
        common_tests::update_note(store).await;
    }

    #[tokio::test]
    async fn add_branch() {
        let store: InMemoryStore<PlainNote> = InMemoryStore::new();
        common_tests::add_branch(store).await;
    }

    #[tokio::test]
    async fn delete_note_specific() {
        let store: InMemoryStore<PlainNote> = InMemoryStore::new();
        let loc1 = store
            .new_note(PlainNote::new("Note".into()), None)
            .await
            .unwrap();
        store.delete_note(&loc1).await.unwrap();
        assert!(store.ims.read().await.is_deleted(&loc1).unwrap());
    }

    #[tokio::test]
    async fn delete_note_current() {
        let store: InMemoryStore<PlainNote> = InMemoryStore::new();
        let loc1 = store
            .new_note(PlainNote::new("Note".into()), None)
            .await
            .unwrap();
        store.delete_note(&loc1.current()).await.unwrap();
        assert!(store.ims.read().await.is_deleted(&loc1).unwrap());
        assert!(matches!(
            store.ims.read().await.is_current(&loc1).err().unwrap(),
            NoteStoreError::NoteDeleted(_)
        ));
    }

    #[tokio::test]
    async fn delete_note_with_branches() {
        let store: InMemoryStore<PlainNote> = InMemoryStore::new();
        common_tests::delete_note_with_branches(store).await;
    }

    #[tokio::test]
    async fn resurrect_deleted_note() {
        let store: InMemoryStore<PlainNote> = InMemoryStore::new();
        let loc1 = store
            .new_note(PlainNote::new("Foo".into()), None)
            .await
            .unwrap();
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
            store
                .get_note(&loc1.current())
                .await
                .unwrap()
                .get_note_inner(),
            PlainNote::new("Foo1".into())
        );
    }

    #[tokio::test]
    async fn delete_middle_note_sequence() {
        let store: InMemoryStore<PlainNote> = InMemoryStore::new();
        common_tests::delete_middle_note_sequence(store).await;
    }
}
