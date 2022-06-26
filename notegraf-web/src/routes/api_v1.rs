use crate::NoteType;
use actix_web::{get, post, web, HttpResponse, Responder};
use notegraf::errors::NoteStoreError;
use notegraf::notemetadata::NoteMetadata;
use notegraf::notestore::BoxedNoteStore;
use notegraf::{NoteLocator, NoteSerializable};
use serde::Deserialize;

async fn get_note_by_locator(
    store: web::Data<BoxedNoteStore<NoteType>>,
    loc: &NoteLocator,
) -> impl Responder {
    let result = store.as_ref().get_note(loc).await;
    match result {
        Ok(note) => HttpResponse::Ok().json(NoteSerializable::all_fields(note)),
        Err(e) => {
            if let NoteStoreError::NoteNotExist(n) = e {
                info!("Attempted to get a note by id \"{}\", but not found", &n);
                HttpResponse::NotFound().finish()
            } else {
                error!("Note store internal error {:?}", e);
                HttpResponse::InternalServerError().finish()
            }
        }
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
        Err(e) => {
            error!("Note store internal error {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_note_current)
        .service(get_note_specific)
        .service(new_note);
}
