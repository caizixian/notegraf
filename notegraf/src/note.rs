//! Core types of Notegraf.
use crate::notemetadata::NoteMetadata;
use crate::notetype::NoteType;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt::{self, Debug, Display};

/// ID of notes.
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

impl From<&str> for NoteID {
    fn from(id: &str) -> NoteID {
        NoteID::new(id.to_owned())
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

impl From<&str> for Revision {
    fn from(revision: &str) -> Revision {
        Revision::new(revision.to_owned())
    }
}

/// Represent a complete note entity for downstream consumption
///
/// Note properties can be stored as is by the storage backend, but can also be computed.
/// Expensive computation can be cached, but it's the storage's responsibility to keep the cache
/// coherent.
pub trait Note<T: NoteType>: Debug + erased_serde::Serialize {
    fn get_note_inner(&self) -> T;
    fn get_id(&self) -> NoteID;
    fn get_revision(&self) -> Revision;
    fn get_parent(&self) -> Option<NoteID>;
    fn get_branches(&self) -> HashSet<NoteID>;
    // Represents a sequence of notes
    fn get_prev(&self) -> Option<NoteID>;
    fn get_next(&self) -> Option<NoteID>;
    // Represents bi-directional references
    fn get_references(&self) -> HashSet<NoteID>;
    fn get_referents(&self) -> HashSet<NoteID>;
    fn get_metadata(&self) -> NoteMetadata;
}

/// A type for locating a note.
#[derive(Debug, Serialize, Deserialize)]
pub enum NoteLocator {
    Current(NoteID),
    Specific(NoteID, Revision),
}

impl NoteLocator {
    /// Get ID of the locator.
    pub fn get_id(&self) -> &NoteID {
        match self {
            NoteLocator::Current(id) => id,
            NoteLocator::Specific(id, _) => id,
        }
    }

    /// Get revision of the locator.
    ///
    /// If the locator is not specifying a revision, returns `None`.
    pub fn get_revision(&self) -> Option<&Revision> {
        if let NoteLocator::Specific(_, rev) = self {
            Some(rev)
        } else {
            None
        }
    }

    /// Return a ([`NoteID`], [`Revision`]) tuple.
    pub fn unpack(&self) -> (&NoteID, Option<&Revision>) {
        (self.get_id(), self.get_revision())
    }

    /// Get a locator to point at a specific revision of the same note.
    pub fn at_revision(&self, r: &Revision) -> Self {
        NoteLocator::Specific(self.get_id().clone(), r.clone())
    }

    /// Get a locator to point at the current revision of the same note.
    pub fn current(&self) -> Self {
        NoteLocator::Current(self.get_id().clone())
    }
}
