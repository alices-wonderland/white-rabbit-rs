use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use super::{journal, AccessItemType};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, DeriveEntityModel)]
#[sea_orm(table_name = "journals_users")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub journal_id: uuid::Uuid,
  #[sea_orm(primary_key)]
  pub user_id: uuid::Uuid,
  #[sea_orm(primary_key)]
  pub is_admin: bool,
}

impl From<Model> for journal::AccessItem {
  fn from(val: Model) -> Self {
    Self {
      id: val.user_id,
      typ: AccessItemType::User,
    }
  }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "super::User",
    from = "Column::UserId",
    to = "super::user::Column::Id",
    on_update = "Cascade",
    on_delete = "Cascade"
  )]
  User,
  #[sea_orm(
    belongs_to = "super::Journal",
    from = "Column::JournalId",
    to = "super::journal::Column::Id",
    on_update = "Cascade",
    on_delete = "Cascade"
  )]
  Journal,
}

impl ActiveModelBehavior for ActiveModel {}
