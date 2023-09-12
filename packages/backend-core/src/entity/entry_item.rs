use crate::entity::{account, entry};
use rust_decimal::Decimal;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, Hash, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "entry_item")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub entry_id: Uuid,
  #[sea_orm(primary_key)]
  pub account_id: Uuid,
  pub amount: Decimal,
  pub price: Option<Decimal>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "entry::Entity",
    from = "Column::EntryId",
    to = "entry::Column::Id",
    on_update = "Cascade",
    on_delete = "Cascade"
  )]
  Entry,

  #[sea_orm(
    belongs_to = "account::Entity",
    from = "Column::AccountId",
    to = "account::Column::Id",
    on_update = "Cascade",
    on_delete = "Cascade"
  )]
  Account,
}

impl ActiveModelBehavior for ActiveModel {}

impl Related<entry::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Entry.def()
  }
}

impl Related<account::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Account.def()
  }
}
