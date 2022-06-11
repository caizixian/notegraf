use crate::NoteID;
use serde::de::DeserializeOwned;
use serde::Serialize;

mod plain;
pub use plain::PlainNote;
mod markdown;
pub use markdown::MarkdownNote;

pub trait NoteType: Serialize + DeserializeOwned + Clone + PartialEq + Eq + Send + Sync {
    type Error;
    fn get_references(&self) -> Vec<&NoteID>;
    fn update_reference(
        &mut self,
        old_referent: NoteID,
        new_referent: NoteID,
    ) -> Result<(), Self::Error>;
}
