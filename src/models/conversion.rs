use crate::models::{conversion, tag};
use sea_orm::{ActiveModelTrait, Set};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

// Operations: Thumbnail, Scaledown, Makeclip, Categorize
// Status: Pending, Running, Completed, Failed

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, DeriveEntityModel)]
#[sea_orm(table_name = "conversions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[serde(skip)]
    pub source_filename: String,
    pub operation: String,
    pub time_requested: i64,
    pub time_completed: Option<i64>,
    pub status: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub async fn process(&self, db: &DatabaseConnection) -> Result<(), sea_orm::DbErr> {
        match Operation::from_str_case_insensitive(&self.operation) {
            Some(Operation::Thumbnail) => {
                // TODO: Implement thumbnail generation
            }
            Some(Operation::Scaledown) => {
                // TODO: Implement scaledown logic
            }
            Some(Operation::Makeclip) => {
                // TODO: Implement makeclip logic
            }
            Some(Operation::Categorize) => {
                use std::process::{Command, Stdio};
                use std::path::Path;
                // Extract a frame at 1s using ffmpeg
                let ffmpeg_path = std::env::var("FFMPEG_PATH").map_err(|_| sea_orm::DbErr::Custom("FFMPEG_PATH not defined in environment".into()))?;
                let output_dir = Path::new("segments/ai/conversions");
                if !output_dir.exists() {
                    std::fs::create_dir_all(output_dir).map_err(|e| sea_orm::DbErr::Custom(format!("Failed to create output directory: {}", e)))?;
                }
                let output_path = output_dir.join(format!("{}.jpg", self.id));
                let output_path_str = output_path.to_string_lossy().to_string();
                let args = vec![
                    "-ss", "1",
                    "-i", &self.source_filename,
                    "-frames:v", "1",
                    "-q:v", "2",
                    &output_path_str,
                ];
                let status = Command::new(&ffmpeg_path)
                    .args(&args)
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()
                    .map_err(|e| sea_orm::DbErr::Custom(format!("Failed to start ffmpeg: {}", e)))?;
                if !status.success() {
                    return Err(sea_orm::DbErr::Custom(format!("ffmpeg failed with exit code: {}", status.code().unwrap_or(-1))));
                }
                // Call the AI tagging function on output_path
                // The AI tagging function expects a URL, so you may need to construct a URL to the image
                // For now, assume the server is running on localhost and port 443 (HTTPS)
                let image_url = format!("https://media.aiowa.com/categorize/{}.jpg", self.id);
                match crate::tools::ai::tag_image(&image_url).await {
                    Ok(tags) => {
                        for tag_str in &tags.tags {
                            let new_tag = tag::ActiveModel {
                                source_filename: Set(self.source_filename.clone()),
                                tag: Set(tag_str.clone()),
                                slug: Set(tag::Model::normalize_tag(&tag_str)),
                                ..Default::default()
                            };
                            if let Err(e) = new_tag.insert(db).await {
                                eprintln!("Failed to insert tag '{}': {}", tag_str, e);
                            }
                        }
                        // Optionally, save tags.description somewhere as well
                        println!("AI tags: {:?}", tags);
                        // Update conversion status to completed
                        let mut am: conversion::ActiveModel = self.clone().into();
                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_secs() as i64)
                            .unwrap_or(0);
                        am.status = Set("completed".to_string());
                        am.time_completed = Set(Some(now));
                        if let Err(e) = am.update(db).await {
                            eprintln!("Failed to update conversion status: {}", e);
                        }
                    }
                    Err(e) => {
                        eprintln!("AI tagging failed: {}", e);
                        // Update conversion status to failed
                        let mut am: conversion::ActiveModel = self.clone().into();
                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_secs() as i64)
                            .unwrap_or(0);
                        am.status = Set("failed".to_string());
                        am.time_completed = Set(Some(now));
                        if let Err(e) = am.update(db).await {
                            eprintln!("Failed to update conversion status: {}", e);
                        }
                    }
                }
            }
            None => {
                // Unknown operation
            }
        }
        Ok(())
    }
}




#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Text")]
pub enum Operation {
    #[sea_orm(string_value = "thumbnail")]
    Thumbnail,
    #[sea_orm(string_value = "scaledown")]
    Scaledown,
    #[sea_orm(string_value = "makeclip")]
    Makeclip,
    #[sea_orm(string_value = "categorize")]
    Categorize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Text")]
pub enum Status {
    #[sea_orm(string_value = "pending")]
    Pending,
    #[sea_orm(string_value = "running")]
    Running,
    #[sea_orm(string_value = "completed")]
    Completed,
    #[sea_orm(string_value = "failed")]
    Failed,
}

impl Operation {
    pub fn from_str_case_insensitive(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "thumbnail" => Some(Operation::Thumbnail),
            "scaledown" => Some(Operation::Scaledown),
            "makeclip" => Some(Operation::Makeclip),
            "categorize" => Some(Operation::Categorize),
            _ => None,
        }
    }
}