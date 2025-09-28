use actix_web::{get, post, delete, web, HttpResponse};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde::Deserialize;
use std::path::PathBuf;
use crate::models::tag;

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
							"<button type='button' class='btn btn-link text-danger p-0 ms-2' hx-delete='/tags/{}' hx-target='.tags-list' hx-swap='outerHTML' aria-label='Delete'>&times;</button>",
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
				"<div class='text-muted mt-3'>Tags for {}</div>{}<button class='badge bg-primary border-0' hx-get='/{}/tags/new' hx-target='#new-tag-form' hx-swap='innerHTML'>+ New</button><div id='new-tag-form' class='mt-2'></div>",
				filename,
				tags_html,
				video_path_str.trim_start_matches('/')
			);
			HttpResponse::Ok().content_type("text/html").body(html)
		}
		Err(err) => {
			eprintln!("Error fetching tags: {}", err);
			HttpResponse::InternalServerError().body("Internal server error")
		}
	}
}

// HTMX endpoint: returns a form for creating a new tag for a specific video path
#[get("{video_path:.*}/tags/new")]
pub async fn new(video_path: web::Path<PathBuf>) -> HttpResponse {
	let video_path_str = video_path.display().to_string();
	let action_path = format!("/{}/tags", video_path_str.trim_start_matches('/'));
	let form_html = format!(r#"
<form hx-post="{}" hx-target=".tags-list" hx-swap="innerHTML" class="d-flex align-items-center gap-2 mt-2">
    <input type="text" name="tag" class="form-control form-control-sm" placeholder="Enter tag" required style="max-width:150px;">
    <button type="submit" class="btn btn-primary btn-sm">Add</button>
</form>
"#, action_path);
	HttpResponse::Ok().content_type("text/html").body(form_html)
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
	eprintln!("Received POST tag for video_path: {}", source_filename);
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

	// After successful insert, return a standard 303 redirect to the tags index for this video
	let redirect_url = format!("/{}/tags", source_filename.trim_start_matches('/'));
	HttpResponse::SeeOther()
		.append_header(("Location", redirect_url))
		.finish()
}

#[delete("/tags/{tag_id}")]
pub async fn delete(
	tag_id: web::Path<i32>,
	db: web::Data<DatabaseConnection>,
) -> HttpResponse {
	use sea_orm::EntityTrait;
	use crate::models::tag::Entity as TagEntity;

	// Find the tag to get its source_filename for reloading the list
	let tag = TagEntity::find_by_id(*tag_id).one(db.get_ref()).await;
	let source_filename = match tag {
		Ok(Some(tag)) => tag.source_filename.clone(),
		_ => return HttpResponse::NotFound().body("Tag not found"),
	};

	// Delete the tag
	if let Err(err) = TagEntity::delete_by_id(*tag_id).exec(db.get_ref()).await {
		eprintln!("Error deleting tag: {}", err);
		return HttpResponse::InternalServerError().body("Failed to delete tag");
	}

	// Fetch all tags associated with the given video path
	let tags = TagEntity::find()
		.filter(tag::Column::SourceFilename.eq(source_filename.clone()))
		.all(db.get_ref())
		.await;

	match tags {
		Ok(tags) => {
			let filename = std::path::Path::new(&source_filename)
				.file_name()
				.map(|f| f.to_string_lossy())
				.unwrap_or_default();
			let tags_html = if !tags.is_empty() {
				tags
					.into_iter()
					.map(|tag| {
						let delete_button = format!(
							"<button type='button' class='btn btn-link text-danger p-0 ms-2' hx-delete='/tags/{}' hx-target='#tags-list' hx-swap='outerHTML' aria-label='Delete'>&times;</button>",
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
				"<div id='tags-list'><div class='text-muted mt-3'>Tags for {}</div>{}<button class='badge bg-primary border-0' hx-get='/{}'/tags/new' hx-target='#new-tag-form' hx-swap='innerHTML'>+ New</button><div id='new-tag-form' class='mt-2'></div></div>",
				filename,
				tags_html,
				source_filename.trim_start_matches('/')
			);
			HttpResponse::Ok().content_type("text/html").body(html)
		}
		Err(err) => {
			eprintln!("Error fetching tags: {}", err);
			HttpResponse::InternalServerError().body("Internal server error")
		}
	}
}

pub fn tags_routes(cfg: &mut web::ServiceConfig) {
	cfg.service(index);
	cfg.service(new);
	cfg.service(create);
	cfg.service(delete);
}
