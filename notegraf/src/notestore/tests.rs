use crate::errors::NoteStoreError;
use crate::notemetadata::NoteMetadataEditable;
use crate::{NoteLocator, NoteStore, PlainNote};
use std::option::Option::None;

async fn is_deleted(
    store: &impl NoteStore<PlainNote>,
    loc: &NoteLocator,
) -> Result<bool, NoteStoreError> {
    let cr = store.get_current_revision(loc).await?;
    Ok(cr.is_none())
}

pub(super) async fn unique_id(store: impl NoteStore<PlainNote>) {
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
    assert_ne!(loc1.get_id(), loc2.get_id());
}

pub(super) async fn new_note_revision(store: impl NoteStore<PlainNote>) {
    let loc = store
        .new_note(
            "".to_owned(),
            PlainNote::new("Foo".into()),
            NoteMetadataEditable::unchanged(),
        )
        .await
        .unwrap();
    let rev = loc.get_revision().unwrap();
    assert_eq!(
        &store
            .get_current_revision(&loc.current())
            .await
            .unwrap()
            .unwrap(),
        rev
    );
}

pub(super) async fn new_note_retrieve(store: impl NoteStore<PlainNote>) {
    let note_inner = PlainNote::new("Foo".into());
    let loc = store
        .new_note(
            "".to_owned(),
            note_inner.clone(),
            NoteMetadataEditable::unchanged(),
        )
        .await
        .unwrap();
    assert_eq!(
        store
            .get_note(&loc.current())
            .await
            .unwrap()
            .get_note_inner(),
        note_inner
    );
    assert_eq!(
        store.get_note(&loc).await.unwrap().get_note_inner(),
        note_inner
    );
}

pub(super) async fn update_note(store: impl NoteStore<PlainNote>) {
    let loc1 = store
        .new_note(
            "".to_owned(),
            PlainNote::new("Foo".into()),
            NoteMetadataEditable::unchanged(),
        )
        .await
        .unwrap();
    let rev1 = loc1.get_revision().unwrap();
    let created1 = store
        .get_note(&loc1.current())
        .await
        .unwrap()
        .get_metadata()
        .created_at;
    let modified1 = store
        .get_note(&loc1.current())
        .await
        .unwrap()
        .get_metadata()
        .modified_at;
    let loc2 = store
        .update_note(
            &loc1,
            None,
            Some(PlainNote::new("Foo1".into())),
            NoteMetadataEditable::unchanged(),
        )
        .await
        .unwrap();
    let rev2 = loc2.get_revision().unwrap();
    assert_ne!(rev1, rev2);
    assert_eq!(
        &store.get_current_revision(&loc1).await.unwrap().unwrap(),
        rev2
    );
    assert_eq!(
        store
            .get_note(&loc1.current())
            .await
            .unwrap()
            .get_note_inner(),
        PlainNote::new("Foo1".into())
    );
    assert_eq!(
        store
            .get_note(&loc1.at_revision(rev2))
            .await
            .unwrap()
            .get_note_inner(),
        PlainNote::new("Foo1".into())
    );
    assert_ne!(
        store
            .get_note(&loc1.at_revision(rev2))
            .await
            .unwrap()
            .get_metadata()
            .modified_at,
        modified1
    );
    assert_eq!(
        store
            .get_note(&loc1.at_revision(rev2))
            .await
            .unwrap()
            .get_metadata()
            .created_at,
        created1
    );
    let revisions = store.get_revisions(&loc1).await.unwrap();
    assert_eq!(revisions.len(), 2);
    assert_eq!(revisions[0].0, rev1.clone());
    assert_eq!(revisions[1].0, rev2.clone());
}

pub(super) async fn add_branch(store: impl NoteStore<PlainNote>) {
    let loc1 = store
        .new_note(
            "".to_owned(),
            PlainNote::new("Branch".into()),
            NoteMetadataEditable::unchanged(),
        )
        .await
        .unwrap();
    let loc2 = store
        .new_note(
            "".to_owned(),
            PlainNote::new("Parent".into()),
            NoteMetadataEditable::unchanged(),
        )
        .await
        .unwrap();
    assert!(!store
        .get_note(&loc2.current())
        .await
        .unwrap()
        .get_branches()
        .contains(loc1.get_id()));
    store
        .add_branch(&loc2.get_id(), loc1.get_id())
        .await
        .unwrap();
    assert!(store
        .get_note(&loc2.current())
        .await
        .unwrap()
        .get_branches()
        .contains(loc1.get_id()));
}

pub(super) async fn delete_note_specific(store: impl NoteStore<PlainNote>) {
    let loc1 = store
        .new_note(
            "".to_owned(),
            PlainNote::new("Note".into()),
            NoteMetadataEditable::unchanged(),
        )
        .await
        .unwrap();
    store.delete_note(&loc1).await.unwrap();
    assert!(is_deleted(&store, &loc1).await.unwrap());
}

pub(super) async fn delete_note_current(store: impl NoteStore<PlainNote>) {
    let loc1 = store
        .new_note(
            "".to_owned(),
            PlainNote::new("Note".into()),
            NoteMetadataEditable::unchanged(),
        )
        .await
        .unwrap();
    store.delete_note(&loc1.current()).await.unwrap();
    assert!(is_deleted(&store, &loc1).await.unwrap());
    assert!(matches!(
        store.get_current_revision(&loc1).await.ok().unwrap(),
        None
    ));
}

