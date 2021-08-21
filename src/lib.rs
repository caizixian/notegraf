//! A graph-oriented notebook
pub mod note;
pub mod notestore;
pub mod notetype;
pub mod url;

pub use note::{Note, NoteID, Revision, Tag};
pub use notestore::{InMemoryStore, NoteStore};
pub use notetype::{NoteType, PlainNote};
