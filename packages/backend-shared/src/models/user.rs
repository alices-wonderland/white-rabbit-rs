use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i32,
  #[sea_orm(unique, indexed)]
  pub name: String,
  #[sea_orm(indexed)]
  pub role: Role,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "i8", db_type = "TinyInteger")]
pub enum Role {
  User = 0,
  Admin = 1,
  Owner = 2,
}

impl Related<super::AuthId> for Entity {
  fn to() -> RelationDef {
    Relation::AuthId.def()
  }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(has_many = "super::AuthId")]
  AuthId,
}

impl ActiveModelBehavior for ActiveModel {}