pub(super) async fn delete_note_with_branches(store: impl NoteStore<PlainNote>) {
    let loc1 = store
        .new_note(
            "".to_owned(),
            PlainNote::new("Branch".into()),
            NoteMetadataEditable::unchanged(),
        )
        .await
        .unwrap();
    let loc2 = store
        .new_note(
            "".to_owned(),
            PlainNote::new("Parent".into()),
            NoteMetadataEditable::unchanged(),
        )
        .await
        .unwrap();
    store
        .add_branch(&loc2.get_id(), loc1.get_id())
        .await
        .unwrap();
    assert!(store
        .get_note(&loc2.current())
        .await
        .unwrap()
        .get_branches()
        .contains(loc1.get_id()));
    assert!(matches!(
        store.delete_note(&loc2.current()).await,
        Err(NoteStoreError::HasBranches(_))
    ));
}

pub(super) async fn resurrect_deleted_note(store: impl NoteStore<PlainNote>) {
    let loc1 = store
        .new_note(
            "".to_owned(),
            PlainNote::new("Foo".into()),
            NoteMetadataEditable::unchanged(),
        )
        .await
        .unwrap();
    let loc2 = store
        .update_note(
            &loc1,
            None,
            Some(PlainNote::new("Foo1".into())),
            NoteMetadataEditable::unchanged(),
        )
        .await
        .unwrap();
    store.delete_note(&loc1.current()).await.unwrap();
    let revisions = store.get_revisions(&loc1).await.unwrap();
    let (last_revision, last_note) = revisions.last().unwrap();
    let last_inner = last_note.get_note_inner();
    assert_eq!(last_inner, PlainNote::new("Foo1".into()));
    assert_eq!(last_revision, loc2.get_revision().unwrap());
    store
        .update_note(
            &NoteLocator::Specific(loc1.get_id().clone(), last_revision.clone()),
            None,
            Some(last_inner),
            NoteMetadataEditable::unchanged(),
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

pub(super) async fn delete_middle_note_sequence(store: impl NoteStore<PlainNote>) {
    let loc1 = store
        .new_note(
            "".to_owned(),
            PlainNote::new("Tail".into()),
            NoteMetadataEditable::unchanged(),
        )
        .await
        .unwrap();
    let loc2 = store
        .new_note(
            "".to_owned(),
            PlainNote::new("Middle".into()),
            NoteMetadataEditable::unchanged(),
        )
        .await
        .unwrap();
    let loc3 = store
        .new_note(
            "".to_owned(),
            PlainNote::new("Head".into()),
            NoteMetadataEditable::unchanged(),
        )
        .await
        .unwrap();
    store
        .append_note(&loc3.get_id(), loc2.get_id())
        .await
        .unwrap();
    store
        .append_note(&loc2.get_id(), loc1.get_id())
        .await
        .unwrap();
    store.delete_note(&loc2.current()).await.unwrap();
    assert_eq!(
        &store
            .get_note(&loc1.current())
            .await
            .unwrap()
            .get_prev()
            .unwrap(),
        loc3.get_id()
    );
    assert_eq!(
        &store
            .get_note(&loc3.current())
            .await
            .unwrap()
            .get_next()
            .unwrap(),
        loc1.get_id()
    );
}

pub(super) async fn resurrect_note_in_sequence(store: impl NoteStore<PlainNote>) {
    let loc1 = store
        .new_note(
            "".to_owned(),
            PlainNote::new("Tail".into()),
            NoteMetadataEditable::unchanged(),
        )
        .await
        .unwrap();
    let loc2 = store
        .new_note(
            "".to_owned(),
            PlainNote::new("Middle".into()),
            NoteMetadataEditable::unchanged(),
        )
        .await
        .unwrap();
    let loc3 = store
        .new_note(
            "".to_owned(),
            PlainNote::new("Head".into()),
            NoteMetadataEditable::unchanged(),
        )
        .await
        .unwrap();
    store
        .append_note(&loc3.get_id(), loc2.get_id())
        .await
        .unwrap();
    store
        .append_note(&loc2.get_id(), loc1.get_id())
        .await
        .unwrap();
    store.delete_note(&loc2.current()).await.unwrap();
    assert_eq!(
        &store
            .get_note(&loc1.current())
            .await
            .unwrap()
            .get_prev()
            .unwrap(),
        loc3.get_id()
    );
    assert_eq!(
        &store
            .get_note(&loc3.current())
            .await
            .unwrap()
            .get_next()
            .unwrap(),
        loc1.get_id()
    );
    assert!(matches!(
        &store.get_current_revision(&loc2).await.ok().unwrap(),
        None
    ));

    let revisions = store.get_revisions(&loc2).await.unwrap();
    let (last_revision, last_note) = revisions.last().unwrap();
    let last_inner = last_note.get_note_inner();
    assert_eq!(last_inner, PlainNote::new("Middle".into()));
    store
        .update_note(
            &NoteLocator::Specific(loc2.get_id().clone(), last_revision.clone()),
            None,
            Some(last_inner),
            NoteMetadataEditable::unchanged(),
        )
        .await
        .unwrap();
    assert_eq!(
        store
            .get_note(&loc2.current())
            .await
            .unwrap()
            .get_note_inner(),
        PlainNote::new("Middle".into())
    );
    assert_eq!(
        store.get_note(&loc2.current()).await.unwrap().get_next(),
        None
    );
}
