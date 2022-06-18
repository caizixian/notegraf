use crate::errors::NoteStoreError;
use crate::{NoteStore, PlainNote};
use std::option::Option::None;

pub(super) async fn unique_id(store: impl NoteStore<PlainNote>) {
    let loc1 = store
        .new_note("".to_owned(), PlainNote::new("Foo".into()), None)
        .await
        .unwrap();
    let loc2 = store
        .new_note("".to_owned(), PlainNote::new("Bar".into()), None)
        .await
        .unwrap();
    assert_ne!(loc1.get_id(), loc2.get_id());
}

pub(super) async fn new_note_revision(store: impl NoteStore<PlainNote>) {
    let loc = store
        .new_note("".to_owned(), PlainNote::new("Foo".into()), None)
        .await
        .unwrap();
    let rev = loc.get_revision().unwrap();
    assert_eq!(
        &store.get_current_revision(&loc.current()).await.unwrap(),
        rev
    );
}

pub(super) async fn new_note_retrieve(store: impl NoteStore<PlainNote>) {
    let note_inner = PlainNote::new("Foo".into());
    let loc = store
        .new_note("".to_owned(), note_inner.clone(), None)
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
        .new_note("".to_owned(), PlainNote::new("Foo".into()), None)
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
        .update_note(&loc1, None, Some(PlainNote::new("Foo1".into())), None)
        .await
        .unwrap();
    let rev2 = loc2.get_revision().unwrap();
    assert_ne!(rev1, rev2);
    assert_eq!(&store.get_current_revision(&loc1).await.unwrap(), rev2);
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
    assert_eq!(
        store.get_revisions(&loc1).await.unwrap(),
        vec![rev2.clone(), rev1.clone()]
    )
}

pub(super) async fn add_branch(store: impl NoteStore<PlainNote>) {
    let loc1 = store
        .new_note("".to_owned(), PlainNote::new("Branch".into()), None)
        .await
        .unwrap();
    let loc2 = store
        .new_note("".to_owned(), PlainNote::new("Parent".into()), None)
        .await
        .unwrap();
    store.add_branch(&loc2, loc1.get_id()).await.unwrap();
    assert!(!store
        .get_note(&loc2) // This points to an old revision
        .await
        .unwrap()
        .get_branches()
        .contains(loc1.get_id()));
    assert!(store
        .get_note(&loc2.current())
        .await
        .unwrap()
        .get_branches()
        .contains(loc1.get_id()));
}

pub(super) async fn delete_note_with_branches(store: impl NoteStore<PlainNote>) {
    let loc1 = store
        .new_note("".to_owned(), PlainNote::new("Branch".into()), None)
        .await
        .unwrap();
    let loc2 = store
        .new_note("".to_owned(), PlainNote::new("Parent".into()), None)
        .await
        .unwrap();
    store.add_branch(&loc2, loc1.get_id()).await.unwrap();
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

pub(super) async fn delete_middle_note_sequence(store: impl NoteStore<PlainNote>) {
    let loc1 = store
        .new_note("".to_owned(), PlainNote::new("Tail".into()), None)
        .await
        .unwrap();
    let loc2 = store
        .new_note("".to_owned(), PlainNote::new("Middle".into()), None)
        .await
        .unwrap();
    let loc3 = store
        .new_note("".to_owned(), PlainNote::new("Head".into()), None)
        .await
        .unwrap();
    store.append_note(&loc3, loc2.get_id()).await.unwrap();
    store.append_note(&loc2, loc1.get_id()).await.unwrap();
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
