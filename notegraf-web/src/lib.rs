#[macro_use]
extern crate tracing;
#[macro_use]
extern crate lazy_static;

pub mod configuration;
pub mod routes;
pub mod startup;
pub mod telemetry;

#[cfg(feature = "notetype_plain")]
pub type NoteType = notegraf::PlainNote;
#[cfg(feature = "notetype_markdown")]
pub type NoteType = notegraf::MarkdownNote;
