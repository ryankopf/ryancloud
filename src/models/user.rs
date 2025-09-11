use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub username: String,
    pub password_hash: String,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {}

impl sea_orm::RelationTrait for Relation {
    fn def(&self) -> sea_orm::RelationDef {
        match self {
            _ => unreachable!("No relations defined for user")
        }
    }
}

impl ActiveModelBehavior for ActiveModel {}
