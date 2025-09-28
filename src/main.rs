mod controllers;
mod models;
mod tools;
mod utils;
use actix_web::{web, App, HttpServer};
use tools::conversions::process_conversion_queue;
use actix_web::cookie::Key;
use actix_web::middleware::Logger;
use actix_session::SessionMiddleware;
use dotenvy::from_path;
use std::env;
use std::process::Command;
use controllers::login::is_logged_in;
use utils::args::handle_args;
use utils::database::get_ffmpeg_path;
use utils::ssl::get_certificates;
use utils::redirect::redirect_to_https;

use std::fs::File;
use std::io::BufReader;
use rustls::ServerConfig;
use rustls_pemfile::{certs, pkcs8_private_keys};


fn load_rustls_config(cert_path: &std::path::Path, key_path: &std::path::Path) -> ServerConfig {
    let mut cert_reader = BufReader::new(File::open(cert_path).expect("Cannot open cert file"));
    let cert_chain = certs(&mut cert_reader)
        .collect::<Result<Vec<_>, _>>()
        .expect("Invalid certificate(s)");

    let mut key_reader = BufReader::new(File::open(key_path).expect("Cannot open key file"));
    let mut keys = pkcs8_private_keys(&mut key_reader)
        .collect::<Result<Vec<_>, _>>()
        .expect("Invalid private key");
    let key = keys.remove(0);

    ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert_chain, rustls::pki_types::PrivateKeyDer::Pkcs8(key))
        .expect("Bad cert/key pair")
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

    let tls_config = load_rustls_config(&cert_path, &key_path);

    let db_for_worker = db.clone();
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
                    Key::from(&[0; 64]), // left exactly as you had it
                )
                .cookie_secure(true)
                .build(),
            )
            .configure(controllers::ai::ai_routes)
            .configure(controllers::clips::clips_routes)
            .configure(controllers::points::points_routes)
            .configure(controllers::tags::tags_routes)
            .configure(controllers::login::login_routes)
            .configure(controllers::search::search_routes)
            .configure(controllers::signup::signup_routes)
            .configure(controllers::videos::video_routes)
            .configure(controllers::files::files_routes) // Must be last.
    });

    // HTTPS listener
    let https = https_server
        .bind_rustls_0_23(("0.0.0.0", 443), tls_config)
        .expect("Failed to bind to 443")
        .run();

    // HTTP server for redirects
    let http = HttpServer::new(|| {
        App::new().default_service(web::to(redirect_to_https))
    })
    .bind(("0.0.0.0", 80))
    .expect("Failed to bind to 80")
    .run();

    // Start the conversion queue processor as a background task
    let conversion_worker = tokio::spawn(async move {
        process_conversion_queue(&db_for_worker).await;
    });

    let (https_res, http_res, worker_res) = tokio::join!(https, http, conversion_worker);
    if let Err(e) = https_res {
        eprintln!("HTTPS server error: {}", e);
        std::process::exit(1);
    }
    if let Err(e) = http_res {
        eprintln!("HTTP server error: {}", e);
        std::process::exit(1);
    }
    if let Err(e) = worker_res {
        eprintln!("Conversion worker task failed: {}", e);
        std::process::exit(1);
    }
}
