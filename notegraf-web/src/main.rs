use actix_web::{web, App, HttpRequest, HttpServer, Responder};
use notegraf::{InMemoryStore, PlainNote, NoteStore, NoteLocator};
use actix_web::web::Data;

type NT = PlainNote;
type NS = InMemoryStore<NT>;

async fn index() -> impl Responder {
    "Notegraf".to_owned()
}

async fn new_note(ns: web::Data<NS>) -> impl Responder {
    let loc = ns
        .as_ref()
        .new_note(PlainNote::new("Hello world".into()))
        .await
        .unwrap();
    format!("{:?}", loc)
}

async fn get_note(req: HttpRequest, ns: web::Data<NS>) -> impl Responder {
    let note_id = req.match_info().get("note_id").unwrap();
    let loc = if let Some(revision) = req.match_info().get("revision") {
        NoteLocator::Specific(note_id.into(), revision.into())
    } else {
        NoteLocator::Current(note_id.into())
    };
    let note = ns.as_ref().get_note(&loc).await.unwrap();
    format!("{:?}", note)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let ns: Data<NS> = web::Data::new(NS::new());
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
