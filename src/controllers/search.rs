use actix_session::Session; // Import Session
use actix_web::{get, web, HttpResponse};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use crate::models::clip;
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
        html += &format!(
            "<li class='list-group-item'><a href='/segments/{clip_filename}'>{clip_name}</a><p>{clip_description}</p></li>",
            clip_filename = clip.clip_filename,
            clip_name = clip.name.unwrap_or_else(|| "Untitled".to_string()),
            clip_description = clip.description.unwrap_or_else(|| "No description available.".to_string()),
        );
    }

    for file in file_results {
        html += &format!("<li class='list-group-item'><a href='{}'>{}</a></li>", file, file);
    }

    html += "</ul></div>";

    HttpResponse::Ok().content_type("text/html").body(html)
}

pub fn search_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(index);
}
