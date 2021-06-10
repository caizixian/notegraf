use crate::{NoteID, NoteType, Tag};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MarkdownNoteError {}

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq, Eq)]
pub struct MarkdownNote {
    body: String,
    tags: HashSet<Tag>,
}

impl MarkdownNote {
    pub fn new(body: String) -> Self {
        MarkdownNote {
            body,
            ..Default::default()
        }
    }

    pub fn add_tag(&mut self, tag: Tag) {
        self.tags.insert(tag);
    }
}

impl NoteType for MarkdownNote {
    type Error = MarkdownNoteError;

    fn get_references(&self) -> Vec<&NoteID> {
        todo!()
    }

    fn get_tags(&self) -> Vec<&Tag> {
        self.tags.iter().collect()
    }

    fn update_reference(
        &mut self,
        _old_referent: NoteID,
        _new_referent: NoteID,
    ) -> Result<(), Self::Error> {
        todo!()
    }
}
