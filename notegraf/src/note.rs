//! Core types of Notegraf.
use crate::errors::NoteStoreError;
use crate::notemetadata::NoteMetadata;
use crate::notetype::NoteType;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};
use std::collections::HashSet;
use std::fmt::{self, Debug, Display};
use uuid::Uuid;

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

impl From<Uuid> for NoteID {
    fn from(id: Uuid) -> NoteID {
        NoteID::new(id.to_string())
    }
}

impl From<&Uuid> for NoteID {
    fn from(id: &Uuid) -> NoteID {
        NoteID::new(id.to_string())
    }
}

impl NoteID {
    pub fn new(id: String) -> Self {
        NoteID { id }
    }

    pub fn to_uuid(&self) -> Option<Uuid> {
        Uuid::parse_str(&self.id).ok()
    }

    pub fn try_to_uuid(&self) -> Result<Uuid, NoteStoreError> {
        self.to_uuid()
            .ok_or_else(|| NoteStoreError::NotUuid(self.id.clone()))
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

    pub fn to_uuid(&self) -> Option<Uuid> {
        Uuid::parse_str(&self.revision).ok()
    }

    pub fn try_to_uuid(&self) -> Result<Uuid, NoteStoreError> {
        self.to_uuid()
            .ok_or_else(|| NoteStoreError::NotUuid(self.revision.clone()))
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

impl From<&Uuid> for Revision {
    fn from(revision: &Uuid) -> Revision {
        Revision::new(revision.to_string())
    }
}

impl From<Uuid> for Revision {
    fn from(revision: Uuid) -> Revision {
        Revision::new(revision.to_string())
    }
}

/// Represent a complete note entity for downstream consumption
///
/// Note properties can be stored as is by the storage backend, but can also be computed.
/// Expensive computation can be cached, but it's the storage's responsibility to keep the cache
/// coherent.
pub trait Note<T: NoteType>: Debug {
    fn get_title(&self) -> String;
    fn get_note_inner(&self) -> T;
    fn get_id(&self) -> NoteID;
    fn get_revision(&self) -> Revision;
    // Represent branches
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

pub enum NoteField {
    Title,
    NoteInner,
    ID,
    Revision,
    Parent,
    Branches,
    Prev,
    Next,
    References,
    Referents,
    Metadata,
}

pub struct NoteFieldSelector {
    fields: Vec<NoteField>,
}

impl NoteFieldSelector {
    pub fn new(fields: Vec<NoteField>) -> Self {
        NoteFieldSelector { fields }
    }
}

impl Default for NoteFieldSelector {
    fn default() -> Self {
        NoteFieldSelector::new(vec![
            NoteField::Title,
            NoteField::NoteInner,
            NoteField::ID,
            NoteField::Revision,
            NoteField::Parent,
            NoteField::Branches,
            NoteField::Prev,
            NoteField::Next,
            NoteField::References,
            NoteField::Referents,
            NoteField::Metadata,
        ])
    }
}

pub struct NoteSerializable<T> {
    s: NoteFieldSelector,
    n: Box<dyn Note<T>>,
}

impl<T> Serialize for NoteSerializable<T>
where
    T: NoteType,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("Note", self.s.fields.len())?;
        for f in &self.s.fields {
            match f {
                NoteField::Title => {
                    s.serialize_field("title", &self.n.get_title())?;
                }
                NoteField::NoteInner => {
                    s.serialize_field("note_inner", &self.n.get_note_inner())?;
                }
                NoteField::ID => {
                    s.serialize_field("id", &self.n.get_id())?;
                }
                NoteField::Revision => {
                    s.serialize_field("revision", &self.n.get_revision())?;
                }
                NoteField::Parent => {
                    s.serialize_field("parent", &self.n.get_parent())?;
                }
                NoteField::Branches => {
                    s.serialize_field("branches", &self.n.get_branches())?;
                }
                NoteField::Prev => {
                    s.serialize_field("prev", &self.n.get_prev())?;
                }
                NoteField::Next => {
                    s.serialize_field("next", &self.n.get_next())?;
                }
                NoteField::References => {
                    s.serialize_field("references", &self.n.get_references())?;
                }
                NoteField::Referents => {
                    s.serialize_field("referents", &self.n.get_referents())?;
                }
                NoteField::Metadata => {
                    s.serialize_field("metadata", &self.n.get_metadata())?;
                }
            }
        }
        s.end()
    }
}

impl<T> NoteSerializable<T> {
    pub fn all_fields(note: Box<dyn Note<T>>) -> Self {
        NoteSerializable {
            s: NoteFieldSelector::default(),
            n: note,
        }
    }
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

    /// Return a ([`Uuid`], [`Uuid`]) tuple.
    pub fn unpack_uuid(&self) -> Result<(Uuid, Option<Uuid>), NoteStoreError> {
        let (id, revision) = self.unpack();
        Ok((
            id.try_to_uuid()?,
            revision.map(|x| x.try_to_uuid()).transpose()?,
        ))
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
