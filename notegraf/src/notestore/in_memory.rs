//! In-memory storage of notes
use crate::errors::NoteStoreError;
use crate::note::NoteLocator;
use crate::notemetadata::{NoteMetadata, NoteMetadataEditable};
use crate::notestore::search::SearchRequest;
use crate::notestore::Revisions;
use crate::{Note, NoteID, NoteStore, NoteType, Revision};
use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::marker::PhantomData;
use std::path::Path;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InMemoryNoteStored<T> {
    title: String,
    note_inner: String,
    id: NoteID,
    revision: Revision,
    branches: HashSet<NoteID>,
    next: Option<NoteID>,
    metadata: NoteMetadata,
    _phantom: PhantomData<T>,
}

#[derive(Debug, Clone)]
struct InMemoryNoteComputed<T> {
    title: String,
    note_inner: T,
    id: NoteID,
    revision: Revision,
    is_current: bool,
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
    fn get_title(&self) -> String {
        self.title.clone()
    }

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

    fn is_current(&self) -> bool {
        self.is_current
    }
}

/// In-memory storage.
///
/// This is mostly designed for development use, because there is no persistence layer.
#[derive(Debug, Serialize, Deserialize)]
struct InMemoryStoreInner<T> {
    pub notes: HashMap<NoteID, HashMap<Revision, InMemoryNoteStored<T>>>,
    current_revision: HashMap<NoteID, Revision>,
    note_id_counter: u64,
    revision_id_counter: HashMap<NoteID, u64>,
}

impl<T: NoteType> Default for InMemoryStoreInner<T> {
    fn default() -> Self {
        InMemoryStoreInner {
            notes: Default::default(),
            current_revision: Default::default(),
            note_id_counter: 0,
            revision_id_counter: Default::default(),
        }
    }
}

fn note_contains_lexemes(title: &str, note_inner: &str, lexemes: &[String]) -> bool {
    let title = title.to_lowercase();
    let note_inner = note_inner.to_lowercase();
    for lexeme in lexemes {
        let lexeme_lower = lexeme.to_lowercase();
        if !title.contains(&lexeme_lower) && !note_inner.contains(&lexeme_lower) {
            return false;
        }
    }
    true
}

fn note_excludes_lexemes(title: &str, note_inner: &str, lexemes_excluded: &[String]) -> bool {
    let title = title.to_lowercase();
    let note_inner = note_inner.to_lowercase();
    for lexeme in lexemes_excluded {
        let lexeme_lower = lexeme.to_lowercase();
        if title.contains(&lexeme_lower) || note_inner.contains(&lexeme_lower) {
            return false;
        }
    }
    true
}

fn note_is_orphan<T: NoteType>(note: &dyn Note<T>) -> bool {
    note.get_prev().is_none() && note.get_parent().is_none() && note.get_references().is_empty()
}

impl<T: NoteType> InMemoryStoreInner<T> {
    pub fn new() -> Self {
        Default::default()
    }

    /// Generate a new [`NoteID`].
    ///
    /// We use a deterministic sequential format for easy testing
    fn get_new_noteid(&mut self) -> NoteID {
        let note_id = NoteID::new(format!("note-{}", self.note_id_counter));
        self.note_id_counter += 1;
        note_id
    }

    /// Generate a new [`Revision`].
    ///
    /// We use a deterministic sequential format for easy testing
    fn get_new_revision(&mut self, note_id: &NoteID) -> Revision {
        let revision_counter = self.revision_id_counter.entry(note_id.clone()).or_insert(0);
        let revision = Revision::new(format!("revision-{}", *revision_counter));
        *revision_counter += 1;
        revision
    }

