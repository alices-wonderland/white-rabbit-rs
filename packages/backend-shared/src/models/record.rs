use chrono::NaiveDate;
use sea_orm::entity::prelude::*;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, DeriveEntityModel)]
#[sea_orm(table_name = "records")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: uuid::Uuid,
  #[sea_orm(indexed)]
  pub journal_id: uuid::Uuid,
  #[sea_orm(unique, indexed)]
  pub name: String,
  pub description: String,
  #[sea_orm(column_name = "type")]
  pub typ: Type,
  pub date: NaiveDate,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "i8", db_type = "TinyInteger")]
pub enum Type {
  Record = 0,
  Check = 1,
}

impl Related<super::Journal> for Entity {
  fn to() -> RelationDef {
    Relation::Journal.def()
  }
}

impl Related<super::RecordTag> for Entity {
  fn to() -> RelationDef {
    Relation::Tag.def()
  }
}

impl Related<super::RecordItem> for Entity {
  fn to() -> RelationDef {
    Relation::Item.def()
  }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(has_many = "super::RecordTag")]
  Tag,
  #[sea_orm(
    belongs_to = "super::Journal",
    from = "Column::JournalId",
    to = "super::journal::Column::Id"
  )]
  Journal,
  #[sea_orm(has_many = "super::RecordItem")]
  Item,
}

impl ActiveModelBehavior for ActiveModel {}
