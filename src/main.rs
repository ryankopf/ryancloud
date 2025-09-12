const HTML_HEADER: &str = r#"<!DOCTYPE html><html lang='en'><head><meta charset='UTF-8'><meta name='viewport' content='width=device-width, initial-scale=1'><title>File Server</title><link href='https://cdn.jsdelivr.net/npm/bootstrap@5.3.0/dist/css/bootstrap.min.css' rel='stylesheet'></head><body class='bg-light'><div class='container py-4'><h1 class='mb-4'>File Server</h1>"#;
const HTML_FOOTER: &str = "</div></body></html>";
mod models;
use actix_web::{web, App, HttpServer, HttpResponse, HttpRequest, Result};
use actix_session::{Session, SessionMiddleware};
use actix_web::cookie::Key;
use crate::models::auth::{is_logged_in, login as login_handler, logout as logout_handler};
use serde::Deserialize;
use actix_multipart::Multipart;
use futures_util::stream::StreamExt as _;
use actix_files::NamedFile;
use std::env;
use sea_orm::{Database, DatabaseConnection};
use std::path::PathBuf;
use std::fs;
use actix_web::Error as ActixError;
use crate::models::user::ActiveModel;
use sea_orm::{Set, ActiveModelTrait};
use bcrypt;


#[derive(Deserialize)]
struct LoginForm {
    username: String,
    password: String,
}

// Signup form (GET)
async fn signup_form() -> HttpResponse {
    let html = r#"
        <h1>Sign Up</h1>
        <form action="/signup" method="post">
            <input type="text" name="username" placeholder="Username" required><br>
            <input type="password" name="password" placeholder="Password" required><br>
            <button type="submit">Sign Up</button>
        </form>
        <a href="/">Back</a>
    "#;
    HttpResponse::Ok().content_type("text/html").body(html)
}

// Signup handler (POST)
async fn signup(
    data: web::Data<AppState>,
    form: web::Form<LoginForm>,
) -> Result<HttpResponse, ActixError> {
    let password_hash = bcrypt::hash(&form.password, bcrypt::DEFAULT_COST).unwrap();
    let user = ActiveModel {
        username: Set(form.username.clone()),
        password_hash: Set(password_hash),
        access_level: Set("None".to_string()),
        ..Default::default()
    };
    match user.insert(&data.db).await {
        Ok(_) => Ok(HttpResponse::Found().append_header(("Location", "/login")).finish()),
        Err(e) => Ok(HttpResponse::InternalServerError().body(format!("Error: {}", e))),
    }
}

// Serve login form (GET)
async fn login_form() -> HttpResponse {
    let html = r#"
        <h1>Login</h1>
        <form action="/login" method="post">
            <input type="text" name="username" placeholder="Username" required><br>
            <input type="password" name="password" placeholder="Password" required><br>
            <button type="submit">Login</button>
        </form>
        <a href="/">Back</a>
    "#;
    HttpResponse::Ok().content_type("text/html").body(html)
}

// Unified handler: serve file if path is file, list if directory
async fn browse(
    data: web::Data<AppState>,
    req: HttpRequest,
    path: Option<web::Path<String>>,
    session: Session,
) -> Result<HttpResponse, ActixError> {
    let mut target = data.folder.clone();
    let subpath = path.as_ref().map(|p| p.as_str()).unwrap_or("");
    if !subpath.is_empty() {
        target = target.join(subpath);
    }

    if target.is_file() {
        // Serve file for download
        Ok(NamedFile::open(target)?.into_response(&req))
    } else {
        // List directory contents
    let mut html = String::new();
    html += HTML_HEADER;
    html += "<div class='card'><div class='card-header'>File List</div><ul class='list-group list-group-flush'>";
        match fs::read_dir(&target) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let file_name = entry.file_name().to_string_lossy().to_string();
                    let link = if subpath.is_empty() {
                        format!("/{}", file_name)
                    } else {
                        format!("/{}/{}", subpath, file_name)
                    };
                    html += &format!("<li class='list-group-item'><a href='{}'>{}</a></li>", link, file_name);
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
        Ok(HttpResponse::Ok().content_type("text/html").body(html))
    }
}
// Handle folder creation
async fn create_folder(
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
async fn upload(
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


struct AppState {
    folder: PathBuf,
    db: DatabaseConnection,
}


#[actix_web::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let folder = if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        env::current_dir().unwrap()
    };

    // Set up database connection (update with your DB URL as needed)
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://users.db".to_string());
    let db = Database::connect(&db_url).await.unwrap_or_else(|e| {
        panic!("Failed to connect to DB at '{}': {}", db_url, e);
    });

    println!("Serving folder: {:?}", folder);
    let state = web::Data::new(AppState { folder, db });

    let server = HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .wrap(SessionMiddleware::new(
                actix_session::storage::CookieSessionStore::default(),
                Key::from(&[0; 64]),
            ))
            .route("/login", web::get().to(login_form))
            .route("/login", web::post().to(|data: web::Data<AppState>, session: Session, form: web::Form<LoginForm>| async move {
                let tuple_form = (form.username.clone(), form.password.clone());
                login_handler(web::Data::new(data.db.clone()), session, web::Form(tuple_form)).await
            }))
            .route("/logout", web::post().to(|session: Session| async move {
                logout_handler(session).await
            }))
            .route("/signup", web::get().to(signup_form))
            .route("/signup", web::post().to(signup))
            .route("/upload", web::post().to(upload))
            .route("/create_folder", web::post().to(create_folder))
            .route("/", web::get().to(|data: web::Data<AppState>, req: HttpRequest, session: Session| {
                browse(data, req, None, session)
            }))
            .route(
                "/{path:.*}",
                web::get().to(
                    |data: web::Data<AppState>, req: HttpRequest, path: web::Path<String>, session: Session| {
                        browse(data, req, Some(path), session)
                    },
                ),
            )
    });

    let server = match server.bind(("0.0.0.0", 80)) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to bind to 0.0.0.0:80: {}", e);
            std::process::exit(1);
        }
    };

    if let Err(e) = server.run().await {
        eprintln!("Server error: {}", e);
        std::process::exit(1);
    }
}
