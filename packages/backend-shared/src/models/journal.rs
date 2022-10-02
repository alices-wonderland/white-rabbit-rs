use std::collections::HashSet;

use sea_orm::entity::prelude::*;
use sea_orm::sea_query::{Expr, IntoCondition};
use sea_orm::ConnectionTrait;
use serde::{Deserialize, Serialize};

use super::{
  journal_group, journal_user, AccessItem, Account, Group, IntoPresentation, JournalGroup, JournalTag, JournalUser,
  Record, User,
};

pub const TYPE: &str = "journal";
pub const MULTIPLE: &str = "journals";

#[derive(Clone, Debug, Eq, PartialEq, DeriveEntityModel)]
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

impl Related<JournalTag> for Entity {
  fn to() -> RelationDef {
    Relation::Tag.def()
  }
}

impl Related<Account> for Entity {
  fn to() -> RelationDef {
    Relation::Account.def()
  }
}

impl Related<Record> for Entity {
  fn to() -> RelationDef {
    Relation::Record.def()
  }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(has_many = "JournalTag")]
  Tag,
  #[sea_orm(has_many = "Account")]
  Account,
  #[sea_orm(has_many = "Record")]
  Record,
}

impl ActiveModelBehavior for ActiveModel {}

pub struct JournalUserAdmin;

impl Linked for JournalUserAdmin {
  type FromEntity = Entity;

  type ToEntity = User;

  fn link(&self) -> Vec<RelationDef> {
    vec![
      journal_user::Relation::Journal
        .def()
        .on_condition(|_, right| {
          Expr::tbl(right, journal_user::Column::IsAdmin)
            .eq(true)
            .into_condition()
        })
        .rev(),
      journal_user::Relation::User.def(),
    ]
  }
}

pub struct JournalUserMember;

impl Linked for JournalUserMember {
  type FromEntity = Entity;

  type ToEntity = User;

  fn link(&self) -> Vec<RelationDef> {
    vec![
      journal_user::Relation::Journal
        .def()
        .on_condition(|_, right| {
          Expr::tbl(right, journal_user::Column::IsAdmin)
            .eq(false)
            .into_condition()
        })
        .rev(),
      journal_user::Relation::User.def(),
    ]
  }
}

pub struct JournalGroupAdmin;

impl Linked for JournalGroupAdmin {
  type FromEntity = Entity;

  type ToEntity = Group;

  fn link(&self) -> Vec<RelationDef> {
    vec![
      journal_group::Relation::Journal
        .def()
        .on_condition(|_, right| {
          Expr::tbl(right, journal_group::Column::IsAdmin)
            .eq(true)
            .into_condition()
        })
        .rev(),
      journal_group::Relation::Group.def(),
    ]
  }
}

pub struct JournalGroupMember;

impl Linked for JournalGroupMember {
  type FromEntity = Entity;

  type ToEntity = Group;

  fn link(&self) -> Vec<RelationDef> {
    vec![
      journal_group::Relation::Journal
        .def()
        .on_condition(|_, right| {
          Expr::tbl(right, journal_group::Column::IsAdmin)
            .eq(false)
            .into_condition()
        })
        .rev(),
      journal_group::Relation::Group.def(),
    ]
  }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Presentation {
  pub id: uuid::Uuid,
  pub name: String,
  pub description: String,
  pub unit: String,
  #[serde(rename = "isArchived")]
  pub is_archived: bool,
  pub tags: HashSet<String>,
  pub admins: HashSet<AccessItem>,
  pub members: HashSet<AccessItem>,
}

#[async_trait::async_trait]
impl IntoPresentation for Model {
  type Presentation = Presentation;

  async fn into_presentation(self, conn: &impl ConnectionTrait) -> anyhow::Result<Self::Presentation> {
    let tags = self
      .find_related(JournalTag)
      .all(conn)
      .await?
      .into_iter()
      .map(|item| item.tag)
      .collect();

    let (admin_users, member_users): (Vec<_>, Vec<_>) = JournalUser::find()
      .filter(journal_user::Column::JournalId.eq(self.id))
      .all(conn)
      .await?
      .into_iter()
      .partition(|item| item.is_admin);
    let admin_users: HashSet<AccessItem> = admin_users.into_iter().map(|item| item.into()).collect();
    let member_users: HashSet<AccessItem> = member_users.into_iter().map(|item| item.into()).collect();

    let (admin_groups, member_groups): (Vec<_>, Vec<_>) = JournalGroup::find()
      .filter(journal_group::Column::JournalId.eq(self.id))
      .all(conn)
      .await?
      .into_iter()
      .partition(|item| item.is_admin);
    let admin_groups: HashSet<AccessItem> = admin_groups.into_iter().map(|item| item.into()).collect();
    let member_groups: HashSet<AccessItem> = member_groups.into_iter().map(|item| item.into()).collect();

    let Model {
      id,
      name,
      description,
      unit,
      is_archived,
    } = self;
    Ok(Presentation {
      id,
      name,
      description,
      unit,
      is_archived,
      tags,
      admins: vec![admin_users, admin_groups].into_iter().flatten().collect(),
      members: vec![member_users, member_groups].into_iter().flatten().collect(),
    })
  }
}

impl From<Presentation> for Model {
  fn from(
    Presentation {
      id,
      name,
      description,
      unit,
      is_archived,
      ..
    }: Presentation,
  ) -> Self {
    Self {
      id,
      name,
      description,
      unit,
      is_archived,
    }
  }
}
