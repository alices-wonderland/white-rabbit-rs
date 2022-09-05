use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, DeriveEntityModel)]
#[sea_orm(table_name = "record_items")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub record_id: uuid::Uuid,
  #[sea_orm(primary_key)]
  pub account_id: uuid::Uuid,
  pub amount: Option<Decimal>,
  pub price: Option<Decimal>,
}

impl Related<super::Record> for Entity {
  fn to() -> RelationDef {
    Relation::Record.def()
  }
}

impl Related<super::Account> for Entity {
  fn to() -> RelationDef {
    Relation::Account.def()
  }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "super::Record",
    from = "Column::RecordId",
    to = "super::record::Column::Id",
    on_update = "Cascade",
    on_delete = "Cascade"
  )]
  Record,

  #[sea_orm(
    belongs_to = "super::Account",
    from = "Column::AccountId",
    to = "super::account::Column::Id",
    on_update = "Cascade",
    on_delete = "Cascade"
  )]
  Account,
}

impl ActiveModelBehavior for ActiveModel {}
