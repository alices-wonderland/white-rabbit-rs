use crate::account::account_tags;
use crate::journal;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, DeriveEntityModel)]
#[sea_orm(table_name = "account")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: Uuid,
  #[sea_orm(indexed)]
  pub name: String,
  pub description: String,
  #[sea_orm(indexed)]
  pub journal_id: Uuid,
  #[sea_orm(indexed)]
  pub parent_id: Option<Uuid>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "journal::Entity",
    from = "Column::JournalId",
    to = "crate::journal::Column::Id"
  )]
  Journal,
  #[sea_orm(belongs_to = "Entity", from = "Column::ParentId", to = "Column::Id")]
  Parent,
  #[sea_orm(has_many = "account_tags::Entity")]
  Tags,
}

pub struct ParentLink;

impl Linked for ParentLink {
  type FromEntity = Entity;

  type ToEntity = Entity;

  fn link(&self) -> Vec<RelationDef> {
    vec![Relation::Parent.def()]
  }
}

pub struct ChildrenLink;

impl Linked for ChildrenLink {
  type FromEntity = Entity;

  type ToEntity = Entity;

  fn link(&self) -> Vec<RelationDef> {
    vec![Relation::Parent.def().rev()]
  }
}

impl Related<journal::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Journal.def()
  }
}

impl Related<account_tags::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Tags.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}
