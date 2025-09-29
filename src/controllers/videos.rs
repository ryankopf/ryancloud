use actix_web::{get, web, HttpResponse, HttpRequest, Responder};
use std::path::{PathBuf, Path};
use actix_web::http::header;
use std::fs;

const SHOW_HTML: &str = include_str!("../views/videos/show.html");


#[get("/videos/{video_path:.*}")]
pub async fn show(video_path: web::Path<PathBuf>) -> HttpResponse {
    let filename = format!("/{}", video_path.display());
    let html = SHOW_HTML.replace("{{filename}}", &filename);

    HttpResponse::Ok().content_type("text/html").body(html)
}

// Utility: get sorted list of video files in the same directory, and find next/prev
fn get_sorted_videos_and_index(current_path: &Path) -> Option<(Vec<String>, usize)> {
    let video_extensions = ["mp4", "avi", "mov", "mkv", "webm"];
    let parent = current_path.parent()?;
    let mut entries: Vec<_> = fs::read_dir(parent).ok()?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().is_file() && e.path().extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| video_extensions.contains(&ext.to_lowercase().as_str()))
                .unwrap_or(false)
        })
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    entries.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
    let file_name = current_path.file_name()?.to_string_lossy();
    let idx = entries.iter().position(|n| n == &file_name)?;
    Some((entries, idx))
}

#[get("/videos/{video_path:.*}/next")]
pub async fn next(_req: HttpRequest, video_path: web::Path<PathBuf>) -> impl Responder {
    let current = video_path.into_inner();
    let abs_path = Path::new("").join(&current);
    if let Some((files, idx)) = get_sorted_videos_and_index(&abs_path) {
        let next_idx = if idx + 1 < files.len() { idx + 1 } else { idx };
        let next_file = &files[next_idx];
        let new_path = abs_path.parent().unwrap_or(Path::new("")).join(next_file);
        let rel_path = new_path.strip_prefix("").unwrap_or(&new_path);
        let url = format!("/videos/{}", rel_path.display());
        return HttpResponse::Found().append_header((header::LOCATION, url)).finish();
    }
    // fallback: redirect to current
    let url = format!("/videos/{}", abs_path.display());
    HttpResponse::Found().append_header((header::LOCATION, url)).finish()
}

#[get("/videos/{video_path:.*}/prev")]
pub async fn prev(_req: HttpRequest, video_path: web::Path<PathBuf>) -> impl Responder {
    let current = video_path.into_inner();
    let abs_path = Path::new("").join(&current);
    if let Some((files, idx)) = get_sorted_videos_and_index(&abs_path) {
        let prev_idx = if idx > 0 { idx - 1 } else { idx };
        let prev_file = &files[prev_idx];
        let new_path = abs_path.parent().unwrap_or(Path::new("")).join(prev_file);
        let rel_path = new_path.strip_prefix("").unwrap_or(&new_path);
        let url = format!("/videos/{}", rel_path.display());
        return HttpResponse::Found().append_header((header::LOCATION, url)).finish();
    }
    // fallback: redirect to current
    let url = format!("/videos/{}", abs_path.display());
    HttpResponse::Found().append_header((header::LOCATION, url)).finish()
}

pub fn video_routes(cfg: &mut web::ServiceConfig) {
    cfg
        .service(next)
        .service(prev)
        .service(show)
        ;
}
