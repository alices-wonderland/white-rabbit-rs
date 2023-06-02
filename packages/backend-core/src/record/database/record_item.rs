use crate::{account, record};
use rust_decimal::Decimal;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, Hash, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "record_item")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub record_id: Uuid,
  #[sea_orm(primary_key)]
  pub account_id: Uuid,
  pub amount: Decimal,
  pub price: Option<Decimal>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "record::Entity",
    from = "Column::RecordId",
    to = "crate::record::Column::Id",
    on_update = "Cascade",
    on_delete = "Cascade"
  )]
  Record,

  #[sea_orm(
    belongs_to = "account::Entity",
    from = "Column::AccountId",
    to = "crate::account::Column::Id",
    on_update = "Cascade",
    on_delete = "Cascade"
  )]
  Account,
}

impl ActiveModelBehavior for ActiveModel {}

impl Related<record::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Record.def()
  }
}

impl Related<account::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Account.def()
  }
}
