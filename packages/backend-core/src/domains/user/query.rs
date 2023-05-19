use crate::domains::user::model::Column;
use crate::Role;

use sea_orm::sea_query::IntoCondition;
use sea_orm::{ColumnTrait, Condition};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug, Default)]
pub struct UserQuery {
  pub id: HashSet<Uuid>,
  pub name: (String, bool),
  pub role: Option<Role>,
}

impl IntoCondition for UserQuery {
  fn into_condition(self) -> Condition {
    let name = self.name.0.trim();

    Condition::all()
      .add_option(if self.id.is_empty() { None } else { Some(Column::Id.is_in(self.id)) })
      .add_option(if name.is_empty() {
        None
      } else {
        Some(if self.name.1 { Column::Name.like(&format!("%{}%", name)) } else { Column::Name.eq(name) })
      })
      .add_option(self.role.map(|role| Column::Role.eq(role)))
  }
}
