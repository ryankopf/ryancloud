mod controllers;
mod models;
mod utils;
use actix_web::{web, App, HttpServer};
use actix_session::SessionMiddleware;
use actix_web::cookie::Key;
use serde::Deserialize;
use std::env;
use sea_orm::DatabaseConnection;
use std::path::PathBuf;
use controllers::login::is_logged_in;
use actix_web::middleware::Logger;
use dotenvy::from_path; // Updated to use dotenvy for environment variable loading
use std::process::Command;
use utils::database::{get_ffmpeg_path, set_ffmpeg_path};

#[derive(Deserialize)]
struct LoginForm {
    username: String,
    password: String,
}

struct AppState {
    folder: PathBuf,
    db: DatabaseConnection,
}

async fn handle_args(args: &[String], db: &DatabaseConnection) {
    if args.len() > 1 {
        for arg in &args[1..] {
            match arg.as_str() {
                "--where" => {
                    let db_path = utils::database::db_path();
                    println!("Database path: {:?}", db_path);
                }
                "--help" => {
                    println!("Usage: {} [OPTIONS]\n\nOptions:\n  --where       Print the path to the database file.\n  --help        Show this help message.\n  --folder=PATH Specify the folder to serve.\n  --set-ffmpeg=PATH Set the FFMPEG_PATH in the database.", args[0]);
                }
                _ if arg.starts_with("--folder=") => {
                    if let Some(path) = arg.strip_prefix("--folder=") {
                        println!("Folder argument provided: {}", path);
                    }
                }
                _ if arg.starts_with("--set-ffmpeg=") => {
                    if let Some(path) = arg.strip_prefix("--set-ffmpeg=") {
                        if let Err(e) = set_ffmpeg_path(db, path).await {
                            eprintln!("Failed to set FFMPEG_PATH: {}", e);
                        } else {
                            println!("FFMPEG_PATH set to: {}", path);
                        }
                    }
                }
                _ => {
                    println!("Unknown argument: {}", arg);
                }
            }
        }
        std::process::exit(0);
    }
}

#[actix_web::main]
async fn main() {
    match from_path(".env") {
        Ok(_) => println!("Environment variables loaded from .env"),
        Err(e) => eprintln!("Warning: Could not load .env file: {}", e),
    }

    let db = utils::database::get_database().await.unwrap_or_else(|e| {
        panic!("Failed to connect to database: {}", e);
    });

    let args: Vec<String> = env::args().collect();
    handle_args(&args, &db).await;

    // Check for FFMPEG.
    let ffmpeg_path = get_ffmpeg_path(&db).await.or_else(|| std::env::var("FFMPEG_PATH").ok());
    if let Some(ffmpeg_path) = ffmpeg_path {
        if Command::new(&ffmpeg_path).arg("-version").output().is_err() {
            eprintln!("Error: FFMPEG_PATH is set but the executable is not accessible or invalid.");
            std::process::exit(1);
        }
    } else {
        eprintln!("Error: FFMPEG_PATH is not set in the database or environment. Please set it using --set-ffmpeg=PATH or set the FFMPEG_PATH environment variable.");
        std::process::exit(1);
    }

    let folder = env::current_dir().unwrap();
    println!("Serving folder: {:?}", folder);

    let db_data = web::Data::new(db);
    let folder_data = web::Data::new(folder);

    let http_server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(db_data.clone())
            .app_data(folder_data.clone())
            .wrap(
                SessionMiddleware::builder(
                    actix_session::storage::CookieSessionStore::default(),
                    Key::from(&[0; 64]),
                )
                .cookie_secure(false)
                .build()
            )
            .configure(controllers::clips::clips_routes)
            .configure(controllers::login::login_routes)
            .configure(controllers::search::search_routes)
            .configure(controllers::signup::signup_routes)
            .configure(controllers::videos::video_routes)
            .configure(controllers::files::files_routes) // Must be last.
    });

    let server = match http_server.bind(("0.0.0.0", 80)) {
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
