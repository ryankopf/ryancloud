use actix_web::{get, post, web, HttpResponse};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde::Deserialize;
use std::path::PathBuf;
use crate::models::clip;
use crate::utils::makeclip::create_video_clip;
use regex::Regex;

#[get("/clips/{video_path:.*}")]
pub async fn index(
    video_path: web::Path<PathBuf>,
    db: web::Data<DatabaseConnection>,
) -> HttpResponse {
    let video_path_str = video_path.display().to_string();
    let video_path_obj = PathBuf::from(&video_path_str);
    let videopath = video_path_obj.parent().map(|p| p.display().to_string()).unwrap_or_default();
    // let filename = video_path_obj.file_name().map(|f| f.to_string_lossy().to_string()).unwrap_or_default();

    // Fetch all clips associated with the given video path
    let clips = clip::Entity::find()
        .filter(clip::Column::SourceFilename.eq(video_path_str.clone()))
        .all(db.get_ref())
        .await;

    match clips {
        Ok(clips) => {
            let clips_html = if !clips.is_empty() {
                clips
                    .into_iter()
                    .map(|clip| {
                        // Build the video src as /{videopath}/segments/{clip_filename}
                        let src = if !videopath.is_empty() {
                            format!("/{}/segments/thumbs/{}.webp", videopath, clip.clip_filename)
                        } else {
                            format!("/segments/thumbs/{}.webp", clip.clip_filename)
                        };
                        format!(
                            "<div><b>{}</b><p>{}</p><img src='{}' class='w-100' onclick='replaceMe();return false;'></div>",
                            clip.name.unwrap_or_else(|| "Untitled".to_string()),
                            clip.description.unwrap_or_else(|| "No description available.".to_string()),
                            src,
                        )
                    })
                    .collect::<String>()
            } else {
                "<p>No clips found.</p>".to_string()
            };
            let html = format!(
                "<html><body><h6>Clips for {}</h6>{}</body></html>",
                video_path_str, clips_html
            );
            HttpResponse::Ok().content_type("text/html").body(html)
        }
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
    let working_directory = video_path.parent()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "".to_string());

    // Log incoming data for debugging
    eprintln!("Received POST request for video_path: {}", source_filename);
    eprintln!("Form data: start={}, end={}, name={:?}, description={:?}", form.start, form.end, form.name, form.description);

    // Validate form data
    if form.start >= form.end {
        return HttpResponse::BadRequest().body("Invalid clip range: 'start' must be less than 'end'");
    }

    if form.name.as_ref().map_or(true, |name| name.trim().is_empty()) {
        return HttpResponse::BadRequest().body("Clip name cannot be blank");
    }

    // Generate clip filename
    let clip_filename = form.name.as_ref()
        .map(|name| {
            let re = Regex::new(r"[^a-zA-Z0-9]+").unwrap();
            let sanitized = re.replace_all(name, "-").to_lowercase();
            format!("{}.mp4", sanitized.trim_matches('-'))
        })
        .unwrap_or_else(|| "clip.mp4".to_string());

    // Insert into DB
    let new_clip = clip::ActiveModel {
        source_filename: Set(source_filename.clone()),
        clip_filename: Set(clip_filename.clone()),
        start: Set(form.start),
        end: Set(form.end),
        name: Set(form.name.clone()),
        description: Set(form.description.clone()),
        working_directory: Set(working_directory), // Set working directory to the directory path
        ..Default::default()
    };

    if let Err(err) = new_clip.insert(db.get_ref()).await {
        eprintln!("Error creating clip: {}", err);
        return HttpResponse::InternalServerError().body("Failed to create clip");
    }

    // Kick off ffmpeg (async fire-and-forget)
    // Clip in the same directory + "/segments/"
    let clip_filepath = video_path.parent()
        .map(|p| {
            let clips_dir = p.join("segments");
            if !clips_dir.exists() {
                if let Err(err) = std::fs::create_dir_all(&clips_dir) {
                    eprintln!("Failed to create segments directory: {}", err);
                }
            }
            clips_dir.join(&clip_filename).display().to_string()
        })
        .unwrap_or_else(|| clip_filename.clone());
    match create_video_clip(&source_filename, form.start, form.end, &clip_filepath) {
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
