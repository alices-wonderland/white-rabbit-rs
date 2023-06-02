use crate::record::{Column, Entity};
use crate::Query as _;
use sea_orm::{EntityTrait, QueryFilter, Select};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct Query {
  pub id: HashSet<Uuid>,
}

impl From<Query> for Select<Entity> {
  fn from(value: Query) -> Self {
    let Query { id } = value;

    let mut select = Entity::find();

    if let Some(expr) = Query::id_expr(Column::Id, id) {
      select = select.filter(expr);
    }

    select
  }
}

impl crate::Query for Query {
  type Entity = Entity;
  type Column = Column;
}
