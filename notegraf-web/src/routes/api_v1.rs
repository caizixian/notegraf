use crate::NoteType;
use actix_web::{delete, get, post, web, HttpResponse, Responder};
use notegraf::errors::NoteStoreError;
use notegraf::notemetadata::NoteMetadataEditable;
use notegraf::notestore::BoxedNoteStore;
use notegraf::{NoteLocator, NoteSerializable};
use serde::Deserialize;
use std::collections::HashSet;

fn notestore_error_handler(e: &NoteStoreError) -> HttpResponse {
    match e {
        NoteStoreError::NoteNotExist(_) => HttpResponse::NotFound().body(e.to_string()),
        NoteStoreError::NoteIDConflict(_) => HttpResponse::Conflict().body(e.to_string()),
        NoteStoreError::RevisionNotExist(_, _) => HttpResponse::NotFound().body(e.to_string()),
        NoteStoreError::IOError(_) => {
            error!("Note store internal error {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
        NoteStoreError::SerdeError(_) => HttpResponse::BadRequest().body(e.to_string()),
        NoteStoreError::UpdateOldRevision(_, _) => HttpResponse::Conflict().body(e.to_string()),
        NoteStoreError::DeleteOldRevision(_, _) => HttpResponse::Conflict().body(e.to_string()),
        NoteStoreError::NotAChild(_, _) => HttpResponse::Conflict().body(e.to_string()),
        NoteStoreError::ExistingNext(_, _) => HttpResponse::Conflict().body(e.to_string()),
        NoteStoreError::HasBranches(_) => HttpResponse::Conflict().body(e.to_string()),
        NoteStoreError::HasReferences(_) => HttpResponse::Conflict().body(e.to_string()),
        NoteStoreError::ParseError(_) => HttpResponse::BadRequest().body(e.to_string()),
        NoteStoreError::PostgreSQLError(_) => {
            error!("Note store internal error {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
        NoteStoreError::NoteInnerError(_) => HttpResponse::BadRequest().body(e.to_string()),
        NoteStoreError::NotUuid(_) => HttpResponse::BadRequest().body(e.to_string()),
    }
}

async fn get_note_by_locator(
    store: web::Data<BoxedNoteStore<NoteType>>,
    loc: &NoteLocator,
) -> impl Responder {
    let result = store.as_ref().get_note(loc).await;
    match result {
        Ok(note) => HttpResponse::Ok().json(NoteSerializable::all_fields(note)),
        Err(e) => notestore_error_handler(&e),
    }
}

#[delete("/note/{note_id}")]
#[instrument(
    skip(store, params),
    fields(
        note_id = %params.0
    )
)]
async fn delete_note_current(
    store: web::Data<BoxedNoteStore<NoteType>>,
    params: web::Path<(String,)>,
) -> impl Responder {
    let (note_id,) = params.into_inner();
    let loc = NoteLocator::Current(note_id.into());
    let res = store.delete_note(&loc).await;
    match res {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => notestore_error_handler(&e),
    }
}

#[get("/note/{note_id}/revision/{revision_id}")]
#[instrument(
    skip(store, params),
    fields(
        note_id = %params.0,
        revision_id = %params.1
    )
)]
async fn get_note_specific(
    store: web::Data<BoxedNoteStore<NoteType>>,
    params: web::Path<(String, String)>,
) -> impl Responder {
    let (note_id, revision_id) = params.into_inner();
    let loc = NoteLocator::Specific(note_id.into(), revision_id.into());
    get_note_by_locator(store, &loc).await
}

#[derive(Deserialize)]
struct NewNoteData {
    title: String,
    note_inner: String,
    metadata_tags: String,
    metadata_custom_metadata: String,
}

#[post("/note")]
#[instrument(skip(store, note))]
async fn new_note(
    store: web::Data<BoxedNoteStore<NoteType>>,
    note: web::Json<NewNoteData>,
) -> impl Responder {
    let note = note.into_inner();
    let res = serde_json::from_str(&note.metadata_custom_metadata);
    if let Err(e) = res {
        return HttpResponse::BadRequest().body(e.to_string());
    }
    let custom_metadata = res.unwrap();
    let tags: HashSet<String> = HashSet::from_iter(
        note.metadata_tags
            .split(',')
            .into_iter()
            .map(|tag| tag.trim().to_owned()),
    );
    let res = store
        .new_note(
            note.title,
            NoteType::from(note.note_inner),
            NoteMetadataEditable {
                custom_metadata: Some(custom_metadata),
                tags: Some(tags),
            },
        )
        .await;
    match res {
        Ok(loc) => HttpResponse::Ok().json(loc),
        Err(e) => notestore_error_handler(&e),
    }
}

#[get("/note/{note_id}/revision")]
#[instrument(
    skip(store, params),
    fields(
        note_id = %params.0
    )
)]
async fn get_revisions(
    store: web::Data<BoxedNoteStore<NoteType>>,
    params: web::Path<(String,)>,
) -> impl Responder {
    let (note_id,) = params.into_inner();
    let loc = NoteLocator::Current(note_id.into());
    let res = store.get_revisions(&loc).await;
    if let Err(e) = res {
        return notestore_error_handler(&e);
    }
    let revisions: Vec<NoteSerializable<NoteType>> = res
        .unwrap()
        .into_iter()
        .map(|x| NoteSerializable::all_fields(x))
        .collect();
    return HttpResponse::Ok().json(revisions);
}

#[post("/note/{note_id}/revision")]
#[instrument(
    skip(store, params, note),
    fields(
        note_id = %params.0
    )
)]
async fn update_note(
    store: web::Data<BoxedNoteStore<NoteType>>,
    params: web::Path<(String,)>,
    note: web::Json<NewNoteData>,
) -> impl Responder {
    let (note_id,) = params.into_inner();
    let loc = NoteLocator::Current(note_id.into());
    let note = note.into_inner();
    let res = serde_json::from_str(&note.metadata_custom_metadata);
    if let Err(e) = res {
        return HttpResponse::BadRequest().body(e.to_string());
    }
    let custom_metadata = res.unwrap();
    let tags: HashSet<String> = HashSet::from_iter(
        note.metadata_tags
            .split(',')
            .into_iter()
            .map(|tag| tag.trim().to_owned()),
    );
    let res = store
        .update_note(
            &loc,
            Some(note.title),
            Some(NoteType::from(note.note_inner)),
            NoteMetadataEditable {
                custom_metadata: Some(custom_metadata),
                tags: Some(tags),
            },
        )
        .await;
    match res {
        Ok(loc) => HttpResponse::Ok().json(loc),
        Err(e) => notestore_error_handler(&e),
    }
}

#[get("/note/{note_id}")]
#[instrument(
    skip(store, params),
    fields(
        note_id = %params.0
    )
)]
async fn get_note_current(
    store: web::Data<BoxedNoteStore<NoteType>>,
    params: web::Path<(String,)>,
) -> impl Responder {
    let (note_id,) = params.into_inner();
    let loc = NoteLocator::Current(note_id.into());
    get_note_by_locator(store, &loc).await
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_note_current)
        .service(get_note_specific)
        .service(new_note)
        .service(delete_note_current)
        .service(update_note)
        .service(get_revisions);
}
