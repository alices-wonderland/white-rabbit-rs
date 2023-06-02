use crate::account;
use sea_orm::entity::prelude::*;

use uuid::Uuid;

#[derive(Clone, Debug, Hash, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "account_tag")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub account_id: Uuid,
  #[sea_orm(primary_key)]
  pub tag: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "account::Entity",
    from = "Column::AccountId",
    to = "account::Column::Id",
    on_update = "Cascade",
    on_delete = "Cascade"
  )]
  Account,
}

impl Related<account::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Account.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}
