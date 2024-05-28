use crate::entity::account_tag;
use crate::entity::journal;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Hash, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "accounts")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: Uuid,
  #[sea_orm(indexed)]
  pub journal_id: Uuid,
  #[sea_orm(indexed)]
  pub name: String,
  pub description: String,
  #[sea_orm(indexed)]
  pub unit: String,
  #[sea_orm(indexed, column_name = "type")]
  pub typ: Type,
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
  #[sea_orm(has_many = "account_tag::Entity")]
  Tags,
}

impl Related<journal::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Journal.def()
  }
}

impl Related<account_tag::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Tags.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(
  Debug,
  Copy,
  Clone,
  Hash,
  Eq,
  PartialEq,
  Ord,
  PartialOrd,
  strum_macros::Display,
  Serialize,
  Deserialize,
  EnumIter,
  DeriveActiveEnum,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(1))")]
pub enum Type {
  #[sea_orm(string_value = "I")]
  Income,
  #[sea_orm(string_value = "E")]
  Expense,
  #[sea_orm(string_value = "A")]
  Asset,
  #[sea_orm(string_value = "L")]
  Liability,
  #[sea_orm(string_value = "Q")]
  Equity,
}
