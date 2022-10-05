use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use super::{account, record, Account, Record};
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "record_items")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub record_id: uuid::Uuid,
  #[sea_orm(primary_key)]
  pub account_id: uuid::Uuid,
  pub amount: Decimal,
  pub price: Option<Decimal>,
}

impl Related<Record> for Entity {
  fn to() -> RelationDef {
    Relation::Record.def()
  }
}

impl Related<Account> for Entity {
  fn to() -> RelationDef {
    Relation::Account.def()
  }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "Record",
    from = "Column::RecordId",
    to = "record::Column::Id",
    on_update = "Cascade",
    on_delete = "Cascade"
  )]
  Record,

  #[sea_orm(
    belongs_to = "Account",
    from = "Column::AccountId",
    to = "account::Column::Id",
    on_update = "Cascade",
    on_delete = "Cascade"
  )]
  Account,
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Presentation {
  #[serde(rename = "accountId")]
  pub account_id: uuid::Uuid,
  pub amount: Decimal,
  pub price: Option<Decimal>,
}

impl From<Model> for Presentation {
  fn from(
    Model {
      account_id,
      amount,
      price,
      ..
    }: Model,
  ) -> Self {
    Self {
      account_id,
      amount,
      price,
    }
  }
}
