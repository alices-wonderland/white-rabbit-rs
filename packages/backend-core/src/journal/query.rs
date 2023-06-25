use crate::journal::{journal_user, Column, Entity};
use crate::Query as _;
use sea_orm::sea_query::SimpleExpr;
use sea_orm::{
  ColumnTrait, Condition, EntityTrait, JoinType, QueryFilter, QuerySelect, RelationTrait, Select,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug, Default, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Query {
  #[serde(default)]
  pub id: HashSet<Uuid>,
  #[serde(default)]
  pub name: (String, bool),
  #[serde(default)]
  pub description: String,
  #[serde(default)]
  pub unit: String,
  #[serde(default)]
  pub admin: HashSet<Uuid>,
  #[serde(default)]
  pub member: HashSet<Uuid>,
}

impl From<Query> for Select<Entity> {
  fn from(val: Query) -> Self {
    let Query { id, name, description, unit, admin, member } = val;

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

    let unit = unit.trim();
    if !unit.is_empty() {
      select = select.filter(Column::Unit.eq(unit));
    }

    if !admin.is_empty() || !member.is_empty() {
      select = select.join_rev(JoinType::InnerJoin, journal_user::Relation::Journal.def()).filter(
        Condition::any().add_option(user_query(admin, true)).add_option(user_query(member, false)),
      );
    }

    select
  }
}

impl crate::Query for Query {
  type Entity = Entity;
  type Column = Column;
}

fn user_query(ids: HashSet<Uuid>, is_admin: bool) -> Option<SimpleExpr> {
  if ids.is_empty() {
    None
  } else {
    Some(
      journal_user::Column::Field
        .eq(if is_admin { journal_user::Field::Admin } else { journal_user::Field::Member })
        .and(journal_user::Column::UserId.is_in(ids)),
    )
  }
}

impl From<HashSet<Uuid>> for Query {
  fn from(value: HashSet<Uuid>) -> Self {
    Query { id: value, ..Default::default() }
  }
}
