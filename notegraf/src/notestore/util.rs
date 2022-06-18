use crate::notemetadata::NoteMetadata;
use crate::notestore::BoxedNoteStore;
use crate::MarkdownNote;
use std::collections::HashSet;
use std::option::Option::None;

pub async fn populate_test_data(store: &BoxedNoteStore<MarkdownNote>) {
    let loc1 = store
        .new_note(
            Some("A big sequence!".to_owned()),
            "# Sequence1\nbody1".into(),
            None,
        )
        .await
        .unwrap();
    let loc2 = store
        .new_note(
            None,
            "## Sequence2\nbody2".into(),
            Some(NoteMetadata {
                tags: HashSet::from_iter(
                    vec!["tag1".to_owned(), "tag2".to_owned()].iter().cloned(),
                ),
                ..Default::default()
            }),
        )
        .await
        .unwrap();
    store.append_note(&loc1, loc2.get_id()).await.unwrap();
    let loc3 = store
        .new_note(None, "## Sequence3".into(), None)
        .await
        .unwrap();
    store.append_note(&loc2, loc3.get_id()).await.unwrap();
}
