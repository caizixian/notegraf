use crate::notetype::NoteType;

pub(super) type NoteID = String;
pub(super) type Tag = String;

pub struct Note<T: NoteType> {
    note: T
}

impl<T: NoteType> Note<T> {
    pub fn new(note: T) {
        Note { note };
    }
}
