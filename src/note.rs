use crate::notetype::NoteType;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};
use std::time::SystemTime;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone, Hash)]
pub struct NoteID {
    id: String,
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

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
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

#[derive(Debug, Clone)]
pub struct Note<T: NoteType> {
    pub note_inner: T,
    pub id: NoteID,
    pub revision: Revision,
    pub created_at: SystemTime,
    pub modified_at: SystemTime,
}

impl<T: NoteType> Note<T> {
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
