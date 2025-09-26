use actix_web::{get, web, HttpResponse};
use std::path::PathBuf;

const SHOW_HTML: &str = include_str!("../views/videos/show.html");

#[get("/videos/{video_path:.*}")]
pub async fn show(video_path: web::Path<PathBuf>) -> HttpResponse {
    let filename = format!("/{}", video_path.display());
    let html = SHOW_HTML.replace("{{filename}}", &filename);

    HttpResponse::Ok().content_type("text/html").body(html)
}

pub fn video_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(show);
}
