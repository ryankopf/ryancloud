// Serve login form (GET)
async fn login_form() -> HttpResponse {
    let html = r#"
        <h1>Login</h1>
        <form action=\"/login\" method=\"post\">
            <input type=\"text\" name=\"username\" placeholder=\"Username\" required><br>
            <input type=\"password\" name=\"password\" placeholder=\"Password\" required><br>
            <button type=\"submit\">Login</button>
        </form>
        <a href=\"/\">Back</a>
    "#;
    HttpResponse::Ok().content_type("text/html").body(html)
}
mod models;
use actix_web::{web, App, HttpServer, HttpResponse, HttpRequest, Result};
use actix_session::{Session, SessionMiddleware};
use actix_web::cookie::Key;
use crate::models::auth::{is_logged_in, login as login_handler, logout as logout_handler};
use actix_multipart::Multipart;
use futures_util::stream::StreamExt as _;
use actix_files::NamedFile;
use std::env;
use sea_orm::{Database, DatabaseConnection};
use std::path::PathBuf;
use std::fs;
use actix_web::Error as ActixError;

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
        let mut html = String::from("<h1>File List</h1><ul>");
        match fs::read_dir(&target) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let file_name = entry.file_name().to_string_lossy().to_string();
                    let link = if subpath.is_empty() {
                        format!("/{}", file_name)
                    } else {
                        format!("/{}/{}", subpath, file_name)
                    };
                    html += &format!("<li><a href=\"{}\">{}</a></li>", link, file_name);
                }
            }
            Err(e) => {
                html += &format!("<li>Error reading directory: {}</li>", e);
            }
        }
        html += "</ul>";
        // Add login button
        if !is_logged_in(&session) {
            html += r#"<a href="/login">Login</a>"#;
        }
        // Only show upload if logged in
        if is_logged_in(&session) {
            html += r#"
            <form action="/upload" method="post" enctype="multipart/form-data">
                <input type="file" name="files" multiple>
                <button type="submit">Upload</button>
            </form>
            "#;
            html += r#"<form action="/logout" method="post"><button type="submit">Logout</button></form>"#;
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
        Ok(HttpResponse::Ok().content_type("text/html").body(html))
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
                results.push((filename.clone(), format!("Error: {}", e)));
                continue;
            }
        };

        // Now safely consume the stream
        while let Some(chunk) = field.next().await {
            let data = chunk?;
            use std::io::Write;
            if let Err(e) = f.write_all(&data) {
                results.push((filename.clone(), format!("Write error: {}", e)));
                break;
            }
        }

        results.push((filename.clone(), "Uploaded".to_string()));
    }

    let mut html = String::from("<h1>Upload Results</h1><ul>");
    for (file, status) in results {
        html += &format!("<li>{}: {}</li>", file, status);
    }
    html += "</ul><a href=\"/\">Back</a>";

    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}


struct AppState {
    folder: PathBuf,
    db: DatabaseConnection,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let folder = if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        env::current_dir().unwrap()
    };

    // Set up database connection (update with your DB URL as needed)
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://users.db".to_string());
    let db = Database::connect(&db_url).await.expect("Failed to connect to DB");

    println!("Serving folder: {:?}", folder);
    let state = web::Data::new(AppState { folder, db });

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .wrap(SessionMiddleware::new(
                actix_session::storage::CookieSessionStore::default(),
                Key::from(&[0; 64]),
            ))
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
            .route("/upload", web::post().to(upload))
            // Login form (GET)
            .route("/login", web::get().to(login_form))
            // Login handler (POST)
            .route("/login", web::post().to(|data: web::Data<AppState>, session: Session, form: web::Form<(String, String)>| async move {
                login_handler(web::Data::new(data.db.clone()), session, form).await
            }))
            // Logout handler (POST)
            .route("/logout", web::post().to(|session: Session| async move {
                logout_handler(session).await
            }))
    })
    .bind(("0.0.0.0", 80))?
    .run()
    .await
}
