use super::PostgreSQLNote;
use crate::errors::NoteStoreError;
use crate::notemetadata::NoteMetadata;
use crate::notestore::postgresql::get_new_revision;
use crate::{NoteID, NoteLocator, NoteType};
use chrono::{DateTime, Utc};
use sqlx::postgres::PgQueryResult;
use sqlx::{query, query_as, Postgres, Transaction};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Clone)]
pub(super) struct PostgreSQLNoteEditable<T> {
    pub(super) id: Uuid,
    pub(super) revision: Uuid,
    pub(super) title: String,
    pub(super) note_inner: T,
    pub(super) parent: Option<Uuid>,
    pub(super) prev: Option<Uuid>,
    pub(super) metadata: NoteMetadata,
}

#[derive(sqlx::FromRow)]
struct PostgreSQLNoteRow {
    revision: Uuid,
    id: Uuid,
    title: String,
    note_inner: String,
    parent: Option<Uuid>,
    prev: Option<Uuid>,
    referents: Vec<Uuid>,
    metadata_schema_version: i64,
    metadata_created_at: DateTime<Utc>,
    metadata_modified_at: DateTime<Utc>,
    metadata_tags: Vec<String>,
    metadata_custom_metadata: serde_json::Value,
}

impl<T> TryFrom<PostgreSQLNoteEditable<T>> for PostgreSQLNoteRow
where
    T: NoteType,
{
    type Error = NoteStoreError;

    fn try_from(n: PostgreSQLNoteEditable<T>) -> Result<Self, Self::Error> {
        let referents: Vec<Uuid> = match n.note_inner.get_referents() {
            Ok(r) => r,
            Err(e) => return Err(NoteStoreError::NoteInnerError(e.to_string())),
        }
        .iter()
        .map(|x| x.try_to_uuid())
        .collect::<Result<Vec<Uuid>, NoteStoreError>>()?;
        let tags: Vec<String> = n.metadata.tags.iter().cloned().collect();
        let note_inner: String = n.note_inner.clone().into();
        Ok(PostgreSQLNoteRow {
            revision: n.revision,
            id: n.id,
            title: n.title,
            note_inner,
            parent: n.parent,
            prev: n.prev,
            referents,
            metadata_schema_version: n.metadata.schema_version as i64,
            metadata_created_at: n.metadata.created_at,
            metadata_modified_at: n.metadata.modified_at,
            metadata_tags: tags,
            metadata_custom_metadata: n.metadata.custom_metadata,
        })
    }
}

impl<T> From<PostgreSQLNoteRow> for PostgreSQLNoteEditable<T>
where
    T: NoteType,
{
    fn from(n: PostgreSQLNoteRow) -> Self {
        let metadata = NoteMetadata {
            schema_version: n.metadata_schema_version as u64,
            created_at: n.metadata_created_at,
            modified_at: n.metadata_modified_at,
            tags: HashSet::from_iter(n.metadata_tags.iter().cloned()),
            custom_metadata: n.metadata_custom_metadata,
        };
        let note_inner: T = T::from(n.note_inner);
        PostgreSQLNoteEditable {
            id: n.id,
            revision: n.revision,
            title: n.title,
            note_inner,
            parent: n.parent,
            prev: n.prev,
            metadata,
        }
    }
}

#[derive(sqlx::FromRow)]
pub(super) struct PostgreSQLNoteRowJoined {
    pub(super) revision: Uuid,
    pub(super) id: Uuid,
    pub(super) title: String,
    pub(super) note_inner: String,
    pub(super) parent: Option<Uuid>,
    pub(super) branches: Option<Vec<Uuid>>,
    pub(super) prev: Option<Uuid>,
    pub(super) next: Option<Vec<Uuid>>,
    pub(super) referents: Vec<Uuid>,
    pub(super) references: Option<Vec<Uuid>>,
    pub(super) metadata_schema_version: i64,
    pub(super) metadata_created_at: DateTime<Utc>,
    pub(super) metadata_modified_at: DateTime<Utc>,
    pub(super) metadata_tags: Vec<String>,
    pub(super) metadata_custom_metadata: serde_json::Value,
}

