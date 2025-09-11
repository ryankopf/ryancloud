
use actix_web::{web, App, HttpServer, HttpResponse, HttpRequest};
use actix_files::NamedFile;
use std::env;
use std::path::{PathBuf};
use std::fs;



use actix_web::{Error as ActixError};

// Unified handler: serve file if path is file, list if directory
async fn browse(data: web::Data<AppState>, req: HttpRequest, path: Option<web::Path<String>>) -> Result<actix_web::HttpResponse, ActixError> {
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
                    let file_path = entry.path();
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
        Ok(HttpResponse::Ok().content_type("text/html").body(html))
    }
}

struct AppState {
    folder: PathBuf,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let folder = if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        env::current_dir().unwrap()
    };
    println!("Serving folder: {:?}", folder);
    let state = web::Data::new(AppState { folder });
    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .route("/", web::get().to(|data: web::Data<AppState>, req: HttpRequest| browse(data, req, None)))
            .route("/{path:.*}", web::get().to(|data: web::Data<AppState>, req: HttpRequest, path: web::Path<String>| browse(data, req, Some(path))))
    })
    .bind(("0.0.0.0", 80))?
    .run()
    .await
}
