



#[get("/search")]
pub async fn index(
    // video_path: web::Path<PathBuf>,
    db: web::Data<DatabaseConnection>,
) -> HttpResponse {
    
}

pub fn search_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(index);
}
