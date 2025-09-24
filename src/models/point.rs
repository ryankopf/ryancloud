use std::fs;
use std::path::PathBuf;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, DeriveEntityModel)]
#[sea_orm(table_name = "points")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[serde(skip)]
    // pub working_directory: String,
    pub source_filename: String,
    // pub clip_filename: String,
    pub time: i64,
    pub name: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    
}
