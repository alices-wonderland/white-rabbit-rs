use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, DeriveEntityModel)]
#[sea_orm(table_name = "groups_users")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub group_id: uuid::Uuid,
  #[sea_orm(primary_key)]
  pub user_id: uuid::Uuid,
  #[sea_orm(primary_key)]
  pub is_admin: bool,
}

impl Related<super::Group> for Entity {
  fn to() -> RelationDef {
    Relation::Group.def()
  }
}

impl Related<super::User> for Entity {
  fn to() -> RelationDef {
    Relation::User.def()
  }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "super::User",
    from = "Column::UserId",
    to = "super::user::Column::Id",
    on_update = "Cascade",
    on_delete = "Cascade"
  )]
  User,
  #[sea_orm(
    belongs_to = "super::Group",
    from = "Column::GroupId",
    to = "super::group::Column::Id",
    on_update = "Cascade",
    on_delete = "Cascade"
  )]
  Group,
}

impl ActiveModelBehavior for ActiveModel {}
