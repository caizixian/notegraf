use crate::{NoteID, NoteType};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PlainNoteError {
    #[error("this note doesn't refer to `{0}`")]
    ReferenceNotExist(NoteID),
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq, Eq)]
pub struct PlainNote {
    body: String,
    references: HashSet<NoteID>,
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

    pub fn split_off(mut self, at: usize) -> (Self, Self) {
        let new_body = self.body.split_off(at);
        (self, PlainNote::new(new_body))
    }

    pub fn merge(mut self, other: Self) -> Self {
        self.body.push_str(&other.body);
        self.references.extend(other.references);
        self
    }
}

impl NoteType for PlainNote {
    type Error = PlainNoteError;

    fn get_referents(&self) -> Result<HashSet<NoteID>, Self::Error> {
        Ok(self.references.clone())
    }

    fn update_referent(
        &mut self,
        old_referent: NoteID,
        new_referent: NoteID,
    ) -> Result<(), Self::Error> {
        if !self.references.contains(&old_referent) {
            return Err(Self::Error::ReferenceNotExist(old_referent));
        }
        self.references.remove(&old_referent);
        self.references.insert(new_referent);
        Ok(())
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
        assert!(note
            .get_referents()
            .unwrap()
            .contains(&&NoteID::new("ID1".into())));
        assert!(note
            .get_referents()
            .unwrap()
            .contains(&&NoteID::new("ID2".into())));
    }

    #[test]
    fn update_references() {
        let mut note = PlainNote::new("Foo".into());
        note.add_reference(NoteID::new("ID1".into()));
        note.add_reference(NoteID::new("ID2".into()));
        note.update_referent(NoteID::new("ID1".into()), NoteID::new("ID3".into()))
            .unwrap();
        assert!(note
            .get_referents()
            .unwrap()
            .contains(&&NoteID::new("ID3".into())));
        assert!(note
            .get_referents()
            .unwrap()
            .contains(&&NoteID::new("ID2".into())));
    }

    #[test]
    fn dedup_references() {
        let mut note = PlainNote::new("Foo".into());
        note.add_reference(NoteID::new("ID1".into()));
        note.add_reference(NoteID::new("ID2".into()));
        note.add_reference(NoteID::new("ID2".into()));
        assert_eq!(note.get_referents().unwrap().len(), 2);
    }
}
