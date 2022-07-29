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
struct NotePostData {
    title: String,
    note_inner: String,
    metadata_tags: String,
    metadata_custom_metadata: String,
}

struct NoteStoreEditArgument {
    title: String,
    note_inner: NoteType,
    metadata: NoteMetadataEditable,
}

impl TryFrom<NotePostData> for NoteStoreEditArgument {
    type Error = String;

    fn try_from(note: NotePostData) -> Result<Self, Self::Error> {
        let custom_metadata =
            serde_json::from_str(&note.metadata_custom_metadata).map_err(|e| e.to_string())?;
        let tags: HashSet<String> = HashSet::from_iter(
            note.metadata_tags
                .split(',')
                .into_iter()
                .map(|tag| tag.trim().to_owned())
                .filter(|tag| !tag.is_empty()),
        );
        Ok(NoteStoreEditArgument {
            title: note.title,
            note_inner: NoteType::from(note.note_inner),
            metadata: NoteMetadataEditable {
                tags: Some(tags),
                custom_metadata: Some(custom_metadata),
            },
        })
    }
}

#[post("/note")]
#[instrument(skip(store, note))]
async fn new_note(
    store: web::Data<BoxedNoteStore<NoteType>>,
    note: web::Json<NotePostData>,
) -> impl Responder {
    let note: Result<NoteStoreEditArgument, String> = note.into_inner().try_into();
    if let Err(e) = note {
        return HttpResponse::BadRequest().body(e);
    }
    let note = note.unwrap();
    let res = store
        .new_note(note.title, note.note_inner, note.metadata)
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
    HttpResponse::Ok().json(revisions)
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
    note: web::Json<NotePostData>,
) -> impl Responder {
    let (note_id,) = params.into_inner();
    let loc = NoteLocator::Current(note_id.into());
    let note: Result<NoteStoreEditArgument, String> = note.into_inner().try_into();
    if let Err(e) = note {
        return HttpResponse::BadRequest().body(e);
    }
    let note = note.unwrap();
    let res = store
        .update_note(&loc, Some(note.title), Some(note.note_inner), note.metadata)
        .await;
    match res {
        Ok(loc) => HttpResponse::Ok().json(loc),
        Err(e) => notestore_error_handler(&e),
    }
}

#[post("/note/{note_id}/branch")]
#[instrument(
    skip(store, params, note),
    fields(
      note_id = %params.0
    )
)]
async fn new_branch(
    store: web::Data<BoxedNoteStore<NoteType>>,
    params: web::Path<(String,)>,
    note: web::Json<NotePostData>,
) -> impl Responder {
    let (note_id,) = params.into_inner();
    let loc = NoteLocator::Current(note_id.into());
    let note: Result<NoteStoreEditArgument, String> = note.into_inner().try_into();
    if let Err(e) = note {
        return HttpResponse::BadRequest().body(e);
    }
    let note = note.unwrap();
    let res = store
        .add_branch(loc.get_id(), note.title, note.note_inner, note.metadata)
        .await;
    match res {
        Ok(loc_child) => HttpResponse::Ok().json(loc_child),
        Err(e) => notestore_error_handler(&e),
    }
}

#[post("/note/{note_id}/next")]
#[instrument(
    skip(store, params, note),
    fields(
        note_id = %params.0
    )
)]
async fn new_next(
    store: web::Data<BoxedNoteStore<NoteType>>,
    params: web::Path<(String,)>,
    note: web::Json<NotePostData>,
) -> impl Responder {
    let (note_id,) = params.into_inner();
    let loc = NoteLocator::Current(note_id.into());
    let note: Result<NoteStoreEditArgument, String> = note.into_inner().try_into();
    if let Err(e) = note {
        return HttpResponse::BadRequest().body(e);
    }
    let note = note.unwrap();
    let res = store
        .append_note(loc.get_id(), note.title, note.note_inner, note.metadata)
        .await;
    match res {
        Ok(loc_next) => HttpResponse::Ok().json(loc_next),
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

#[derive(Deserialize, Debug)]
struct SearchQuery {
    query: Option<String>,
}

#[get("/note")]
#[instrument(skip(store, search))]
async fn search(
    store: web::Data<BoxedNoteStore<NoteType>>,
    search: web::Query<SearchQuery>,
) -> impl Responder {
    let search = search.into_inner();
    let query = search.query;
    let res = if let Some(q) = query {
        store.search(&q.into()).await
    } else {
        store.search(&"".to_owned().into()).await
    };
    if let Err(e) = res {
        return notestore_error_handler(&e);
    }
    let revisions: Vec<NoteSerializable<NoteType>> = res
        .unwrap()
        .into_iter()
        .map(|x| NoteSerializable::all_fields(x))
        .collect();
    HttpResponse::Ok().json(revisions)
}

#[get("/tags")]
#[instrument(skip(store))]
async fn get_tags(store: web::Data<BoxedNoteStore<NoteType>>) -> impl Responder {
    let res = store.tags().await;

    if let Err(e) = res {
        return notestore_error_handler(&e);
    }
    HttpResponse::Ok().json(res.unwrap())
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_note_current)
        .service(get_note_specific)
        .service(new_note)
        .service(delete_note_current)
        .service(update_note)
        .service(get_revisions)
        .service(search)
        .service(new_branch)
        .service(new_next)
        .service(get_tags);
}
