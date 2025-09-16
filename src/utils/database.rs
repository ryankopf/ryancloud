use directories::ProjectDirs;
use std::path::{PathBuf};

const DB_FILE: &str = "database.sqlite";

fn project_data_dir() -> Option<PathBuf> {
    ProjectDirs::from("com", "Ryan", "Cloud")
        .map(|proj_dirs| proj_dirs.data_dir().to_path_buf())
}

fn fallback_dir() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

pub fn db_path() -> PathBuf {
    let dir = project_data_dir().unwrap_or_else(fallback_dir);
    let _ = std::fs::create_dir_all(&dir);
    dir.join(DB_FILE)
}

pub fn get_database() -> PathBuf {
    db_path()
}

const CREATE_USERS_TABLE: &str = r#"
CREATE TABLE users (
    id INT PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    access_level TEXT NOT NULL
);
"#;
const CREATE_CLIPS_TABLE: &str = r#"
CREATE TABLE clips (
    id INT PRIMARY KEY AUTOINCREMENT,
    working_directory TEXT NOT NULL,
    source_filename TEXT NOT NULL,
    clip_filename TEXT NOT NULL,
    start BIGINT NOT NULL,
    end BIGINT NOT NULL,
    name TEXT,
    description TEXT
);
"#;
