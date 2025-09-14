const HTML_HEADER: &str = r#"<!DOCTYPE html><html lang='en'><head><meta charset='UTF-8'><meta name='viewport' content='width=device-width, initial-scale=1'><title>File Server</title><link href='https://cdn.jsdelivr.net/npm/bootstrap@5.3.0/dist/css/bootstrap.min.css' rel='stylesheet'></head><body class='bg-light'><div class='container py-4'><h1 class='mb-4'>File Server</h1>"#;
const HTML_FOOTER: &str = "</div></body></html>";
mod models;
use actix_web::{web, App, HttpServer, HttpRequest};
use actix_session::{Session, SessionMiddleware};
use actix_web::cookie::Key;
use serde::Deserialize;
use std::env;
use sea_orm::{Database, DatabaseConnection};
use std::path::PathBuf;
mod controllers;
use controllers::files::{browse, upload, create_folder};
use controllers::login::{login_form, login as login_handler, logout as logout_handler, is_logged_in};
use controllers::signup::{signup, signup_form};

#[derive(Deserialize)]
struct LoginForm {
    username: String,
    password: String,
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
            .wrap(SessionMiddleware::builder(
                actix_session::storage::CookieSessionStore::default(),
                Key::from(&[0; 64]),
            )
            .cookie_secure(false)
            .build())
            .route("/login", web::get().to(login_form))
            .route("/login", web::post().to(login_handler))
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
