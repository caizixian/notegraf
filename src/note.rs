use crate::notetype::NoteType;
use std::time::SystemTime;

pub(super) type NoteID = String;
pub(super) type Tag = String;

pub struct Note<T: NoteType> {
    note: T,
    created_at: SystemTime,
    modified_at: SystemTime,
}

impl<T: NoteType> Note<T> {
    pub fn new(note: T) -> Self {
        Note {
            note,
            created_at: SystemTime::now(),
            modified_at: SystemTime::now(),
        }
    }
}
