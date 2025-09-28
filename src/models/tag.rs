use sea_orm::entity::prelude::*;
use sea_orm::ColumnTrait;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, DeriveEntityModel)]
#[sea_orm(table_name = "tags")]
pub struct Model {
	#[sea_orm(primary_key)]
	pub id: i32,
	#[serde(skip)]
	pub source_filename: String,
	pub tag: String, // The tag word or phrase
	pub slug: String, // Normalized version for searching (downcased, dashes)
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
	/// Check if a duplicate tag exists in the database (same slug OR tag for the same source_filename)
	pub async fn is_duplicate(&self, db: &DatabaseConnection) -> Result<bool, sea_orm::DbErr> {
		let found = Entity::find()
			.filter(Column::SourceFilename.eq(&self.source_filename))
			.filter(
				Column::Slug.eq(&self.slug)
				.or(Column::Tag.eq(&self.tag))
			)
			.one(db)
			.await?;
		Ok(found.is_some())
	}
	/// Normalize a tag string: downcase, trim, replace spaces with dashes, remove non-alphanumeric except dashes
	pub fn normalize_tag(tag: &str) -> String {
		tag.trim()
			.to_lowercase()
			.replace(|c: char| c.is_whitespace(), "-")
			.replace(|c: char| !c.is_ascii_alphanumeric() && c != '-', "")
	}

	/// Create a new tag model (id is 0 for new, will be set by DB)
	pub fn new(source_filename: String, tag: String) -> Self {
		let slug = Self::normalize_tag(&tag);
		Self {
			id: 0,
			source_filename,
			tag,
			slug,
		}
	}
}
