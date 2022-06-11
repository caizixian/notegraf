use std::time::SystemTime;
use serde::{Deserialize, Serialize};

pub static NOTE_METADATA_CURRENT_SCHEMA_VERSION: u64 = 0;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NoteMetadata {
    pub schema_version: u64,
    pub created_at: SystemTime,
    pub modified_at: SystemTime,
}

impl Default for NoteMetadata {
    fn default() -> Self {
        let now = SystemTime::now();
        NoteMetadata {
            schema_version: NOTE_METADATA_CURRENT_SCHEMA_VERSION,
            created_at: now,
            modified_at: now
        }
    }
}

impl NoteMetadata {
    pub(crate) fn on_update_note(&self) -> Self{
        // We cannot update the metadata if it's based on a newer schema
        assert!(NOTE_METADATA_CURRENT_SCHEMA_VERSION >= self.schema_version);
        NoteMetadata {
            schema_version: NOTE_METADATA_CURRENT_SCHEMA_VERSION,
            created_at: self.created_at,
            modified_at: SystemTime::now()
        }
    }
}
