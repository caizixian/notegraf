use std::collections::HashSet;
use crate::{MarkdownNote};
use crate::notemetadata::NoteMetadata;
use crate::notestore::BoxedNoteStore;

pub async fn populate_test_data(store: &BoxedNoteStore<MarkdownNote>) {
    let loc1 = store.new_note("# Sequence1\nbody1".into(), None).await.unwrap();
    let loc2 = store.new_note(
        "## Sequence2\nbody2".into(),
        Some(NoteMetadata {
            tags: HashSet::from_iter(vec!["tag1".to_owned(), "tag2".to_owned()].iter().cloned()),
            ..Default::default()
        })
    ).await.unwrap();
    store.append_note(&loc1, loc2.get_id()).await.unwrap();
    let loc3 = store.new_note("## Sequence3".into(), None).await.unwrap();
    store.append_note(&loc2, loc3.get_id()).await.unwrap();
}