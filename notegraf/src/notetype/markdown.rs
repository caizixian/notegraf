use crate::url::NotegrafURL;
use crate::{NoteID, NoteType};
use pulldown_cmark::Tag as PTag;
use pulldown_cmark::{Event, LinkType, Options, Parser};
use pulldown_cmark_to_cmark::cmark;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MarkdownNoteError {
    #[error("format error")]
    FormatError(#[from] fmt::Error),
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(into = "String", from = "String")]
pub struct MarkdownNote {
    body: String,
}

impl From<String> for MarkdownNote {
    fn from(note: String) -> MarkdownNote {
        MarkdownNote::new(note)
    }
}

impl From<&str> for MarkdownNote {
    fn from(note: &str) -> MarkdownNote {
        MarkdownNote::new(note.to_owned())
    }
}

impl From<MarkdownNote> for String {
    fn from(note: MarkdownNote) -> String {
        note.body
    }
}

impl MarkdownNote {
    pub fn new(body: String) -> Self {
        MarkdownNote { body }
    }

    fn extract_note_id_from_url(link: &str) -> Option<NoteID> {
        let url = NotegrafURL::parse(link);
        if let Ok(NotegrafURL::Note(ref id)) = url {
            Some(id.clone())
        } else {
            None
        }
    }

    fn change_note_url(link: &str, old: &NoteID, new: &NoteID) -> Option<String> {
        let url = NotegrafURL::parse(link);
        if let Ok(NotegrafURL::Note(ref id)) = url {
            if id == old {
                Some(format!("{}", NotegrafURL::Note(new.clone())))
            } else {
                None
            }
        } else {
            None
        }
    }
}

fn cmark_options() -> Options {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_SMART_PUNCTUATION);
    options
}

impl NoteType for MarkdownNote {
    type Error = MarkdownNoteError;

    fn get_referents(&self) -> Result<HashSet<NoteID>, Self::Error> {
        let options = cmark_options();
        let mut referents = HashSet::new();
        let parser = Parser::new_ext(&self.body, options);
        for event in parser {
            if let Event::Start(PTag::Link(_linktype, destination, _title)) = event {
                if let Some(id) = MarkdownNote::extract_note_id_from_url(&destination) {
                    referents.insert(id);
                }
            }
        }
        Ok(referents)
    }

    fn update_referent(
        &mut self,
        old_referent: NoteID,
        new_referent: NoteID,
    ) -> Result<(), Self::Error> {
        let options = cmark_options();
        let mut buf = String::new();
        let mut change_autolink_text = false;
        let mut old_autolink = None;
        let mut new_autolink = None;
        let parser = Parser::new_ext(&self.body, options).map(|event| match event {
            Event::Start(PTag::Link(linktype, destination, title)) => {
                let new_destination =
                    MarkdownNote::change_note_url(&destination, &old_referent, &new_referent);
                if let Some(l) = new_destination {
                    if linktype == LinkType::Autolink {
                        change_autolink_text = true;
                        old_autolink = Some(destination.clone().into_string());
                        new_autolink = Some(l.clone());
                    }
                    Event::Start(PTag::Link(linktype, l.into(), title))
                } else {
                    Event::Start(PTag::Link(linktype, destination, title))
                }
            }
            Event::Text(text) => {
                if change_autolink_text {
                    Event::Text(
                        text.replace(
                            old_autolink.as_ref().unwrap(),
                            new_autolink.as_ref().unwrap(),
                        )
                        .into(),
                    )
                } else {
                    Event::Text(text)
                }
            }
            Event::End(PTag::Link(linktype, destination, title)) => {
                let new_destination =
                    MarkdownNote::change_note_url(&destination, &old_referent, &new_referent);
                if let Some(l) = new_destination {
                    if linktype == LinkType::Autolink {
                        change_autolink_text = false;
                        old_autolink = None;
                        new_autolink = None;
                    }
                    Event::End(PTag::Link(linktype, l.into(), title))
                } else {
                    Event::End(PTag::Link(linktype, destination, title))
                }
            }

            _ => event,
        });
        match cmark(parser, &mut buf) {
            Ok(_) => {
                self.body = buf;
                Ok(())
            }
            Err(e) => Err(MarkdownNoteError::FormatError(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewrite_url() {
        let id_old = NoteID::new("old".into());
        let id_new = NoteID::new("new".into());
        let old_url = "notegraf:/note/old";
        let new_url = MarkdownNote::change_note_url(old_url, &id_old, &id_new);
        assert_eq!(new_url.unwrap(), "notegraf:/note/new");
        assert_eq!(
            MarkdownNote::change_note_url("notegraf:/note/foo", &id_old, &id_new),
            None
        );
        assert_eq!(
            MarkdownNote::change_note_url("notegraf:/tag/old", &id_old, &id_new),
            None
        );
        assert_eq!(
            MarkdownNote::change_note_url("notegraf:/note/old/bar", &id_old, &id_new),
            None
        );
        assert_eq!(
            MarkdownNote::change_note_url("http://host/note/old", &id_old, &id_new),
            None
        );
    }

    #[test]
    fn referent_markdown_link() {
        let note = MarkdownNote::new(r#"[foo](notegraf:/note/note-1)"#.into());
        let referents = note.get_referents().unwrap();
        assert_eq!(referents.len(), 1);
        assert_eq!(
            referents.iter().next().unwrap(),
            &NoteID::new("note-1".to_owned())
        );
    }

    #[test]
    fn referent_markdown_autolink() {
        let note = MarkdownNote::new(r#"<notegraf:/note/note-1>"#.into());
        let referents = note.get_referents().unwrap();
        assert_eq!(referents.len(), 1);
        assert_eq!(
            referents.iter().next().unwrap(),
            &NoteID::new("note-1".to_owned())
        );
    }

    #[test]
    fn rewrite_markdown_link() {
        let id_old = NoteID::new("old".into());
        let id_new = NoteID::new("new".into());
        let mut note = MarkdownNote::new(r#"[foo](notegraf:/note/old)"#.into());
        note.update_referent(id_old, id_new).unwrap();
        assert_eq!(note.body, r#"[foo](notegraf:/note/new)"#)
    }

    #[test]
    fn rewrite_markdown_autolink() {
        let id_old = NoteID::new("old".into());
        let id_new = NoteID::new("new".into());
        let mut note = MarkdownNote::new(r#"<notegraf:/note/old>"#.into());
        note.update_referent(id_old, id_new).unwrap();
        assert_eq!(note.body, r#"<notegraf:/note/new>"#)
    }

    #[test]
    fn serialize() {
        let ser = serde_json::to_string(&MarkdownNote {
            body: "Hello, world!".to_owned(),
        })
        .unwrap();
        assert_eq!(ser, "\"Hello, world!\"");
    }

    #[test]
    fn deserialize() {
        let note: MarkdownNote = serde_json::from_str("\"Hello, world!\"").unwrap();
        assert_eq!(note.body, "Hello, world!");
    }
}
