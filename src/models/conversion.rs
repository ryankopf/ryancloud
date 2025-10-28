use crate::models::{conversion, tag};
use sea_orm::{ActiveModelTrait, Set};
use sea_orm::EntityTrait;
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
    pub times_tried: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Request a conversion operation. Returns true if a new conversion was created, false if one already exists.
    /// If a conversion exists but was requested over 1 hour ago, creates a new one with incremented times_tried.
    pub async fn request_conversion(
        db: &DatabaseConnection,
        source_filename: String,
        operation: String,
    ) -> Result<bool, sea_orm::DbErr> {
        use sea_orm::{ColumnTrait, QueryFilter};
        
        // Check for existing conversion with same source_filename and operation
        let existing = Entity::find()
            .filter(Column::SourceFilename.eq(&source_filename))
            .filter(Column::Operation.eq(&operation))
            .filter(
                Column::Status.eq("pending")
                    .or(Column::Status.eq("running"))
            )
            .one(db)
            .await?;
        
        if let Some(existing_conversion) = existing {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);
            
            let time_diff = now - existing_conversion.time_requested;
            
            // If less than 1 hour (3600 seconds), don't create a new one
            if time_diff < 3600 {
                println!("Conversion already pending/running for {} ({}), skipping", source_filename, operation);
                return Ok(false);
            }
            
            // More than 1 hour old, create a new one with incremented times_tried
            println!("Existing conversion for {} ({}) is over 1 hour old, creating new attempt", source_filename, operation);
            let new_conversion = ActiveModel {
                source_filename: Set(source_filename),
                operation: Set(operation),
                time_requested: Set(now),
                time_completed: Set(None),
                status: Set("pending".to_string()),
                times_tried: Set(existing_conversion.times_tried + 1),
                ..Default::default()
            };
            
            new_conversion.insert(db).await?;
            return Ok(true);
        }
        
        // No existing conversion, create a new one
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        
        let new_conversion = ActiveModel {
            source_filename: Set(source_filename),
            operation: Set(operation),
            time_requested: Set(now),
            time_completed: Set(None),
            status: Set("pending".to_string()),
            times_tried: Set(1),
            ..Default::default()
        };
        
        new_conversion.insert(db).await?;
        Ok(true)
    }

    pub async fn process(&self, db: &DatabaseConnection) -> Result<(), sea_orm::DbErr> {
        match Operation::from_str_case_insensitive(&self.operation) {
            Some(Operation::Thumbnail) => {
                use std::path::Path;
                
                // Get ffmpeg path from environment or database
                let ffmpeg_path = crate::utils::database::get_ffmpeg_path(db).await
                    .or_else(|| std::env::var("FFMPEG_PATH").ok())
                    .ok_or_else(|| sea_orm::DbErr::Custom("FFMPEG_PATH not defined".into()))?;
                
                // Determine output path: source_filename -> source_filename/thumbs/filename.webp
                let source_path = Path::new(&self.source_filename);
                let parent = source_path.parent().ok_or_else(|| sea_orm::DbErr::Custom("Invalid source path".into()))?;
                let file_stem = source_path.file_stem().ok_or_else(|| sea_orm::DbErr::Custom("Invalid filename".into()))?;
                
                let thumbs_dir = parent.join("thumbs");
                if !thumbs_dir.exists() {
                    std::fs::create_dir_all(&thumbs_dir).map_err(|e| sea_orm::DbErr::Custom(format!("Failed to create thumbs directory: {}", e)))?;
                }
                
                let output_path = thumbs_dir.join(format!("{}.webp", file_stem.to_string_lossy()));
                let output_path_str = output_path.to_string_lossy().to_string();
                
                println!("Generating thumbnail: {} -> {}", self.source_filename, output_path_str);
                
                // Use the Thumb::generate function
                match crate::models::thumb::Thumb::generate(&self.source_filename, &output_path_str, &ffmpeg_path) {
                    Ok(_) => {
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
                        println!("Thumbnail generated successfully: {}", output_path_str);
                    }
                    Err(e) => {
                        eprintln!("Thumbnail generation failed: {}", e);
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
                        return Err(sea_orm::DbErr::Custom(e));
                    }
                }
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
                            let tag_model = tag::ActiveModel::new(self.source_filename.clone(), tag_str.clone());
                            let tag_model_check = tag::Model {
                                id: 0, // id is not used in is_duplicate
                                source_filename: self.source_filename.clone(),
                                tag: tag_str.clone(),
                                slug: tag::Model::normalize_tag(tag_str),
                            };
                            if tag_model_check.is_duplicate(db).await.unwrap_or(false) {
                                continue; // Skip duplicates
                            }
                            if let Err(e) = tag_model.insert(db).await {
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