impl PostgreSQLNoteRowJoined {
    pub(super) fn into_note<T: NoteType>(self) -> PostgreSQLNote<T> {
        let note_inner: T = T::from(self.note_inner);
        let parent: Option<NoteID> = self.parent.map(|x| x.into());
        let branches: HashSet<NoteID> = match self.branches {
            Some(b) => HashSet::from_iter(b.iter().map(|x| x.into())),
            None => HashSet::new(),
        };
        let prev: Option<NoteID> = self.prev.map(|x| x.into());
        let next: Option<NoteID> = match self.next {
            Some(n) => {
                if n.is_empty() {
                    None
                } else {
                    assert_eq!(n.len(), 1);
                    Some(n[0].into())
                }
            }
            None => None,
        };
        let referents: HashSet<NoteID> =
            HashSet::from_iter(self.referents.iter().map(|x| x.into()));
        let references: HashSet<NoteID> = match self.references {
            Some(r) => HashSet::from_iter(r.iter().map(|x| x.into())),
            None => HashSet::new(),
        };
        let metadata = NoteMetadata {
            schema_version: self.metadata_schema_version as u64,
            created_at: self.metadata_created_at,
            modified_at: self.metadata_modified_at,
            tags: HashSet::from_iter(self.metadata_tags.iter().cloned()),
            custom_metadata: self.metadata_custom_metadata,
        };
        PostgreSQLNote {
            title: self.title,
            note_inner,
            id: self.id.into(),
            revision: self.revision.into(),
            parent,
            branches,
            prev,
            next,
            referents,
            references,
            metadata,
        }
    }
}

async fn get_note_current(
    transaction: &mut Transaction<'_, Postgres>,
    id: Uuid,
) -> Result<PostgreSQLNoteRowJoined, NoteStoreError> {
    // Manual left join on current_revision used instead of the revision_is_current view
    // https://dba.stackexchange.com/questions/238087/group-by-on-view-queries
    let res = query_as!(
            PostgreSQLNoteRowJoined,
            r#"
            SELECT
                revision.revision,
                revision.id,
                revision.title,
                revision.note_inner,
                revision.parent,
                array_remove(array_agg(revision1.id), NULL) AS branches,
                revision.prev,
                array_remove(array_agg(revision2.id), NULL) AS next,
                revision.referents,
                array_remove(array_agg(revision3.id), NULL) AS "references",
                revision.metadata_schema_version,
                revision.metadata_created_at,
                revision.metadata_modified_at,
                revision.metadata_tags,
                revision.metadata_custom_metadata
            FROM revision
            LEFT JOIN current_revision cr on revision.revision = cr.current_revision
            LEFT JOIN revision_only_current AS revision1 on revision1.parent = revision.id
            LEFT JOIN revision_only_current AS revision2 on revision2.prev = revision.id
            -- https://stackoverflow.com/a/29245753
            -- indexes are bound to operators, and the indexed expression must be to the left of
            -- the operator
            LEFT JOIN revision_only_current AS revision3 on revision3.referents @> ARRAY[revision.id]
            WHERE revision.id = $1 AND cr.current_revision IS NOT NULL
            GROUP BY revision.revision
            "#,
            id,
        )
        .fetch_one(transaction)
        .await;
    if let Err(sqlx::Error::RowNotFound) = res {
        Err(NoteStoreError::NoteNotExist(id.into()))
    } else {
        res.map_err(NoteStoreError::PostgreSQLError)
    }
}

async fn get_note_specific(
    transaction: &mut Transaction<'_, Postgres>,
    id: Uuid,
    revision: Uuid,
) -> Result<PostgreSQLNoteRowJoined, NoteStoreError> {
    let res = query_as!(
            PostgreSQLNoteRowJoined,
            r#"
            SELECT
                revision.revision,
                revision.id,
                revision.title,
                revision.note_inner,
                revision.parent,
                array_remove(array_agg(revision1.id), NULL) AS branches,
                revision.prev,
                array_remove(array_agg(revision2.id), NULL) AS next,
                revision.referents,
                array_remove(array_agg(revision3.id), NULL) AS "references",
                revision.metadata_schema_version,
                revision.metadata_created_at,
                revision.metadata_modified_at,
                revision.metadata_tags,
                revision.metadata_custom_metadata
            FROM revision
            LEFT JOIN revision_only_current AS revision1 on revision1.parent = revision.id
            LEFT JOIN revision_only_current AS revision2 on revision2.prev = revision.id
            -- https://stackoverflow.com/a/29245753
            -- indexes are bound to operators, and the indexed expression must be to the left of
            -- the operator
            LEFT JOIN revision_only_current AS revision3 on revision3.referents @> ARRAY[revision.id]
            WHERE revision.id = $1 AND revision.revision = $2
            GROUP BY revision.revision
            "#,
            id,
            revision
        )
        .fetch_one(transaction)
        .await;
    if let Err(sqlx::Error::RowNotFound) = res {
        Err(NoteStoreError::RevisionNotExist(id.into(), revision.into()))
    } else {
        res.map_err(NoteStoreError::PostgreSQLError)
    }
}

