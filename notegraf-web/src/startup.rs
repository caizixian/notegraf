use crate::routes::*;
use crate::NoteType;
use actix_files::{Files, NamedFile};
use actix_web::dev::Server;
use actix_web::middleware::{NormalizePath, TrailingSlash};
use actix_web::web::Data;
use actix_web::{web, App, HttpServer};
use notegraf::notestore::BoxedNoteStore;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

async fn index_file() -> actix_web::Result<NamedFile> {
    Ok(NamedFile::open("./dist/index.html")?)
}

pub fn run(
    listener: TcpListener,
    note_store: BoxedNoteStore<NoteType>,
    debug: bool,
) -> Result<Server, std::io::Error> {
    let ns: Data<BoxedNoteStore<NoteType>> = Data::new(note_store);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(NormalizePath::new(TrailingSlash::Trim))
            .wrap(TracingLogger::default())
            .service(web::scope("/api/v1").configure(api_v1_config))
            .configure(index_config)
            .configure(|cfg| {
                if !debug {
                    cfg.service(Files::new("/static", "./dist/"));
                }
            })
            // Service the static page for client-side routing
            // https://create-react-app.dev/docs/deployment/#serving-apps-with-client-side-routing
            .service(web::resource("/{tail}*").route(web::get().to(index_file)))
            .app_data(ns.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}
