use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub static NOTE_METADATA_CURRENT_SCHEMA_VERSION: u64 = 0;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NoteMetadata {
    pub schema_version: u64,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub tags: HashSet<String>,
    pub custom_metadata: serde_json::Value,
}

impl Default for NoteMetadata {
    fn default() -> Self {
        let now = Utc::now();
        NoteMetadata {
            schema_version: NOTE_METADATA_CURRENT_SCHEMA_VERSION,
            created_at: now,
            modified_at: now,
            tags: HashSet::new(),
            custom_metadata: serde_json::json!({}),
        }
    }
}

impl NoteMetadata {
    pub(crate) fn on_update_note(&self) -> Self {
        // We cannot update the metadata if it's based on a newer schema
        assert!(NOTE_METADATA_CURRENT_SCHEMA_VERSION >= self.schema_version);
        NoteMetadata {
            schema_version: NOTE_METADATA_CURRENT_SCHEMA_VERSION,
            created_at: self.created_at,
            modified_at: Utc::now(),
            tags: self.tags.clone(),
            custom_metadata: self.custom_metadata.clone(),
        }
    }

    pub fn from_editable(m: NoteMetadataEditable) -> Self {
        let mut nm = NoteMetadata::default();
        m.apply(&mut nm);
        nm
    }

    pub fn apply_editable(mut self, m: NoteMetadataEditable) -> Self {
        m.apply(&mut self);
        self
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NoteMetadataEditable {
    pub tags: Option<HashSet<String>>,
    pub custom_metadata: Option<serde_json::Value>,
}

impl NoteMetadataEditable {
    pub fn apply(self, n: &mut NoteMetadata) {
        if let Some(t) = self.tags {
            n.tags = t;
        }
        if let Some(c) = self.custom_metadata {
            n.custom_metadata = c;
        }
    }

    pub fn unchanged() -> Self {
        NoteMetadataEditable {
            tags: None,
            custom_metadata: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::notemetadata::NoteMetadataEditable;
    use crate::{InMemoryStore, NoteStore, PlainNote};
    use std::option::Option::None;

    #[tokio::test]
    async fn update_note_tags() {
        let store: InMemoryStore<PlainNote> = InMemoryStore::new();
        let loc1 = store
            .new_note(
                "".to_owned(),
                PlainNote::new("Foo".into()),
                NoteMetadataEditable::unchanged(),
            )
            .await
            .unwrap();
        let rev1 = loc1.get_revision().unwrap();
        let metadata1 = store
            .get_note(&loc1.current())
            .await
            .unwrap()
            .get_metadata();
        let mut tags = metadata1.tags.clone();
        tags.insert("my_tag".to_owned());
        let new_metadata = NoteMetadataEditable {
            tags: Some(tags),
            custom_metadata: None,
        };
        let loc2 = store
            .update_note(&loc1, None, None, new_metadata)
            .await
            .unwrap();
        let rev2 = loc2.get_revision().unwrap();
        assert_ne!(rev1, rev2);
        assert!(!store
            .get_note(&loc1)
            .await
            .unwrap()
            .get_metadata()
            .tags
            .contains("my_tag"));
        assert!(store
            .get_note(&loc1.at_revision(rev2))
            .await
            .unwrap()
            .get_metadata()
            .tags
            .contains("my_tag"));
        assert!(store
            .get_note(&loc1.current())
            .await
            .unwrap()
            .get_metadata()
            .tags
            .contains("my_tag"));
    }
}
