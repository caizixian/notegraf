use crate::notemetadata::NoteMetadataEditable;
use crate::notestore::BoxedNoteStore;
use crate::MarkdownNote;
use std::collections::HashSet;
use std::option::Option::None;

pub async fn populate_test_data(store: &BoxedNoteStore<MarkdownNote>) {
    let loc1 = store
        .new_note(
            "A big sequence!".to_owned(),
            "# Sequence1\nbody1".into(),
            NoteMetadataEditable::unchanged(),
        )
        .await
        .unwrap();
    let loc2 = store
        .append_note(
            loc1.get_id(),
            "".to_owned(),
            "## Code testing\n`inline code`\n```python\na = [1, 2, 3, 4]\nfor n in a:\n    print(n)\n```\n".into(),
            NoteMetadataEditable {
                tags: Some(HashSet::from_iter(
                    ["tag1".to_owned(), "tag2".to_owned()].iter().cloned(),
                )),
                custom_metadata: None
            },
        )
        .await
        .unwrap();
    let _loc3 = store
        .append_note(
            loc2.get_id(),
            "".to_owned(),
            "## Math testing\n```math\n\\frac{1}{2}\n```\n\nInline math `${\\frac{1}{2}}$`\n"
                .into(),
            NoteMetadataEditable::unchanged(),
        )
        .await
        .unwrap();
}
