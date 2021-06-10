use crate::{NoteID, Tag};
use serde::de::DeserializeOwned;
use serde::Serialize;

mod plain;
pub use plain::PlainNote;

pub trait NoteType: Serialize + DeserializeOwned {
    fn get_references(&self) -> &Vec<NoteID>;
    fn get_tags(&self) -> &Vec<Tag>;
}
