use sea_orm::entity::prelude::*;

use serde::{Deserialize, Serialize};

pub const TYPE: &str = "account";
pub const MULTIPLE: &str = "accounts";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, DeriveEntityModel)]
#[sea_orm(table_name = "accounts")]
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
  pub strategy: Strategy,
  pub unit: String,
  #[sea_orm(default_value = false)]
  pub is_archived: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "i8", db_type = "TinyInteger")]
pub enum Type {
  Income = 0,
  Expense = 1,
  Asset = 2,
  Liability = 3,
  Equity = 4,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "i8", db_type = "TinyInteger")]
pub enum Strategy {
  Fifo = 0,
  Average = 1,
}

impl Related<super::Journal> for Entity {
  fn to() -> RelationDef {
    Relation::Journal.def()
  }
}

impl Related<super::AccountTag> for Entity {
  fn to() -> RelationDef {
    Relation::Tag.def()
  }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(has_many = "super::AccountTag")]
  Tag,
  #[sea_orm(
    belongs_to = "super::Journal",
    from = "Column::JournalId",
    to = "super::journal::Column::Id"
  )]
  Journal,
}

impl ActiveModelBehavior for ActiveModel {}
