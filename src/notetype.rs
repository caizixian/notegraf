use crate::note::{NoteID, Tag};
use std::convert::{TryFrom, TryInto};

pub trait NoteType: TryFrom<String> + TryInto<String> {
    fn get_children(&self) -> Vec<NoteID>;
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
    fn get_children(&self) -> Vec<String> {
        vec![]
    }
    fn get_tags(&self) -> Vec<String> {
        vec![]
    }
}

impl TryFrom<String> for PlainNote {
    type Error = &'static str;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(PlainNote::new(value))
    }
}

impl TryInto<String> for PlainNote {
    type Error = &'static str;
    fn try_into(self) -> Result<String, Self::Error> {
        Ok(self.body)
    }
}