async fn get_row_current(
    transaction: &mut Transaction<'_, Postgres>,
    id: Uuid,
) -> Result<PostgreSQLNoteRow, NoteStoreError> {
    let res = query_as!(
        PostgreSQLNoteRow,
        r#"
            SELECT
                revision.revision,
                revision.id,
                revision.title,
                revision.note_inner,
                revision.parent,
                revision.prev,
                revision.referents,
                revision.metadata_schema_version,
                revision.metadata_created_at,
                revision.metadata_modified_at,
                revision.metadata_tags,
                revision.metadata_custom_metadata
            FROM revision
            LEFT JOIN current_revision cr on revision.revision = cr.current_revision
            WHERE revision.id = $1 AND cr.current_revision IS NOT NULL
            "#,
        id,
    )
    .fetch_one(transaction)
    .await;
    if let Err(sqlx::Error::RowNotFound) = res {
        Err(NoteStoreError::NoteNotExist(id.into()))
    } else {
        res.map_err(NoteStoreError::PostgreSQLError)
    }
}

async fn get_row_specific(
    transaction: &mut Transaction<'_, Postgres>,
    id: Uuid,
    revision: Uuid,
) -> Result<PostgreSQLNoteRow, NoteStoreError> {
    let res = query_as!(
        PostgreSQLNoteRow,
        r#"
            SELECT
                revision,
                id,
                title,
                note_inner,
                parent,
                prev,
                referents,
                metadata_schema_version,
                metadata_created_at,
                metadata_modified_at,
                metadata_tags,
                metadata_custom_metadata
            FROM revision
            WHERE id = $1 AND revision = $2
            "#,
        id,
        revision
    )
    .fetch_one(transaction)
    .await;
    if let Err(sqlx::Error::RowNotFound) = res {
        Err(NoteStoreError::NoteNotExist(id.into()))
    } else {
        res.map_err(NoteStoreError::PostgreSQLError)
    }
}

pub(super) async fn get_note_by_loc(
    transaction: &mut Transaction<'_, Postgres>,
    loc: &NoteLocator,
) -> Result<PostgreSQLNoteRowJoined, NoteStoreError> {
    let (id, revision) = loc.unpack_uuid()?;
    match revision {
        Some(r) => get_note_specific(transaction, id, r).await,
        None => get_note_current(transaction, id).await,
    }
}

async fn get_row_by_loc(
    transaction: &mut Transaction<'_, Postgres>,
    loc: &NoteLocator,
) -> Result<PostgreSQLNoteRow, NoteStoreError> {
    let (id, revision) = loc.unpack_uuid()?;
    match revision {
        Some(r) => get_row_specific(transaction, id, r).await,
        None => get_row_current(transaction, id).await,
    }
}

pub(super) async fn insert_revision<T: NoteType>(
    transaction: &mut Transaction<'_, Postgres>,
    n: PostgreSQLNoteEditable<T>,
) -> Result<NoteLocator, NoteStoreError> {
    let row: PostgreSQLNoteRow = n.try_into()?;
    query!(
        r#"
            INSERT INTO
                revision(
                    revision, id, title, note_inner, parent, prev, referents,
                    metadata_schema_version, metadata_created_at,
                    metadata_modified_at, metadata_tags, metadata_custom_metadata
                )
            VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
        row.revision,
        row.id,
        row.title,
        row.note_inner,
        row.parent,
        row.prev,
        &row.referents,
        row.metadata_schema_version,
        row.metadata_created_at,
        row.metadata_modified_at,
        &row.metadata_tags,
        row.metadata_custom_metadata
    )
    .execute(transaction)
    .await
    .map_err(NoteStoreError::PostgreSQLError)?;
    Ok(NoteLocator::Specific(row.id.into(), row.revision.into()))
}

pub(super) async fn upsert_current_revision(
    transaction: &mut Transaction<'_, Postgres>,
    id: Uuid,
    revision: Uuid,
) -> sqlx::Result<PgQueryResult> {
    query!(
        r#"
            INSERT INTO current_revision (id, current_revision)
            VALUES ($1, $2)
            ON CONFLICT (id) DO UPDATE
            SET current_revision = EXCLUDED.current_revision
            "#,
        id,
        revision
    )
    .execute(transaction)
    .await
}

pub(super) async fn noteid_exist(
    transaction: &mut Transaction<'_, Postgres>,
    id: Uuid,
) -> Result<bool, NoteStoreError> {
    let res = query!(
        r#"
            SELECT id
            FROM note
            WHERE id = $1
            "#,
        id
    )
    .fetch_one(transaction)
    .await;
    match res {
        Ok(_) => Ok(true),
        Err(e) => {
            if matches!(e, sqlx::Error::RowNotFound) {
                Ok(false)
            } else {
                Err(NoteStoreError::PostgreSQLError(e))
            }
        }
    }
}

