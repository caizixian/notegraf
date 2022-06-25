use thiserror::Error;
use url::ParseError;

use crate::{NoteID, Revision};

#[derive(Error, Debug)]
pub enum NoteStoreError {
    #[error("note `{0}` doesn't exist")]
    NoteNotExist(NoteID),
    #[error("note `{0}` is deleted, revision needed if resurrecting a deleted note")]
    NoteDeleted(NoteID),
    #[error("note `{0}` already exists")]
    NoteIDConflict(NoteID),
    #[error("revision`{1}` of note `{0}` doesn't exist")]
    RevisionNotExist(NoteID, Revision),
    #[error("io error")]
    IOError(#[from] std::io::Error),
    #[error("serde error")]
    SerdeError(#[from] serde_json::Error),
    #[error("attempt to update non-current revision `{1}` of note `{0}`")]
    UpdateOldRevision(NoteID, Revision),
    #[error("attempt to delete non-current revision `{1}` of note `{0}`")]
    DeleteOldRevision(NoteID, Revision),
    #[error("inconsistency detected: note `{1}` is not a child of note `{0}`")]
    NotAChild(NoteID, NoteID),
    #[error("cannot append note `{1}` to note `{0}`, because note `{0}` is not the last note in a sequence")]
    ExistingNext(NoteID, NoteID),
    #[error("cannot delete note `{0}`, because it has branches")]
    HasBranches(NoteID),
    #[error("cannot delete note `{0}`, because other notes refer to it")]
    HasReferences(NoteID),
    #[error("note cannot be parsed: `{0}`")]
    ParseError(String),
    #[error("PostgreSQL error")]
    PostgreSQLError(#[from] sqlx::Error),
    #[error("error processing note inner")]
    NoteInnerError(String),
}

/// Error type for Notegraf URL parsing.
#[derive(Error, Debug)]
pub enum URLParseError {
    /// Not a valid URL.
    ///
    /// That is, it can't be parse by the `Url` library.
    #[error("Not a valid URL")]
    NotAURL(#[from] ParseError),
    /// Wrong URL scheme, such as HTTP.
    #[error("URL scheme `{0}` not supported")]
    WrongScheme(String),
    /// The URL cannot be a base, such as a base64 encoded image.
    #[error("The URL cannot be a base")]
    CannotBeABase,
    /// Not a valid Notegraf URL.
    ///
    /// For example, the first part of the URL might not have any of the expected value.
    #[error("Syntax error: `{0}`")]
    SyntaxError(String),
}
