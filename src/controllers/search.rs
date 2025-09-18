use actix_session::Session; // Import Session
use actix_web::{get, web, HttpResponse};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use crate::models::clip;
use crate::models::file::File;
use crate::controllers::files::generate_files_list_html; // Import the helper function
use std::fs;

#[get("/search")]
pub async fn index(
    query: web::Query<std::collections::HashMap<String, String>>,
    db: web::Data<DatabaseConnection>,
    session: Session, // Accept session as a parameter
) -> HttpResponse {
    let search_term = query.get("q").unwrap_or(&"".to_string()).to_lowercase();

    if search_term.is_empty() {
        let folder = std::env::current_dir().unwrap(); // Use the current directory as the folder
        let html = generate_files_list_html(&folder, "", &session);
        return HttpResponse::Ok().content_type("text/html").body(html);
    }

    // Search clips database
    let clips = clip::Entity::find()
        .filter(
            clip::Column::Name.contains(&search_term)
                .or(clip::Column::Description.contains(&search_term))
                .or(clip::Column::SourceFilename.contains(&search_term))
        )
        .all(db.get_ref())
        .await;

    let clips_result = match clips {
        Ok(clips) => clips,
        Err(_) => vec![],
    };

    // Search file system
    let mut file_results = vec![];
    if let Ok(entries) = fs::read_dir(".") {
        for entry in entries.flatten() {
            if let Ok(file_name) = entry.file_name().into_string() {
                if file_name.to_lowercase().contains(&search_term) {
                    file_results.push(file_name);
                }
            }
        }
    }

    // Combine results into HTML
    let mut html = String::new();
    html += "<div class='card'><div class='card-header'>Search Results</div><ul class='list-group list-group-flush'>";

    for clip in clips_result {
        html += &File::clip_preview(&clip);
    }
    
    let video_extensions = ["mp4", "avi", "mov", "mkv", "webm"];
    let mut videos = Vec::new(); // Placeholder for video files if needed

    for file in file_results {
        let ext = file.split('.').last().unwrap_or("").to_lowercase();
        let is_video = video_extensions.contains(&ext.as_str());
        html += &File::file_preview(&file, &file, is_video);
        if is_video {
            videos.push(file);
        }
    }

    html += "</ul></div>";
    if !videos.is_empty() {
        html += "<div class='card mt-4'><div class='card-header'>Videos</div><div class='card-body'><div class='flex flex-wrap gap-3'>";
        for video in videos {
            html += &File::video_preview("", &video);
        }
        html += "</div></div></div>";
    }

    HttpResponse::Ok().content_type("text/html").body(html)
}

pub fn search_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(index);
}
