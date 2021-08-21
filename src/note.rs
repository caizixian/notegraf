//! Core types of Notegraf
use crate::notetype::NoteType;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};
use std::time::SystemTime;

/// ID of notes
///
/// In a given note store ([`crate::notestore`]),
/// [`NoteID`] should uniquely identify a note,
/// which can have different revisions ([`Revision`]).
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone, Hash)]
#[serde(into = "String", from = "String")]
pub struct NoteID {
    id: String,
}

impl From<NoteID> for String {
    fn from(id: NoteID) -> String {
        id.id
    }
}

impl From<String> for NoteID {
    fn from(id: String) -> NoteID {
        NoteID::new(id)
    }
}

impl NoteID {
    pub fn new(id: String) -> Self {
        NoteID { id }
    }
}

impl Display for NoteID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl AsRef<str> for NoteID {
    fn as_ref(&self) -> &str {
        &self.id
    }
}

/// Tags of notes
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone, Hash)]
pub struct Tag {
    tag: String,
}

impl Tag {
    pub fn new(tag: String) -> Self {
        Tag { tag }
    }
}

impl Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.tag)
    }
}

impl AsRef<str> for Tag {
    fn as_ref(&self) -> &str {
        &self.tag
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash, Serialize, Deserialize)]
#[serde(into = "String", from = "String")]
pub struct Revision {
    revision: String,
}

impl Revision {
    pub fn new(revision: String) -> Self {
        Revision { revision }
    }
}

impl Display for Revision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.revision)
    }
}

impl AsRef<str> for Revision {
    fn as_ref(&self) -> &str {
        &self.revision
    }
}

impl From<Revision> for String {
    fn from(revision: Revision) -> String {
        revision.revision
    }
}

impl From<String> for Revision {
    fn from(revision: String) -> Revision {
        Revision::new(revision)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Note<T> {
    pub note_inner: T,
    pub id: NoteID,
    pub revision: Revision,
    pub created_at: SystemTime,
    pub modified_at: SystemTime,
}

impl<T> Note<T>
where
    T: NoteType,
{
    pub fn new(note_inner: T, id: NoteID, revision: Revision) -> Self {
        Note {
            note_inner,
            id,
            revision,
            created_at: SystemTime::now(),
            modified_at: SystemTime::now(),
        }
    }
}
