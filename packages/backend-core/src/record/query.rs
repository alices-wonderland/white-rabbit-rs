use crate::record::{record_item, Column, Entity, Type};
use crate::Query as _;
use chrono::NaiveDate;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Select};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug, Default, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Query {
  #[serde(default)]
  pub id: HashSet<Uuid>,
  pub typ: Option<Type>,
  #[serde(default)]
  pub journal: HashSet<Uuid>,
  #[serde(default)]
  pub account: HashSet<Uuid>,
  pub start: Option<NaiveDate>,
  pub end: Option<NaiveDate>,
}

impl From<Query> for Select<Entity> {
  fn from(value: Query) -> Self {
    let Query { id, typ, journal, account, start, end } = value;

    let mut select = Entity::find();

    if let Some(expr) = Query::id_expr(Column::Id, id) {
      select = select.filter(expr);
    }

    if let Some(typ) = typ {
      select = select.filter(Column::Typ.eq(typ));
    }

    if !journal.is_empty() {
      select = select.filter(Column::JournalId.is_in(journal));
    }

    if !account.is_empty() {
      select =
        select.left_join(record_item::Entity).filter(record_item::Column::AccountId.is_in(account));
    }

    if let Some(start) = start {
      select = select.filter(Column::Date.gte(start));
    }

    if let Some(end) = end {
      select = select.filter(Column::Date.lte(end));
    }

    select
  }
}

impl crate::Query for Query {
  type Entity = Entity;
  type Column = Column;
}
