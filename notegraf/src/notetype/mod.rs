use crate::NoteID;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashSet;
use std::fmt::Debug;

mod plain;
pub use plain::PlainNote;
mod markdown;
pub use markdown::MarkdownNote;

pub trait NoteType:
    Serialize + DeserializeOwned + Clone + Debug + PartialEq + Eq + Send + Sync + 'static
{
    type Error;
    fn get_referents(&self) -> HashSet<NoteID>;
    fn update_referent(
        &mut self,
        old_referent: NoteID,
        new_referent: NoteID,
    ) -> Result<(), Self::Error>;
}
