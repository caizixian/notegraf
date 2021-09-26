//! Notegraf: a graph-oriented notebook.
pub mod note;
pub mod notestore;
pub mod notetype;
pub mod url;

pub use note::{Note, NoteID, NoteLocator, Revision, Tag};
pub use notestore::NoteStore;
pub use notetype::{NoteType, PlainNote};
