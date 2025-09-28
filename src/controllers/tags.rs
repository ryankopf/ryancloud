use actix_web::{get, post, web, HttpResponse};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde::Deserialize;
use std::path::PathBuf;
use crate::models::tag;
use sea_orm::{DbBackend, Statement};
use sea_orm::ConnectionTrait;

#[get("{video_path:.*}/tags")]
pub async fn index(
	video_path: web::Path<PathBuf>,
	db: web::Data<DatabaseConnection>,
) -> HttpResponse {
	let video_path_str = video_path.display().to_string();

	// Fetch all tags associated with the given video path
	let tags = tag::Entity::find()
		.filter(tag::Column::SourceFilename.eq(video_path_str.clone()))
		.all(db.get_ref())
		.await;

	match tags {
		Ok(tags) => {
			let filename = video_path.file_name().map(|f| f.to_string_lossy()).unwrap_or_default();
			let tags_html = if !tags.is_empty() {
				tags
					.into_iter()
					.map(|tag| {
						let delete_button = format!(
							"<a href=\"#\" onclick=\"deleteTag({});return false;\" style='margin-left:8px;color:red;text-decoration:none;font-weight:bold;'>&times;</a>",
							tag.id
						);
						format!(
							"<div><span class='badge bg-info'>{}</span> {}</div>",
							tag.tag,
							delete_button,
						)
					})
					.collect::<String>()
			} else {
				"<p>No tags found.</p>".to_string()
			};
			let html = format!(
				"<div class='text-muted'>Tags for {}</div>{}",
				filename,
				tags_html
			);
			HttpResponse::Ok().content_type("text/html").body(html)
		}
		Err(err) => {
			eprintln!("Error fetching tags: {}", err);
			HttpResponse::InternalServerError().body("Internal server error")
		}
	}
}

#[derive(Deserialize)]
pub struct TagForm {
	pub tag: String,
}

#[post("{video_path:.*}/tags")]
pub async fn create(
	video_path: web::Path<PathBuf>,
	form: web::Form<TagForm>,
	db: web::Data<DatabaseConnection>,
) -> HttpResponse {
	let source_filename = video_path.display().to_string();

	// Log incoming data for debugging
	eprintln!("Received POST request for video_path: {}", source_filename);
	eprintln!("Form data: tag={}", form.tag);

	// Validate form data
	if form.tag.trim().is_empty() {
		return HttpResponse::BadRequest().body("Tag cannot be blank");
	}

	// Insert into DB
	let new_tag = tag::ActiveModel {
		source_filename: Set(source_filename.clone()),
		tag: Set(form.tag.clone()),
		slug: Set(tag::Model::normalize_tag(&form.tag)),
		..Default::default()
	};

	if let Err(err) = new_tag.insert(db.get_ref()).await {
		eprintln!("Error creating tag: {}", err);
		return HttpResponse::InternalServerError().body("Failed to create tag");
	}

	HttpResponse::Created().body("Tag created")
}

pub fn tags_routes(cfg: &mut web::ServiceConfig) {
	cfg.service(index);
	cfg.service(create);
}
