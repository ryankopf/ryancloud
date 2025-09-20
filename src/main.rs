mod controllers;
mod models;
mod utils;
use actix_web::{web, App, HttpServer};
use actix_web::cookie::Key;
use actix_web::middleware::Logger;
use actix_session::SessionMiddleware;
use dotenvy::from_path;
use serde::Deserialize;
use std::env;
use sea_orm::DatabaseConnection;
use std::path::PathBuf;
use std::process::Command;
use controllers::login::is_logged_in;
use utils::args::handle_args;
use utils::database::get_ffmpeg_path;
use utils::ssl::get_certificates;
use utils::redirect::redirect_to_https;

use std::fs::File;
use std::io::BufReader;
// use rustls::{pki_types::CertificateDer, pki_types::PrivateKeyDer, ServerConfig};
// use rustls_pemfile::{certs, pkcs8_private_keys};

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

    let (cert_path, key_path) = get_certificates().unwrap_or_else(|e| {
        eprintln!("Failed to prepare certificates: {}", e);
        std::process::exit(1);
    });
    let _cert_file = &mut BufReader::new(File::open(&cert_path).unwrap());
    let _key_file = &mut BufReader::new(File::open(&key_path).unwrap());


    let db_data = web::Data::new(db);
    let folder_data = web::Data::new(folder);

    let https_server = HttpServer::new(move || {
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
                .build(),
            )
            .configure(controllers::clips::clips_routes)
            .configure(controllers::login::login_routes)
            .configure(controllers::search::search_routes)
            .configure(controllers::signup::signup_routes)
            .configure(controllers::videos::video_routes)
            .configure(controllers::files::files_routes) // Must be last.
    });

    let server = match https_server.bind(("0.0.0.0", 80)) {
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

    // // HTTP server for redirects
    // let http_server = HttpServer::new(|| {
    //     App::new().route("{_:.*}", web::get().to(redirect_to_https))
    // })
    // .bind("0.0.0.0:80").unwrap()
    // .run();

    // // Use `future::join` if you're using the `futures` crate to run both servers concurrently.
    // let (_https_result, _http_result) = futures::future::join(https_server, http_server).await;

}
