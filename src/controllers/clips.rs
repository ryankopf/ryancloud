use actix_web::{get, post, web, HttpResponse};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde::Deserialize;
use std::path::PathBuf;
use crate::models::clip;
use crate::utils::makeclip::create_video_clip;

#[get("/clips/{video_path:.*}")]
pub async fn index(
    video_path: web::Path<PathBuf>,
    db: web::Data<DatabaseConnection>,
) -> HttpResponse {
    let video_path_str = video_path.display().to_string();

    // Fetch all clips associated with the given video path
    let clips = clip::Entity::find()
        .filter(clip::Column::SourceFilename.eq(video_path_str.clone()))
        .all(db.get_ref())
        .await;

    match clips {
        Ok(clips) if !clips.is_empty() => {
            let clips_html: String = clips
                .into_iter()
                .map(|clip| {
                    format!(
                        "<div><h2>{}</h2><p>{}</p><video src='/videos/{}' controls></video></div>",
                        clip.name.unwrap_or_else(|| "Untitled".to_string()),
                        clip.description.unwrap_or_else(|| "No description available.".to_string()),
                        clip.source_filename
                    )
                })
                .collect();

            let html = format!(
                "<html><body><h1>Clips for {}</h1>{}</body></html>",
                video_path_str, clips_html
            );
            HttpResponse::Ok().content_type("text/html").body(html)
        }
        Ok(_) => HttpResponse::NotFound().body("No clips found for the given video"),
        Err(err) => {
            eprintln!("Error fetching clips: {}", err);
            HttpResponse::InternalServerError().body("Internal server error")
        }
    }
}

#[derive(Deserialize)]
pub struct ClipForm {
    pub start: i64,
    pub end: i64,
    pub name: Option<String>,
    pub description: Option<String>,
}

#[post("/clips/{video_path:.*}")]
pub async fn create(
    video_path: web::Path<PathBuf>,
    form: web::Form<ClipForm>,
    db: web::Data<DatabaseConnection>,
) -> HttpResponse {
    let source_filename = video_path.display().to_string();

    // Insert into DB
    let new_clip = clip::ActiveModel {
        source_filename: Set(source_filename.clone()),
        start: Set(form.start),
        end: Set(form.end),
        name: Set(form.name.clone()),
        description: Set(form.description.clone()),
        ..Default::default()
    };

    if let Err(err) = new_clip.insert(db.get_ref()).await {
        eprintln!("Error creating clip: {}", err);
        return HttpResponse::InternalServerError().body("Failed to create clip");
    }

    // Kick off ffmpeg (async fire-and-forget)
    match create_video_clip(&source_filename, form.start, form.end, None) {
        Ok(output_path) => {
            HttpResponse::Created().body(format!("Clip creation started: {}", output_path.display()))
        }
        Err(err) => {
            eprintln!("Error spawning ffmpeg: {}", err);
            HttpResponse::InternalServerError().body(err)
        }
    }
}

pub fn clips_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(index);
    cfg.service(create);
}
