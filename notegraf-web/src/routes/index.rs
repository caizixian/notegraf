use actix_web::{get, web, HttpResponse};

#[get("/health_check")]
#[instrument]
async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(health_check);
}
