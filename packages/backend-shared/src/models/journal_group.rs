use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use super::{journal, AccessItemType};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, DeriveEntityModel)]
#[sea_orm(table_name = "journals_groups")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub journal_id: uuid::Uuid,
  #[sea_orm(primary_key)]
  pub group_id: uuid::Uuid,
  #[sea_orm(primary_key)]
  pub is_admin: bool,
}

impl From<Model> for journal::AccessItem {
  fn from(val: Model) -> Self {
    Self {
      id: val.group_id,
      typ: AccessItemType::Group,
    }
  }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "super::Group",
    from = "Column::GroupId",
    to = "super::group::Column::Id",
    on_update = "Cascade",
    on_delete = "Cascade"
  )]
  Group,
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
