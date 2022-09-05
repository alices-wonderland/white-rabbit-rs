use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, DeriveEntityModel)]
#[sea_orm(table_name = "auth_ids")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub user_id: uuid::Uuid,
  #[sea_orm(primary_key)]
  pub provider: String,
  pub value: String,
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
}

impl ActiveModelBehavior for ActiveModel {}
