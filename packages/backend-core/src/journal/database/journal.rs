use crate::journal::journal_users;
use crate::{account, user};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, DeriveEntityModel)]
#[sea_orm(table_name = "journal")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: Uuid,
  #[sea_orm(unique, indexed)]
  pub name: String,
  pub description: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(has_many = "journal_users::Entity")]
  JournalUsers,
  #[sea_orm(has_many = "account::Entity")]
  Accounts,
}

impl Related<journal_users::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::JournalUsers.def()
  }
}

impl Related<account::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Accounts.def()
  }
}

impl Related<user::Entity> for Entity {
  fn to() -> RelationDef {
    journal_users::Relation::User.def()
  }

  fn via() -> Option<RelationDef> {
    Some(journal_users::Relation::Journal.def().rev())
  }
}

impl ActiveModelBehavior for ActiveModel {}
