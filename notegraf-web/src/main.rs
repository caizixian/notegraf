use actix_web::*;
use notegraf::{InMemoryStore, Note, NoteLocator, PlainNote};

type NoteType = PlainNote;
type NoteStore = Box<dyn notegraf::NoteStore<NoteType> + Sync + Send>;

async fn index() -> Result<web::Json<&'static str>> {
    Ok(web::Json("Notegraf"))
}

async fn new_note(ns: web::Data<NoteStore>) -> Result<web::Json<NoteLocator>> {
    let loc = ns
        .as_ref()
        .new_note(PlainNote::new("Hello world".into()))
        .await
        .unwrap();
    Ok(web::Json(loc))
}

async fn get_note(
    req: HttpRequest,
    ns: web::Data<NoteStore>,
) -> Result<web::Json<Box<dyn Note<NoteType>>>> {
    let note_id = req.match_info().get("note_id").unwrap();
    let loc = if let Some(revision) = req.match_info().get("revision") {
        NoteLocator::Specific(note_id.into(), revision.into())
    } else {
        NoteLocator::Current(note_id.into())
    };
    let note = ns.as_ref().get_note(&loc).await.unwrap();
    Ok(web::Json(note))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let ns: web::Data<NoteStore> = web::Data::new(Box::new(InMemoryStore::new()));
    HttpServer::new(move || {
        App::new()
            .app_data(ns.clone())
            .route("/", web::get().to(index))
            .route("/new_note", web::get().to(new_note))
            .route("/get_note/{note_id}", web::get().to(get_note))
            .route("/get_note/{note_id}/{revision}", web::get().to(get_note))
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