    /// Does the locator points to a current revision
    fn is_current(&self, loc: &NoteLocator) -> Result<bool, NoteStoreError> {
        if let Some(r) = loc.get_revision() {
            // If the argument is a specific revision, then compare it with the current revision
            let current_rev = self.get_current_revision(loc)?;
            if let Some(cr) = current_rev {
                Ok(&cr == r)
            } else {
                Ok(false)
            }
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
        let new_revision = self.get_new_revision(id);
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
        if is_resurrecting {
            // If a note has branches, it cannot be deleted in the first place
            assert!(updated_note.branches.is_empty());
            // If a note previously has a next note, we will clear the attribute, in case the next
            // note now has a prev
            updated_note.next = None;
        }
        note_revisions.insert(new_revision.clone(), updated_note);
        self.current_revision
            .insert(id.clone(), new_revision.clone());
        Ok(NoteLocator::Specific(id.clone(), new_revision))
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
            let cr = self.get_current_revision(loc)?;
            if let Some(r) = cr {
                self.get_note_by_revision(id, &r)
            } else {
                Err(NoteStoreError::RevisionNotExist(
                    id.clone(),
                    Revision::new("current".to_owned()),
                ))
            }
        }
    }

    fn get_references(&self, referent: &NoteID) -> HashSet<NoteID> {
        let mut references = HashSet::new();
        for (id, revision) in &self.current_revision {
            let note = self.get_note_by_revision(id, revision).unwrap();
            if T::from(note.note_inner)
                .get_referents()
                .unwrap()
                .contains(referent)
            {
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

    fn get_all_current_notes(&self) -> Vec<InMemoryNoteStored<T>> {
        self.current_revision
            .iter()
            .map(|(id, revision)| self.get_note_by_revision(id, revision).unwrap())
            .collect()
    }

    fn new_note_helper(
        &mut self,
        title: String,
        note_inner: T,
        metadata: NoteMetadataEditable,
    ) -> NoteLocator {
        let id = self.get_new_noteid();
        let revision = self.get_new_revision(&id);
        let note = InMemoryNoteStored {
            title,
            note_inner: note_inner.into(),
            id: id.clone(),
            revision: revision.clone(),
            branches: Default::default(),
            next: None,
            metadata: NoteMetadata::from_editable(metadata),
            _phantom: PhantomData,
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
        NoteLocator::Specific(id, revision)
    }

    // The methods above are helper methods
    // The methods below are to implement the NoteStore interface
    fn new_note(
        &mut self,
        title: String,
        note_inner: T,
        metadata: NoteMetadataEditable,
    ) -> Result<NoteLocator, NoteStoreError> {
        Ok(self.new_note_helper(title, note_inner, metadata))
    }

    fn compute_stored_note(
        &self,
        s: InMemoryNoteStored<T>,
    ) -> Result<InMemoryNoteComputed<T>, NoteStoreError> {
        let note_inner = T::from(s.note_inner);
        let referents = note_inner
            .get_referents()
            .map_err(|e| NoteStoreError::ParseError(format!("{e:?}")))?;
        let references = self.get_references(&s.id);
        let parent = self.get_parent(&s.id);
        let prev = self.get_prev(&s.id);
        let current_revision = self.get_current_revision(&NoteLocator::Current(s.id.clone()))?;
        let is_current = if let Some(r) = current_revision {
            s.revision == r
        } else {
            false
        };
        Ok(InMemoryNoteComputed {
            title: s.title,
            note_inner,
            id: s.id,
            revision: s.revision,
            is_current,
            parent,
            branches: s.branches,
            prev,
            next: s.next,
            referents,
            references,
            metadata: s.metadata,
        })
    }

    fn get_note(&self, loc: &NoteLocator) -> Result<Box<dyn Note<T>>, NoteStoreError> {
        let note_stored = self.get_note_stored(loc)?;
        Ok(Box::new(self.compute_stored_note(note_stored)?) as Box<dyn Note<T>>)
    }

    fn update_note(
        &mut self,
        loc: &NoteLocator,
        title: Option<String>,
        note_inner: Option<T>,
        note_metadata: NoteMetadataEditable,
    ) -> Result<NoteLocator, NoteStoreError> {
        self.update_note_helper(loc, |old_note| {
            let mut note = old_note.clone();
            if let Some(t) = title {
                note.title = t;
            }
            if let Some(n) = note_inner {
                note.note_inner = n.into();
            }

            note.metadata = note.metadata.apply_editable(note_metadata);
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

    fn get_current_revision(&self, loc: &NoteLocator) -> Result<Option<Revision>, NoteStoreError> {
        let id = loc.get_id();
        if let Some(r) = self.current_revision.get(id) {
            Ok(Some(r.clone()))
        } else if self.notes.contains_key(id) {
            Ok(None)
        } else {
            Err(NoteStoreError::NoteNotExist(id.clone()))
        }
    }

    /// Get all revisions of a note with actual notes
    fn get_revisions(&self, loc: &NoteLocator) -> Result<Revisions<T>, NoteStoreError> {
        let id = loc.get_id();
        let notes: Vec<InMemoryNoteStored<T>> = self
            .notes
            .get(id)
            .ok_or_else(|| NoteStoreError::NoteNotExist(id.clone()))
            .map(|rs| {
                let mut v: Vec<InMemoryNoteStored<T>> = rs.values().cloned().collect();
                v.sort_by_key(|n| n.metadata.modified_at);
                v
            })?;
        notes
            .into_iter()
            .map(|n| {
                self.compute_stored_note(n)
                    .map(|n_computed| Box::new(n_computed) as Box<dyn Note<T>>)
            })
            .collect()
    }

    fn append_note(
        &mut self,
        last: &NoteID,
        title: String,
        note_inner: T,
        metadata: NoteMetadataEditable,
    ) -> Result<NoteLocator, NoteStoreError> {
        let last_loc = NoteLocator::Current(last.clone());
        let last_note = self.get_note_stored(&last_loc)?;
        if let Some(n) = last_note.next {
            return Err(NoteStoreError::ExistingNext(last_note.id, n));
        }
        let loc = self.new_note_helper(title, note_inner, metadata);
        self.update_note_helper(&last_loc, |old_note| {
            let mut note = old_note.clone();
            note.next = Some(loc.get_id().clone());
            Ok(note)
        })?;
        Ok(loc)
    }

    fn add_branch(
        &mut self,
        parent: &NoteID,
        title: String,
        note_inner: T,
        metadata: NoteMetadataEditable,
    ) -> Result<NoteLocator, NoteStoreError> {
        let parent_loc = NoteLocator::Current(parent.clone());
        let child_loc = self.new_note_helper(title, note_inner, metadata);
        self.update_note_helper(&parent_loc, |old_note| {
            let mut note = old_note.clone();
            note.branches.insert(child_loc.get_id().clone());
            Ok(note)
        })?;
        Ok(child_loc)
    }

    fn search(&self, sr: &SearchRequest) -> Result<Revisions<T>, NoteStoreError> {
        let notes: Vec<InMemoryNoteStored<T>> = self.get_all_current_notes();
        let revisions: Result<Revisions<T>, NoteStoreError> = notes
            .into_iter()
            .map(|x| {
                self.compute_stored_note(x)
                    .map(|x_computed| Box::new(x_computed) as Box<dyn Note<T>>)
            })
            .collect();
        let mut revisions: Revisions<T> = revisions?
            .into_iter()
            .filter(|x| {
                note_contains_lexemes(&x.get_title(), &x.get_note_inner().into(), &sr.lexemes)
                    && note_excludes_lexemes(
                        &x.get_title(),
                        &x.get_note_inner().into(),
                        &sr.lexemes_excluded,
                    )
                    && HashSet::from_iter(sr.tags.to_vec()).is_subset(&x.get_metadata().tags)
                    && HashSet::from_iter(sr.tags_excluded.to_vec())
                        .intersection(&x.get_metadata().tags)
                        .count()
                        == 0
                    && (!sr.orphan || note_is_orphan(x.as_ref()))
                    && (!sr.no_tag || x.get_metadata().tags.is_empty())
            })
            .collect();
        if sr.sort_by_created_at() {
            revisions.sort_by_key(|n| Reverse(n.get_metadata().created_at));
        }
        if let Some(l) = sr.limit {
            revisions = revisions.into_iter().take(l as usize).collect();
        }
        Ok(revisions)
    }

    fn tags(&self) -> Result<Vec<String>, NoteStoreError> {
        let mut tags = HashSet::new();
        let notes: Vec<InMemoryNoteStored<T>> = self.get_all_current_notes();
        for note in &notes {
            for tag in &note.metadata.tags {
                tags.insert(tag.clone());
            }
        }
        Ok(Vec::from_iter(tags))
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
}

impl<T: NoteType> Default for InMemoryStore<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: NoteType> NoteStore<T> for InMemoryStore<T> {
    fn new_note(
        &self,
        title: String,
        note_inner: T,
        metadata: NoteMetadataEditable,
    ) -> BoxFuture<Result<NoteLocator, NoteStoreError>> {
        Box::pin(async move {
            let mut ims = self.ims.write().await;
            ims.new_note(title, note_inner, metadata)
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
        title: Option<String>,
        note_inner: Option<T>,
        note_metadata: NoteMetadataEditable,
    ) -> BoxFuture<'a, Result<NoteLocator, NoteStoreError>> {
        Box::pin(async move {
            let mut ims = self.ims.write().await;
            ims.update_note(loc, title, note_inner, note_metadata)
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

    fn get_revisions<'a>(
        &'a self,
        loc: &'a NoteLocator,
    ) -> BoxFuture<'a, Result<Revisions<T>, NoteStoreError>> {
        Box::pin(async move {
            let ims = self.ims.read().await;
            ims.get_revisions(loc)
        })
    }

    fn get_current_revision<'a>(
        &'a self,
        loc: &'a NoteLocator,
    ) -> BoxFuture<'a, Result<Option<Revision>, NoteStoreError>> {
        Box::pin(async move {
            let ims = self.ims.read().await;
            ims.get_current_revision(loc)
        })
    }

    fn append_note<'a>(
        &'a self,
        last: &'a NoteID,
        title: String,
        note_inner: T,
        metadata: NoteMetadataEditable,
    ) -> BoxFuture<'a, Result<NoteLocator, NoteStoreError>> {
        Box::pin(async move {
            let mut ims = self.ims.write().await;
            ims.append_note(last, title, note_inner, metadata)
        })
    }

    fn add_branch<'a>(
        &'a self,
        parent: &'a NoteID,
        title: String,
        note_inner: T,
        metadata: NoteMetadataEditable,
    ) -> BoxFuture<'a, Result<NoteLocator, NoteStoreError>> {
        Box::pin(async move {
            let mut ims = self.ims.write().await;
            ims.add_branch(parent, title, note_inner, metadata)
        })
    }

    fn search<'a>(
        &'a self,
        sr: &'a SearchRequest,
    ) -> BoxFuture<'a, Result<Revisions<T>, NoteStoreError>> {
        Box::pin(async move {
            let ims = self.ims.read().await;
            ims.search(sr)
        })
    }

    fn tags(&self) -> BoxFuture<Result<Vec<String>, NoteStoreError>> {
        Box::pin(async move {
            let ims = self.ims.read().await;
            ims.tags()
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
    use crate::note::NoteSerializable;
    use crate::notestore::tests as common_tests;
    use crate::notetype::PlainNote;
    use std::env;

    #[tokio::test]
    async fn unique_id() {
        common_tests::unique_id(InMemoryStore::new()).await;
    }

    #[tokio::test]
    async fn new_note_revision() {
        common_tests::new_note_revision(InMemoryStore::new()).await;
    }

    #[tokio::test]
    async fn new_note_retrieve() {
        common_tests::new_note_retrieve(InMemoryStore::new()).await;
    }

    #[tokio::test]
    async fn backup() {
        let store: InMemoryStore<PlainNote> = InMemoryStore::new();
        let loc1 = store
            .new_note(
                "".to_owned(),
                PlainNote::new("Foo".into()),
                NoteMetadataEditable::unchanged(),
            )
            .await
            .unwrap();
        let loc2 = store
            .new_note(
                "".to_owned(),
                PlainNote::new("Bar".into()),
                NoteMetadataEditable::unchanged(),
            )
            .await
            .unwrap();

        store.backup(Box::new(env::temp_dir())).await.unwrap();
        let store_restore: InMemoryStore<PlainNote> =
            InMemoryStore::restore(env::temp_dir()).unwrap();
        for loc in [loc1, loc2].iter() {
            let note = store.get_note(loc).await.unwrap();
            let note_restore = store_restore.get_note(loc).await.unwrap();
            assert_eq!(
                serde_json::to_string(&NoteSerializable::all_fields(note)).unwrap(),
                serde_json::to_string(&NoteSerializable::all_fields(note_restore)).unwrap()
            );
        }
    }

    #[tokio::test]
    async fn update_note() {
        common_tests::update_note(InMemoryStore::new()).await;
    }

    #[tokio::test]
    async fn add_branch() {
        common_tests::add_branch(InMemoryStore::new()).await;
    }

    #[tokio::test]
    async fn delete_note_specific() {
        common_tests::delete_note_specific(InMemoryStore::new()).await;
    }

    #[tokio::test]
    async fn delete_note_current() {
        common_tests::delete_note_current(InMemoryStore::new()).await;
    }

    #[tokio::test]
    async fn delete_note_with_branches() {
        common_tests::delete_note_with_branches(InMemoryStore::new()).await;
    }

    #[tokio::test]
    async fn resurrect_deleted_note() {
        common_tests::resurrect_deleted_note(InMemoryStore::new()).await;
    }

    #[tokio::test]
    async fn delete_middle_note_sequence() {
        common_tests::delete_middle_note_sequence(InMemoryStore::new()).await;
    }

    #[tokio::test]
    async fn resurrect_note_in_sequence() {
        common_tests::resurrect_note_in_sequence(InMemoryStore::new()).await;
    }

    #[tokio::test]
    async fn search_recent() {
        common_tests::search_recent(InMemoryStore::new()).await;
    }

    #[tokio::test]
    async fn search_fulltext() {
        common_tests::search_fulltext(InMemoryStore::new()).await;
    }

    #[tokio::test]
    async fn search_nonexist() {
        common_tests::search_nonexist(InMemoryStore::new()).await;
    }

    #[tokio::test]
    async fn backlink() {
        common_tests::backlink(InMemoryStore::new()).await;
    }

    #[tokio::test]
    async fn search_tags() {
        common_tests::search_tags(InMemoryStore::new()).await;
    }

    #[tokio::test]
    async fn search_orphan() {
        common_tests::search_orphan(InMemoryStore::new()).await;
    }

    #[tokio::test]
    async fn search_notag() {
        common_tests::search_notag(InMemoryStore::new()).await;
    }

    #[tokio::test]
    async fn issue_151() {
        common_tests::issue_151(InMemoryStore::new()).await;
    }

    #[tokio::test]
    async fn tags() {
        common_tests::tags(InMemoryStore::new()).await;
    }

    #[tokio::test]
    async fn search_limit_override() {
        common_tests::search_limit_override(InMemoryStore::new()).await;
    }

    #[tokio::test]
    async fn search_tag_exclude() {
        common_tests::search_tag_exclude(InMemoryStore::new()).await;
    }

    #[tokio::test]
    async fn search_lexeme_exclude() {
        common_tests::search_lexeme_exclude(InMemoryStore::new()).await;
    }

    #[tokio::test]
    async fn issue_158() {
        common_tests::issue_158(InMemoryStore::new()).await;
    }
}
