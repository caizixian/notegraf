use crate::notetype::NoteType;
use std::time::SystemTime;

pub(super) type NoteID = String;
pub(super) type Tag = String;

#[derive(Debug)]
pub struct Note<T: NoteType> {
    note: T,
    id: NoteID,
    created_at: SystemTime,
    modified_at: SystemTime,
}

impl<T: NoteType> Note<T> {
    pub fn new(note: T, id: NoteID) -> Self {
        Note {
            note,
            id,
            created_at: SystemTime::now(),
            modified_at: SystemTime::now(),
        }
    }
}
