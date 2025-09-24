use actix_web::{get, post, web, HttpResponse};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde::Deserialize;
use std::path::PathBuf;
use crate::models::point;

#[get("{video_path:.*}/points")]
pub async fn index(
	video_path: web::Path<PathBuf>,
	db: web::Data<DatabaseConnection>,
) -> HttpResponse {
	let video_path_str = video_path.display().to_string();
	let video_path_obj = PathBuf::from(&video_path_str);
	let videopath = video_path_obj.parent().map(|p| p.display().to_string()).unwrap_or_default();

	// Fetch all points associated with the given video path
	let points = point::Entity::find()
		.filter(point::Column::SourceFilename.eq(video_path_str.clone()))
		.all(db.get_ref())
		.await;

	match points {
		Ok(points) => {
			let points_html = if !points.is_empty() {
				points
					.into_iter()
					.map(|point| {
						// Format milliseconds to HH:MM:SS:ms
						let total_ms = point.time;
						let ms = total_ms % 1000;
						let total_seconds = total_ms / 1000;
						let s = total_seconds % 60;
						let total_minutes = total_seconds / 60;
						let m = total_minutes % 60;
						let h = total_minutes / 60;
						let formatted_time = format!("{:02}:{:02}:{:02}:{:03}", h, m, s, ms);
						let time_anchor = format!(
							"<a href=\"#\" onclick=\"jumpToPoint({});return false;\">{}</a>",
							total_ms, formatted_time
						);
						format!(
							"<div>{} {}</div>",
							time_anchor,
							point.name.unwrap_or_else(|| "Untitled".to_string())
						)
					})
					.collect::<String>()
			} else {
				"<p>No points found.</p>".to_string()
			};
			let html = format!(
				"<html><body><h6>Points for {}</h6>{}</body></html>",
				video_path_str, points_html
			);
			HttpResponse::Ok().content_type("text/html").body(html)
		}
		Err(err) => {
			eprintln!("Error fetching points: {}", err);
			HttpResponse::InternalServerError().body("Internal server error")
		}
	}
}

#[derive(Deserialize)]
pub struct PointForm {
	pub time: i64,
	pub name: Option<String>,
}

#[post("{video_path:.*}/points")]
pub async fn create(
	video_path: web::Path<PathBuf>,
	form: web::Form<PointForm>,
	db: web::Data<DatabaseConnection>,
) -> HttpResponse {
	let source_filename = video_path.display().to_string();
	// let working_directory = video_path.parent()
	// 	.map(|p| p.display().to_string())
	// 	.unwrap_or_else(|| "".to_string());

	// Log incoming data for debugging
	eprintln!("Received POST request for video_path: {}", source_filename);
	eprintln!("Form data: time={}, name={:?}", form.time, form.name);

	// Validate form data
	if form.time < 0 {
		return HttpResponse::BadRequest().body("Invalid point: 'time' must be non-negative");
	}

	if form.name.as_ref().map_or(true, |name| name.trim().is_empty()) {
		return HttpResponse::BadRequest().body("Point name cannot be blank");
	}

	// Insert into DB
	let new_point = point::ActiveModel {
		source_filename: Set(source_filename.clone()),
		time: Set(form.time),
		name: Set(form.name.clone()),
		// working_directory: Set(working_directory), // Uncomment if model has this field
		..Default::default()
	};

	if let Err(err) = new_point.insert(db.get_ref()).await {
		eprintln!("Error creating point: {}", err);
		return HttpResponse::InternalServerError().body("Failed to create point");
	}

	HttpResponse::Created().body("Point created")
}

pub fn points_routes(cfg: &mut web::ServiceConfig) {
	cfg.service(index);
	cfg.service(create);
}
