use actix_web::{get, web, HttpResponse};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use crate::models::clip;
use std::fs;

#[get("/search")]
pub async fn index(
    query: web::Query<std::collections::HashMap<String, String>>,
    db: web::Data<DatabaseConnection>,
) -> HttpResponse {
    let search_term = query.get("q").unwrap_or(&"".to_string()).to_lowercase();

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
            "<li class='list-group-item'><b>{}</b><p>{}</p><a href='/segments/{}'>View Clip</a></li>",
            clip.name.unwrap_or_else(|| "Untitled".to_string()),
            clip.description.unwrap_or_else(|| "No description available.".to_string()),
            clip.clip_filename,
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
