use crate::{NoteID, NoteType, Tag};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq, Eq)]
pub struct PlainNote {
    body: String,
    references: HashSet<NoteID>,
    tags: HashSet<Tag>,
}

impl PlainNote {
    pub fn new(body: String) -> Self {
        PlainNote {
            body,
            ..Default::default()
        }
    }

    pub fn add_reference(&mut self, referent: NoteID) {
        self.references.insert(referent);
    }

    pub fn add_tag(&mut self, tag: Tag) {
        self.tags.insert(tag);
    }
}

impl NoteType for PlainNote {
    fn get_references(&self) -> Vec<&NoteID> {
        self.references.iter().collect()
    }

    fn get_tags(&self) -> Vec<&Tag> {
        self.tags.iter().collect()
    }

    fn update_reference(&mut self, old_referent: NoteID, new_referent: NoteID) {
        self.references.remove(&old_referent);
        self.references.insert(new_referent);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retrieve_references() {
        let mut note = PlainNote::new("Foo".into());
        note.add_reference(NoteID::new("ID1".into()));
        note.add_reference(NoteID::new("ID2".into()));
        assert!(note.get_references().contains(&&NoteID::new("ID1".into())));
        assert!(note.get_references().contains(&&NoteID::new("ID2".into())));
    }

    #[test]
    fn update_references() {
        let mut note = PlainNote::new("Foo".into());
        note.add_reference(NoteID::new("ID1".into()));
        note.add_reference(NoteID::new("ID2".into()));
        note.update_reference(NoteID::new("ID1".into()), NoteID::new("ID3".into()));
        assert!(note.get_references().contains(&&NoteID::new("ID3".into())));
        assert!(note.get_references().contains(&&NoteID::new("ID2".into())));
    }

    #[test]
    fn retrieve_tags() {
        let mut note = PlainNote::new("Foo".into());
        note.add_tag(Tag::new("tag1".into()));
        note.add_tag(Tag::new("tag2".into()));
        assert!(note.get_tags().contains(&&Tag::new("tag1".into())));
        assert!(note.get_tags().contains(&&Tag::new("tag2".into())));
    }

    #[test]
    fn dedup_tags() {
        let mut note = PlainNote::new("Foo".into());
        note.add_tag(Tag::new("tag1".into()));
        note.add_tag(Tag::new("tag2".into()));
        note.add_tag(Tag::new("tag2".into()));
        assert_eq!(note.get_tags().len(), 2);
    }

    #[test]
    fn dedup_references() {
        let mut note = PlainNote::new("Foo".into());
        note.add_reference(NoteID::new("ID1".into()));
        note.add_reference(NoteID::new("ID2".into()));
        note.add_reference(NoteID::new("ID2".into()));
        assert_eq!(note.get_references().len(), 2);
    }
}
