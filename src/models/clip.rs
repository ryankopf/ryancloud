use std::fs;
use std::path::PathBuf;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, DeriveEntityModel)]
#[sea_orm(table_name = "clips")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[serde(skip)]
    pub working_directory: String,
    pub source_filename: String,
    pub start: i64,
    pub end: i64,
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn export(&self) -> std::io::Result<()> {
        let mut path = PathBuf::from(&self.working_directory);
        path.push(".clips");
        fs::create_dir_all(&path)?; // ensure .clips directory exists
        path.push(format!("{}.json", self.id));
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        fs::write(path, json)?;
        Ok(())
    }
}
