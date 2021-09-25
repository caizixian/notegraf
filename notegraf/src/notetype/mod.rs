use crate::{NoteID, Tag};
use serde::de::DeserializeOwned;
use serde::Serialize;

mod plain;
pub use plain::PlainNote;
mod markdown;
pub use markdown::MarkdownNote;

pub trait NoteType: Serialize + DeserializeOwned + Clone + PartialEq + Eq {
    type Error;
    fn get_references(&self) -> Vec<&NoteID>;
    fn get_tags(&self) -> Vec<&Tag>;
    fn update_reference(
        &mut self,
        old_referent: NoteID,
        new_referent: NoteID,
    ) -> Result<(), Self::Error>;
}
