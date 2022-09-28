use sea_orm::entity::prelude::*;
use sea_orm::sea_query::{Expr, IntoCondition};
use serde::{Deserialize, Serialize};

use super::AccessItemType;

pub const TYPE: &str = "journal";
pub const MULTIPLE: &str = "journals";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, DeriveEntityModel)]
#[sea_orm(table_name = "journals")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: uuid::Uuid,
  #[sea_orm(unique, indexed)]
  pub name: String,
  pub description: String,
  pub unit: String,
  #[sea_orm(default_value = false)]
  pub is_archived: bool,
}

impl Related<super::JournalTag> for Entity {
  fn to() -> RelationDef {
    Relation::Tag.def()
  }
}

impl Related<super::Account> for Entity {
  fn to() -> RelationDef {
    Relation::Account.def()
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AccessItem {
  pub id: uuid::Uuid,
  #[serde(rename = "type")]
  pub typ: AccessItemType,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(has_many = "super::JournalTag")]
  Tag,
  #[sea_orm(has_many = "super::Account")]
  Account,
}

impl ActiveModelBehavior for ActiveModel {}

pub struct JournalUserAdmin;

impl Linked for JournalUserAdmin {
  type FromEntity = Entity;

  type ToEntity = super::User;

  fn link(&self) -> Vec<RelationDef> {
    vec![
      super::journal_user::Relation::Journal
        .def()
        .on_condition(|_, right| {
          Expr::tbl(right, super::journal_user::Column::IsAdmin)
            .eq(true)
            .into_condition()
        })
        .rev(),
      super::journal_user::Relation::User.def(),
    ]
  }
}

pub struct JournalUserMember;

impl Linked for JournalUserMember {
  type FromEntity = Entity;

  type ToEntity = super::User;

  fn link(&self) -> Vec<RelationDef> {
    vec![
      super::journal_user::Relation::Journal
        .def()
        .on_condition(|_, right| {
          Expr::tbl(right, super::journal_user::Column::IsAdmin)
            .eq(false)
            .into_condition()
        })
        .rev(),
      super::journal_user::Relation::User.def(),
    ]
  }
}

pub struct JournalGroupAdmin;

impl Linked for JournalGroupAdmin {
  type FromEntity = Entity;

  type ToEntity = super::Group;

  fn link(&self) -> Vec<RelationDef> {
    vec![
      super::journal_group::Relation::Journal
        .def()
        .on_condition(|_, right| {
          Expr::tbl(right, super::journal_group::Column::IsAdmin)
            .eq(true)
            .into_condition()
        })
        .rev(),
      super::journal_group::Relation::Group.def(),
    ]
  }
}

pub struct JournalGroupMember;

impl Linked for JournalGroupMember {
  type FromEntity = Entity;

  type ToEntity = super::Group;

  fn link(&self) -> Vec<RelationDef> {
    vec![
      super::journal_group::Relation::Journal
        .def()
        .on_condition(|_, right| {
          Expr::tbl(right, super::journal_group::Column::IsAdmin)
            .eq(false)
            .into_condition()
        })
        .rev(),
      super::journal_group::Relation::Group.def(),
    ]
  }
}
