use crate::{journal, user};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Hash, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "journal_user")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub journal_id: Uuid,
  #[sea_orm(primary_key)]
  pub user_id: Uuid,
  pub field: Field,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "journal::Entity",
    from = "Column::JournalId",
    to = "journal::Column::Id",
    on_update = "Cascade",
    on_delete = "Cascade"
  )]
  Journal,
  #[sea_orm(
    belongs_to = "user::Entity",
    from = "Column::UserId",
    to = "user::Column::Id",
    on_update = "Cascade",
    on_delete = "Cascade"
  )]
  User,
}

impl Related<user::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::User.def()
  }
}

impl Related<journal::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Journal.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(Some(1))")]
pub enum Field {
  #[sea_orm(string_value = "A")]
  Admin,
  #[sea_orm(string_value = "M")]
  Member,
}
