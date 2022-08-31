use sea_orm::entity::prelude::*;
use sea_orm::sea_query::{Expr, IntoCondition};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, DeriveEntityModel)]
#[sea_orm(table_name = "groups")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i32,
  #[sea_orm(unique, indexed)]
  pub name: String,
  pub description: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

pub struct GroupAdmin;

impl Linked for GroupAdmin {
  type FromEntity = Entity;

  type ToEntity = super::User;

  fn link(&self) -> Vec<RelationDef> {
    vec![
      super::group_user::Relation::Group
        .def()
        .on_condition(|_, right| {
          Expr::tbl(right, super::group_user::Column::IsAdmin)
            .eq(true)
            .into_condition()
        })
        .rev(),
      super::group_user::Relation::User.def(),
    ]
  }
}

pub struct GroupMember;

impl Linked for GroupMember {
  type FromEntity = Entity;

  type ToEntity = super::User;

  fn link(&self) -> Vec<RelationDef> {
    vec![
      super::group_user::Relation::Group
        .def()
        .on_condition(|_, right| {
          Expr::tbl(right, super::group_user::Column::IsAdmin)
            .eq(false)
            .into_condition()
        })
        .rev(),
      super::group_user::Relation::User.def(),
    ]
  }
}
