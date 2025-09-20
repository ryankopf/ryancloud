use directories::ProjectDirs;
use sea_orm::{ActiveModelTrait, ConnectionTrait, Database, DatabaseConnection, DbBackend, DbErr, IntoActiveModel, Statement};
use sea_orm::EntityTrait;
use std::path::{PathBuf};
use crate::models::settings::Entity as SettingsEntity;

const DB_FILE: &str = "database.sqlite";

pub fn project_data_dir() -> Option<PathBuf> {
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

pub async fn get_database() -> Result<DatabaseConnection, DbErr> {
    let db_path = db_path();

    if !db_path.exists() {
        let _ = std::fs::File::create(&db_path);

        let db_url = format!("sqlite://{}", db_path.to_string_lossy());
        let db = Database::connect(&db_url).await?;

        db.execute(Statement::from_string(
            DbBackend::Sqlite,
            CREATE_USERS_TABLE.to_string(),
        )).await?;

        db.execute(Statement::from_string(
            DbBackend::Sqlite,
            CREATE_CLIPS_TABLE.to_string(),
        )).await?;

        db.execute(Statement::from_string(
            DbBackend::Sqlite,
            CREATE_SETTINGS_TABLE.to_string(),
        )).await?;

        return Ok(db);
    }

    // Connect to the existing database
    let db_url = format!("sqlite://{}", db_path.to_string_lossy());
    Database::connect(&db_url).await
}

const CREATE_USERS_TABLE: &str = r#"
CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    access_level TEXT NOT NULL
);
"#;
const CREATE_CLIPS_TABLE: &str = r#"
CREATE TABLE clips (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    working_directory TEXT NOT NULL,
    source_filename TEXT NOT NULL,
    clip_filename TEXT NOT NULL,
    start BIGINT NOT NULL,
    end BIGINT NOT NULL,
    name TEXT,
    description TEXT
);
"#;
const CREATE_SETTINGS_TABLE: &str = r#"
CREATE TABLE settings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ffmpeg_path TEXT NOT NULL
);
"#;

pub async fn get_ffmpeg_path(db: &DatabaseConnection) -> Option<String> {
    SettingsEntity::find()
        .one(db)
        .await
        .ok()
        .flatten()
        .map(|settings| settings.ffmpeg_path)
}

pub async fn set_ffmpeg_path(db: &DatabaseConnection, path: &str) -> Result<(), DbErr> {
    let mut settings: crate::models::settings::ActiveModel = SettingsEntity::find()
        .one(db)
        .await?
        .map(|m| m.into_active_model())
        .unwrap_or_else(|| crate::models::settings::ActiveModel {
            id: Default::default(),
            ffmpeg_path: Default::default(),
        });

    settings.ffmpeg_path = sea_orm::ActiveValue::Set(path.to_string());
    settings.update(db).await.map(|_| ())
}
