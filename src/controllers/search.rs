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
            if let Ok(path) = entry.path().into_os_string().into_string() {
                if path.to_lowercase().contains(&search_term) {
                    file_results.push(path);
                }
            }
        }
    }

    // Combine results into HTML
    let clips_html = clips_result
        .into_iter()
        .map(|clip| {
            format!(
                "<div><b>{}</b><p>{}</p><video src='/segments/{}' controls class='w-100'></video></div>",
                clip.name.unwrap_or_else(|| "Untitled".to_string()),
                clip.description.unwrap_or_else(|| "No description available.".to_string()),
                clip.clip_filename,
            )
        })
        .collect::<String>();

    let files_html = file_results
        .into_iter()
        .map(|file| format!("<div><p>{}</p></div>", file))
        .collect::<String>();

    let html = format!(
        "<html><body><h6>Search Results</h6>{}{}</body></html>",
        clips_html, files_html
    );

    HttpResponse::Ok().content_type("text/html").body(html)
}

pub fn search_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(index);
}
