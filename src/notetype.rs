use crate::note::{NoteID, Tag};

pub trait NoteType {
    fn get_references(&self) -> Vec<NoteID>;
    fn get_tags(&self) -> Vec<Tag>;
}

struct PlainNote {
    body: String,
}

impl PlainNote {
    fn new(body: String) -> Self {
        PlainNote { body }
    }
}

impl NoteType for PlainNote {
    fn get_references(&self) -> Vec<NoteID> {
        vec![]
    }
    fn get_tags(&self) -> Vec<Tag> {
        vec![]
    }
}
