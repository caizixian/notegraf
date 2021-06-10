use crate::{NoteID, NoteType, Tag};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq, Eq)]
pub struct PlainNote {
    body: String,
    references: Vec<NoteID>,
    tags: Vec<Tag>,
}

impl PlainNote {
    pub fn new(body: String) -> Self {
        PlainNote {
            body,
            ..Default::default()
        }
    }

    pub fn add_reference(&mut self, referent: NoteID) {
        self.references.push(referent);
    }

    pub fn add_tag(&mut self, tag: Tag) {
        self.tags.push(tag);
    }
}

impl NoteType for PlainNote {
    fn get_references(&self) -> &Vec<NoteID> {
        &self.references
    }
    fn get_tags(&self) -> &Vec<Tag> {
        &self.tags
    }
}
