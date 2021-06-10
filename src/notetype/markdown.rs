use crate::{NoteID, NoteType, Tag};
use pulldown_cmark::{Options, Parser};
use pulldown_cmark_to_cmark::cmark;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MarkdownNoteError {
    #[error("format error")]
    FormatError(#[from] std::fmt::Error),
}

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
        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TASKLISTS);
        options.insert(Options::ENABLE_SMART_PUNCTUATION);
        let mut buf = String::new();
        let parser = Parser::new_ext(&self.body, options).map(|event| match event {
            _ => event,
        });
        match cmark(parser, &mut buf, None) {
            Ok(_) => {
                self.body = buf;
                Ok(())
            }
            Err(e) => Err(MarkdownNoteError::FormatError(e)),
        }
    }
}
