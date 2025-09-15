use actix_web::{get, HttpResponse, web};

// #[mvc_views]
#[get("/videos/{video_filename}")]
pub async fn show(
    video_filename: web::Path<String>,
) -> HttpResponse {

    let controller = "videos";
    let view_name = "show";
    let form_path = format!("./src/views/{}/{}.html", controller, view_name);
    let content = std::fs::read_to_string(&form_path).unwrap_or_default();
    let filename = video_filename.into_inner();
    let html = content.replace("{{filename}}", &filename);
    println!("Trying to serve video: {}", filename);
    HttpResponse::Ok().content_type("text/html").body(html)
}

pub fn video_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(show);
}
