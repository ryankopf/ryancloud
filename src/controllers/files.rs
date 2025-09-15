use actix_web::{web, HttpResponse, HttpRequest, Result};
use actix_session::Session;
use actix_multipart::Multipart;
use futures_util::stream::StreamExt as _;
use actix_files::NamedFile;
use std::fs;
use actix_web::Error as ActixError;
use crate::{AppState, HTML_FOOTER, HTML_HEADER, is_logged_in};
use std::path::PathBuf; // Import PathBuf

// Unified handler: serve file if path is file, list if directory
pub async fn browse(
    folder: web::Data<PathBuf>,
    req: HttpRequest,
    path: Option<web::Path<String>>,
    session: Session,
) -> HttpResponse {
    let mut target = folder.get_ref().clone();
    let subpath = path.as_ref().map(|p| p.as_str()).unwrap_or("");
    if !subpath.is_empty() {
        target = target.join(subpath);
    }

    if target.is_file() {
        // Serve file for download
        NamedFile::open(target)
            .map(|file| file.into_response(&req))
            .unwrap_or_else(|_| HttpResponse::NotFound().finish())
    } else {
        // List directory contents
        let mut html = String::new();
        html += HTML_HEADER;
        html += "<div class='card'><div class='card-header'>File List</div><ul class='list-group list-group-flush'>";
        let video_extensions = ["mp4", "avi", "mov", "mkv", "webm"];

        match fs::read_dir(&target) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let file_name = entry.file_name().to_string_lossy().to_string();
                    let link = if subpath.is_empty() {
                        format!("/{}", file_name)
                    } else {
                        format!("/{}/{}", subpath, file_name)
                    };

                    // Check if the file is a video
                    let is_video = entry
                        .path()
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .map(|ext| video_extensions.contains(&ext.to_lowercase().as_str()))
                        .unwrap_or(false);

                    if is_video {
                        html += &format!(
                            "<li class='list-group-item'><a href='{link}'>{file_name}</a> <a href='/videos/{file_name}' target='_blank'>🎬</a></li>",
                            link = link,
                            file_name = file_name
                        );
                    } else {
                        html += &format!("<li class='list-group-item'><a href='{}'>{}</a></li>", link, file_name);
                    }
                }
            }
            Err(e) => {
                html += &format!("<li class='list-group-item text-danger'>Error reading directory '{}': {}</li>", target.display(), e);
            }
        }
        html += "</ul></div>";
        // Add login button
        if !is_logged_in(&session) {
            html += r#"<a class='btn btn-primary mt-3' href="/login">Login</a>"#;
        }
        // Only show upload and create folder if logged in
        if is_logged_in(&session) {
            html += r#"
            <div class="actions py-4">
                <button class='btn btn-success mt-2' type='button' data-bs-toggle='collapse' data-bs-target='#uploadForm' aria-expanded='false' aria-controls='uploadForm'>Upload Files</button>
                <button class='btn btn-secondary mt-2' type='button' data-bs-toggle='collapse' data-bs-target='#folderForm' aria-expanded='false' aria-controls='folderForm'>New Folder</button>
            </div>
            <div class='collapse my-4' id='uploadForm'>
                <form action='/upload' method='post' enctype='multipart/form-data' class='mb-2'>
                    <input type='file' name='files' multiple class='form-control mb-2'>
                    <button type='submit' class='btn btn-success'>Upload</button>
                </form>
            </div>
            <div class='collapse my-4' id='folderForm'>
                <form action='/create_folder' method='post' class='mb-2'>
                    <input type='text' name='folder_name' placeholder='New folder name' required class='form-control mb-2'>
                    <button type='submit' class='btn btn-secondary'>Create Folder</button>
                </form>
            </div>
            <form action='/logout' method='post' class='mt-4'><button type='submit' class='btn btn-outline-danger'>Logout</button></form>
            <script src='https://cdn.jsdelivr.net/npm/bootstrap@5.3.0/dist/js/bootstrap.bundle.min.js'></script>
            "#;
        }
        html += HTML_FOOTER;
        HttpResponse::Ok().content_type("text/html").body(html)
    }
}
// Handle folder creation
pub async fn create_folder(
    data: web::Data<AppState>,
    form: web::Form<std::collections::HashMap<String, String>>,
    session: Session,
) -> Result<HttpResponse, ActixError> {
    if !is_logged_in(&session) {
        return Ok(HttpResponse::Unauthorized().body("Login required"));
    }
    let folder_name = form.get("folder_name").map(|s| s.trim()).filter(|s| !s.is_empty());
    let folder_name = match folder_name {
        Some(name) => name,
        None => return Ok(HttpResponse::BadRequest().body("Invalid folder name")),
    };
    // Basic validation: no path traversal, only allow simple names
    if folder_name.contains('/') || folder_name.contains('\\') || folder_name.contains("..") {
        return Ok(HttpResponse::BadRequest().body("Invalid folder name"));
    }
    let mut target = data.folder.clone();
    target = target.join(folder_name);
    if target.exists() {
        return Ok(HttpResponse::BadRequest().body("Folder already exists"));
    }
    match std::fs::create_dir(&target) {
        Ok(_) => Ok(HttpResponse::Found().append_header(("Location", "/")).finish()),
        Err(e) => Ok(HttpResponse::InternalServerError().body(format!("Error creating folder: {}", e))),
    }
}

// Handle file uploads
pub async fn upload(
    data: web::Data<AppState>,
    mut payload: Multipart,
) -> Result<HttpResponse, ActixError> {
    let mut results = Vec::new();
    let target_dir = &data.folder;

    while let Some(item) = payload.next().await {
        let mut field = item?;

        // Extract filename into an owned String
        let filename: String = match field.content_disposition()
            .and_then(|cd| cd.get_filename().map(|f| f.to_string()))
        {
            Some(fname) => fname,
            None => {
                results.push(("<unknown>".to_string(), "No filename".to_string()));
                continue;
            }
        };

        let filepath = target_dir.join(&filename);
        if filepath.exists() {
            results.push((filename.clone(), "File exists, skipped".to_string()));
            continue;
        }

        let mut f = match std::fs::File::create(&filepath) {
            Ok(file) => file,
            Err(e) => {
                results.push((filename.clone(), format!("Error creating '{}': {}", filepath.display(), e)));
                continue;
            }
        };

        // Now safely consume the stream
        while let Some(chunk) = field.next().await {
            let data = chunk?;
            use std::io::Write;
            if let Err(e) = f.write_all(&data) {
                results.push((filename.clone(), format!("Write error to '{}': {}", filepath.display(), e)));
                break;
            }
        }

        results.push((filename.clone(), "Uploaded".to_string()));
    }

    let mut html = String::new();
    html += HTML_HEADER;
    html += "<div class='card'><div class='card-header'>Upload Results</div><ul class='list-group list-group-flush'>";
    for (file, status) in results {
        html += &format!("<li class='list-group-item'>{}: {}</li>", file, status);
    }
    html += "</ul></div><a class='btn btn-primary mt-3' href='/'>Back</a>";
    html += HTML_FOOTER;

    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}
