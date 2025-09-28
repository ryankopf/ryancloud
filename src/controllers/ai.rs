use actix_web::{post, web, HttpResponse};
use actix_files::NamedFile;
use sea_orm::{ActiveModelTrait, Set, DatabaseConnection};
use std::path::PathBuf;
use crate::models::conversion;

#[post("{video_path:.*}/categorize")]
pub async fn categorize_video(
	video_path: web::Path<PathBuf>,
	db: web::Data<DatabaseConnection>,
) -> HttpResponse {
	let source_filename = video_path.display().to_string();

	// Create a new conversion request for this video
	let now = std::time::SystemTime::now()
		.duration_since(std::time::UNIX_EPOCH)
		.map(|d| d.as_secs() as i64)
		.unwrap_or(0);

	let new_conversion = conversion::ActiveModel {
		source_filename: Set(source_filename.clone()),
		operation: Set("categorize".to_string()),
		time_requested: Set(now),
		time_completed: Set(None),
		status: Set("pending".to_string()),
		..Default::default()
	};

	match new_conversion.insert(db.get_ref()).await {
		Ok(model) => {
			HttpResponse::Created().body(format!("Conversion request created with id {} for {}", model.id, source_filename))
		}
		Err(err) => {
			eprintln!("Error creating conversion: {}", err);
			HttpResponse::InternalServerError().body("Failed to create conversion request")
		}
	}
}

#[actix_web::get("/categorize/{id}.jpg")]
pub async fn get_categorize_jpg(
	id: web::Path<i32>,
) -> actix_web::Result<NamedFile> {
	let path = format!("segments/ai/conversions/{}.jpg", id);
	Ok(NamedFile::open(path)?)
}

pub fn ai_routes(cfg: &mut web::ServiceConfig) {
	cfg.service(categorize_video);
	cfg.service(get_categorize_jpg);
}
