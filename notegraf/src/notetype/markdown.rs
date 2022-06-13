use crate::url::NotegrafURL;
use crate::{NoteID, NoteType};
use pulldown_cmark::Tag as PTag;
use pulldown_cmark::{Event, LinkType, Options, Parser};
use pulldown_cmark_to_cmark::cmark;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashSet;
use std::fmt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MarkdownNoteError {
    #[error("format error")]
    FormatError(#[from] fmt::Error),
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct MarkdownNote {
    body: String,
}

impl Serialize for MarkdownNote {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.body)
    }
}

struct StringVisitor;

impl<'de> de::Visitor<'de> for StringVisitor {
    type Value = MarkdownNote;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(MarkdownNote { body: v.to_owned() })
    }
}

impl<'de> Deserialize<'de> for MarkdownNote {
    fn deserialize<D>(deserializer: D) -> Result<MarkdownNote, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(StringVisitor)
    }
}

impl MarkdownNote {
    pub fn new(body: String) -> Self {
        MarkdownNote { body }
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

impl NoteType for MarkdownNote {
    type Error = MarkdownNoteError;

    fn get_referents(&self) -> HashSet<NoteID> {
        todo!()
    }

    fn update_referent(
        &mut self,
        old_referent: NoteID,
        new_referent: NoteID,
    ) -> Result<(), Self::Error> {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TASKLISTS);
        options.insert(Options::ENABLE_SMART_PUNCTUATION);
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
