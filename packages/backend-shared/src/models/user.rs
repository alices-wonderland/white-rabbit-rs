use std::collections::HashSet;

use sea_orm::{entity::prelude::*, ConnectionTrait};
use serde::{Deserialize, Serialize};

use super::{group_user, AuthId, Group, IntoPresentation};

pub const TYPE: &str = "user";
pub const MULTIPLE: &str = "users";

#[derive(Clone, Debug, Eq, PartialEq, Hash, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: uuid::Uuid,
  #[sea_orm(unique, indexed)]
  pub name: String,
  #[sea_orm(indexed)]
  pub role: Role,
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "i8", db_type = "TinyInteger")]
pub enum Role {
  User = 0,
  Admin = 1,
  Owner = 2,
}

impl Default for Role {
  fn default() -> Self {
    Self::User
  }
}

impl Related<AuthId> for Entity {
  fn to() -> RelationDef {
    Relation::AuthId.def()
  }
}

impl Related<Group> for Entity {
  fn to() -> RelationDef {
    group_user::Relation::Group.def()
  }

  fn via() -> Option<RelationDef> {
    Some(group_user::Relation::User.def().rev())
  }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(has_many = "AuthId")]
  AuthId,
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Presentation {
  pub id: uuid::Uuid,
  pub name: String,
  pub role: Role,
  #[serde(rename = "authIds")]
  pub auth_ids: HashSet<(String, String)>,
}

#[async_trait::async_trait]
impl IntoPresentation for Model {
  type Presentation = Presentation;

  async fn into_presentation(self, conn: &impl ConnectionTrait) -> anyhow::Result<Self::Presentation> {
    let auth_ids = self
      .find_related(AuthId)
      .all(conn)
      .await?
      .into_iter()
      .map(|item| (item.provider, item.value))
      .collect();
    let Model { id, name, role } = self;
    Ok(Presentation {
      id,
      name,
      role,
      auth_ids,
    })
  }
}

impl From<Presentation> for Model {
  fn from(Presentation { id, name, role, .. }: Presentation) -> Self {
    Self { id, name, role }
  }
}