pub(super) async fn is_current(
    transaction: &mut Transaction<'_, Postgres>,
    loc: &NoteLocator,
) -> Result<bool, NoteStoreError> {
    let (id, revision) = loc.unpack_uuid()?;
    if revision.is_none() {
        return noteid_exist(transaction, id).await;
    }
    let res = query!(
        r#"
            SELECT id
            FROM current_revision
            WHERE id = $1 AND current_revision = $2
            "#,
        id,
        revision.unwrap()
    )
    .fetch_one(transaction)
    .await;
    match res {
        Ok(_) => Ok(true),
        Err(e) => {
            if matches!(e, sqlx::Error::RowNotFound) {
                Ok(false)
            } else {
                Err(NoteStoreError::PostgreSQLError(e))
            }
        }
    }
}

pub(super) async fn delete_revision(
    mut transaction: Transaction<'_, Postgres>,
    loc: &NoteLocator,
) -> Result<(), NoteStoreError> {
    let (id, revision) = loc.unpack_uuid()?;
    let query_result = match revision {
        Some(r) => {
            query!(
                r#"DELETE FROM current_revision WHERE id = $1 AND current_revision = $2"#,
                id,
                r
            )
            .execute(&mut transaction)
            .await?
        }
        None => {
            query!(r#"DELETE FROM current_revision WHERE id = $1"#, id)
                .execute(&mut transaction)
                .await?
        }
    };
    if query_result.rows_affected() != 1 {
        transaction
            .rollback()
            .await
            .map_err(NoteStoreError::PostgreSQLError)?;
        match loc {
            NoteLocator::Current(id) => Err(NoteStoreError::NoteNotExist(id.clone())),
            NoteLocator::Specific(id, revision) => Err(NoteStoreError::RevisionNotExist(
                id.clone(),
                revision.clone(),
            )),
        }
    } else {
        transaction
            .commit()
            .await
            .map_err(NoteStoreError::PostgreSQLError)?;
        Ok(())
    }
}

pub(super) async fn is_deleted(
    transaction: &mut Transaction<'_, Postgres>,
    id: Uuid,
) -> Result<bool, NoteStoreError> {
    // NULL not correctly inferred by sqlx https://github.com/launchbadge/sqlx/issues/367
    let res = query!(
        r#"
            SELECT note.id, cr.current_revision AS "current_revision?"
            FROM note
            LEFT JOIN current_revision cr on cr.id = note.id
            WHERE note.id = $1
            "#,
        id
    )
    .fetch_one(transaction)
    .await;
    match res {
        Ok(row) => Ok(row.current_revision.is_none()),
        Err(e) => {
            if matches!(e, sqlx::Error::RowNotFound) {
                Err(NoteStoreError::NoteNotExist(id.into()))
            } else {
                Err(NoteStoreError::PostgreSQLError(e))
            }
        }
    }
}

pub(super) async fn update_note_helper<F, T>(
    transaction: &mut Transaction<'_, Postgres>,
    loc: &NoteLocator,
    op: F,
) -> Result<NoteLocator, NoteStoreError>
where
    F: FnOnce(&PostgreSQLNoteEditable<T>) -> Result<PostgreSQLNoteEditable<T>, NoteStoreError>,
    T: NoteType,
{
    let (id, rev) = loc.unpack_uuid()?;
    let is_resurrecting = is_deleted(transaction, id).await?;
    let old_note_row: PostgreSQLNoteRow = if is_resurrecting || is_current(transaction, loc).await?
    {
        get_row_by_loc(transaction, loc).await?
    } else {
        return Err(NoteStoreError::UpdateOldRevision(
            id.into(),
            rev.unwrap().into(),
        ));
    };
    let old_note: PostgreSQLNoteEditable<T> = old_note_row.into();
    let new_revision = get_new_revision();
    let mut updated_note = op(&old_note)?;
    updated_note.revision = new_revision;
    updated_note.metadata = updated_note.metadata.on_update_note();
    if is_resurrecting {
        // If a note previously has a prev note, we will clear the attribute, in case the prev
        // note now has a next
        // Similarly for branches
        updated_note.parent = None;
        updated_note.prev = None;
    }
    let new_loc = insert_revision(transaction, updated_note).await?;
    upsert_current_revision(transaction, id, new_revision).await?;
    Ok(new_loc)
}

pub(super) async fn get_revisions(
    transaction: &mut Transaction<'_, Postgres>,
    loc: &NoteLocator,
) -> Result<Vec<Uuid>, NoteStoreError> {
    let id = loc.get_id().try_to_uuid()?;
    let res = query!(
        r#"
            SELECT array_agg(revision) AS revisions
            FROM revision
            WHERE id = $1
            "#,
        id,
    )
    .fetch_one(transaction)
    .await;
    match res {
        Ok(row) => row
            .revisions
            .ok_or_else(|| NoteStoreError::NoteNotExist(id.into())),
        Err(e) => {
            if matches!(e, sqlx::Error::RowNotFound) {
                Err(NoteStoreError::NoteNotExist(id.into()))
            } else {
                Err(NoteStoreError::PostgreSQLError(e))
            }
        }
    }
}