//! Notegraf: a graph-oriented notebook.
pub mod errors;
pub mod note;
pub mod notemetadata;
pub mod notestore;
pub mod notetype;
pub mod url;

pub use note::{Note, NoteID, NoteLocator, Revision};
pub use notestore::NoteStore;
pub use notetype::{MarkdownNote, NoteType, PlainNote};
