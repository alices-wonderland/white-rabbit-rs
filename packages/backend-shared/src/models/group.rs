use std::collections::HashSet;

use sea_orm::entity::prelude::*;
use sea_orm::sea_query::{Expr, IntoCondition};
use sea_orm::ConnectionTrait;
use serde::{Deserialize, Serialize};

use super::{group_user, GroupUser, IntoPresentation, User};

pub const TYPE: &str = "group";
pub const MULTIPLE: &str = "groups";

#[derive(Clone, Debug, Eq, PartialEq, Hash, DeriveEntityModel)]
#[sea_orm(table_name = "groups")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: uuid::Uuid,
  #[sea_orm(unique, indexed)]
  pub name: String,
  pub description: String,
}

impl Related<User> for Entity {
  fn to() -> RelationDef {
    Relation::User.def()
  }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(has_many = "GroupUser")]
  User,
}

impl ActiveModelBehavior for ActiveModel {}

pub struct GroupAdmin;

impl Linked for GroupAdmin {
  type FromEntity = Entity;

  type ToEntity = User;

  fn link(&self) -> Vec<RelationDef> {
    vec![
      group_user::Relation::Group
        .def()
        .on_condition(|_, right| Expr::tbl(right, group_user::Column::IsAdmin).eq(true).into_condition())
        .rev(),
      group_user::Relation::User.def(),
    ]
  }
}

pub struct GroupMember;

impl Linked for GroupMember {
  type FromEntity = Entity;

  type ToEntity = User;

  fn link(&self) -> Vec<RelationDef> {
    vec![
      group_user::Relation::Group
        .def()
        .on_condition(|_, right| Expr::tbl(right, group_user::Column::IsAdmin).eq(false).into_condition())
        .rev(),
      group_user::Relation::User.def(),
    ]
  }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Presentation {
  pub id: uuid::Uuid,
  pub name: String,
  pub description: String,
  pub admins: HashSet<uuid::Uuid>,
  pub members: HashSet<uuid::Uuid>,
}

#[async_trait::async_trait]
impl IntoPresentation for Model {
  type Presentation = Presentation;

  async fn into_presentation(self, conn: &impl ConnectionTrait) -> anyhow::Result<Self::Presentation> {
    let (admins, members): (Vec<_>, Vec<_>) = GroupUser::find()
      .filter(group_user::Column::GroupId.eq(self.id))
      .all(conn)
      .await?
      .into_iter()
      .partition(|item| item.is_admin);
    let Model { id, name, description } = self;
    Ok(Presentation {
      id,
      name,
      description,
      admins: admins.into_iter().map(|user| user.user_id).collect(),
      members: members.into_iter().map(|user| user.user_id).collect(),
    })
  }
}

impl From<Presentation> for Model {
  fn from(
    Presentation {
      id, name, description, ..
    }: Presentation,
  ) -> Self {
    Self { id, name, description }
  }
}
