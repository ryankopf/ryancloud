use actix_web::{get, web, HttpResponse};
use std::path::PathBuf;

#[get("/videos/{video_path:.*}")]
pub async fn show(video_path: web::Path<PathBuf>) -> HttpResponse {
    let controller = "videos";
    let view_name = "show";
    let form_path = format!("./src/views/{}/{}.html", controller, view_name);
    let content = std::fs::read_to_string(&form_path).unwrap_or_default();

    let filename = format!("/{}", video_path.display());
    let html = content.replace("{{filename}}", &filename);

    println!("Trying to serve video: {}", filename);

    HttpResponse::Ok().content_type("text/html").body(html)
}

pub fn video_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(show);
}
