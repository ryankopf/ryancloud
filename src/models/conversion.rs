use std::str::FromStr;
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
                // TODO: Implement categorize logic
                // First, FFMPEG to extract a representative frame, at 1s.
                // Then put the image in /ai/conversions/ID.jpg
                // Then call the AI tagging function on it.
                
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