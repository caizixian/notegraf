use crate::NoteType;
use actix_web::{delete, get, post, web, HttpResponse, Responder};
use notegraf::errors::NoteStoreError;
use notegraf::notemetadata::NoteMetadata;
use notegraf::notestore::BoxedNoteStore;
use notegraf::{NoteLocator, NoteSerializable};
use serde::Deserialize;

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
    metadata: Option<NoteMetadata>,
}

#[post("/note")]
#[instrument(skip(store, note))]
async fn new_note(
    store: web::Data<BoxedNoteStore<NoteType>>,
    note: web::Json<NewNoteData>,
) -> impl Responder {
    let note = note.into_inner();
    let res = store
        .new_note(note.title, NoteType::from(note.note_inner), note.metadata)
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
        .service(delete_note_current);
}
