use crate::journal;
use crate::record::{record_item, record_tag};
use chrono::NaiveDate;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Hash, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "record")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: Uuid,
  #[sea_orm(indexed)]
  pub journal_id: Uuid,
  #[sea_orm(indexed)]
  pub name: String,
  pub description: String,
  #[sea_orm(column_name = "type")]
  pub typ: Type,
  #[sea_orm(indexed)]
  pub date: NaiveDate,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(has_many = "record_tag::Entity")]
  Tag,
  #[sea_orm(
    belongs_to = "journal::Entity",
    from = "Column::JournalId",
    to = "crate::journal::Column::Id"
  )]
  Journal,
  #[sea_orm(has_many = "record_item::Entity")]
  Item,
}

impl Related<journal::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Journal.def()
  }
}

impl Related<record_tag::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Tag.def()
  }
}

impl Related<record_item::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Item.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(
  Debug,
  Clone,
  Copy,
  Hash,
  Eq,
  PartialEq,
  Ord,
  PartialOrd,
  Serialize,
  Deserialize,
  EnumIter,
  DeriveActiveEnum,
)]
#[sea_orm(rs_type = "String", db_type = "String(Some(1))")]
pub enum Type {
  #[sea_orm(string_value = "R")]
  Record,
  #[sea_orm(string_value = "C")]
  Check,
}
