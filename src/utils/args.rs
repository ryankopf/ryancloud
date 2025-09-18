use sea_orm::DatabaseConnection;
use crate::utils::database::set_ffmpeg_path;

pub async fn handle_args(args: &[String], db: &DatabaseConnection) {
    if args.len() > 1 {
        for arg in &args[1..] {
            match arg.as_str() {
                "--where" => {
                    let db_path = crate::utils::database::db_path();
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