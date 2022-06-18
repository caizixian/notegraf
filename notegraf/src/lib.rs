//! Notegraf: a graph-oriented notebook.
#[macro_use]
#[allow(unused_imports)]
extern crate tracing;

pub mod errors;
pub mod note;
pub mod notemetadata;
pub mod notestore;
pub mod notetype;
pub mod url;

pub use note::{Note, NoteID, NoteLocator, NoteSerializable, Revision};
pub use notestore::{InMemoryStore, NoteStore};
pub use notetype::{MarkdownNote, NoteType, PlainNote};
