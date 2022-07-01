use crate::notemetadata::NoteMetadata;
use crate::notestore::BoxedNoteStore;
use crate::MarkdownNote;
use std::collections::HashSet;
use std::option::Option::None;

pub async fn populate_test_data(store: &BoxedNoteStore<MarkdownNote>) {
    let loc1 = store
        .new_note(
            "A big sequence!".to_owned(),
            "# Sequence1\nbody1".into(),
            None,
        )
        .await
        .unwrap();
    let loc2 = store
        .new_note(
            "".to_owned(),
            "## Code testing\n`inline code`\n```python\na = [1, 2, 3, 4]\nfor n in a:\n    print(n)\n```\n".into(),
            Some(NoteMetadata {
                tags: HashSet::from_iter(
                    vec!["tag1".to_owned(), "tag2".to_owned()].iter().cloned(),
                ),
                ..Default::default()
            }),
        )
        .await
        .unwrap();
    store
        .append_note(loc1.get_id(), loc2.get_id())
        .await
        .unwrap();
    let loc3 = store
        .new_note(
            "".to_owned(),
            "## Math testing\n```math\n\\frac{1}{2}\n```\n\nInline math `${\\frac{1}{2}}$`\n"
                .into(),
            None,
        )
        .await
        .unwrap();
    store
        .append_note(loc2.get_id(), loc3.get_id())
        .await
        .unwrap();
}
