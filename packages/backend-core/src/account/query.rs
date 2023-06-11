use crate::account::{account_tag, Column, Entity};
use crate::Query as _;
use sea_orm::{
  ColumnTrait, EntityTrait, JoinType, QueryFilter, QuerySelect, RelationTrait, Select,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug, Default, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Query {
  #[serde(default)]
  pub id: HashSet<Uuid>,
  pub name: (String, bool),
  #[serde(default)]
  pub description: String,
  #[serde(default)]
  pub tag: String,
  #[serde(default)]
  pub journal: HashSet<Uuid>,
}

impl From<Query> for Select<Entity> {
  fn from(val: Query) -> Self {
    let Query { id, name, description, tag, journal } = val;

    let mut select = Entity::find();

    if let Some(expr) = Query::id_expr(Column::Id, id) {
      select = select.filter(expr);
    }

    if let Some(expr) = Query::text_expr(Column::Name, name) {
      select = select.filter(expr);
    }

    if let Some(expr) = Query::text_expr(Column::Description, (description, true)) {
      select = select.filter(expr);
    }

    let tag = tag.trim();
    if !tag.is_empty() {
      select = select
        .join_rev(JoinType::InnerJoin, account_tag::Relation::Account.def())
        .filter(account_tag::Column::Tag.contains(tag));
    }

    if !journal.is_empty() {
      select = select.filter(Column::JournalId.is_in(journal));
    }

    select
  }
}

impl crate::Query for Query {
  type Entity = Entity;
  type Column = Column;
}
