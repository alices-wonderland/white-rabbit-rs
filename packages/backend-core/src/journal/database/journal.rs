use crate::journal::journal_user;
use crate::{account, user};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, Hash, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "journal")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: Uuid,
  #[sea_orm(unique, indexed)]
  pub name: String,
  pub description: String,
  #[sea_orm(indexed)]
  pub unit: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(has_many = "journal_user::Entity")]
  JournalUsers,
  #[sea_orm(has_many = "account::Entity")]
  Accounts,
}

impl Related<journal_user::Entity> for Entity {
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
    journal_user::Relation::User.def()
  }

  fn via() -> Option<RelationDef> {
    Some(journal_user::Relation::Journal.def().rev())
  }
}

impl ActiveModelBehavior for ActiveModel {}
