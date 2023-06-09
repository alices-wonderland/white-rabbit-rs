use crate::user::{Column, Entity, Role};
use crate::Query as _;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QuerySelect, Select};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug, Default, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Query {
  #[serde(default)]
  pub id: HashSet<Uuid>,
  #[serde(default)]
  pub name: (String, bool),
  pub role: Option<Role>,
}

impl From<Query> for Select<Entity> {
  fn from(val: Query) -> Self {
    let Query { id, name, role } = val;
    let mut select = Entity::find().distinct();
    if let Some(expr) = Query::id_expr(Column::Id, id) {
      select = select.filter(expr);
    }
    if let Some(expr) = Query::text_expr(Column::Name, name) {
      select = select.filter(expr);
    }
    if let Some(role) = role {
      select = select.filter(Column::Role.eq(role));
    }

    select
  }
}

impl crate::Query for Query {
  type Entity = Entity;
  type Column = Column;
}
