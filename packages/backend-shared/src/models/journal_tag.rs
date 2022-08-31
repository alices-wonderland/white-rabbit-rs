use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, DeriveEntityModel)]
#[sea_orm(table_name = "journal_tags")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub journal_id: i32,
  #[sea_orm(primary_key)]
  pub tag: String,
}

impl Related<super::Journal> for Entity {
  fn to() -> RelationDef {
    Relation::Journal.def()
  }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
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
