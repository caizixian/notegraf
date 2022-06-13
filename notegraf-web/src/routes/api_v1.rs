use crate::NoteType;
use actix_web::{get, web, HttpResponse, Responder};
use notegraf::errors::NoteStoreError;
use notegraf::notestore::BoxedNoteStore;
use notegraf::NoteLocator;
use serde_json::json;

async fn get_note_by_locator(
    store: web::Data<BoxedNoteStore<NoteType>>,
    loc: &NoteLocator,
) -> impl Responder {
    let result = store.as_ref().get_note(loc).await;
    match result {
        Ok(note) => HttpResponse::Ok().json(json!(note)),
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

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_note_current).service(get_note_specific);
}
