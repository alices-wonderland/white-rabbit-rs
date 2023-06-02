use crate::journal::{journal_user, Column, Entity};
use crate::Query as _;
use sea_orm::sea_query::SimpleExpr;
use sea_orm::{
  ColumnTrait, Condition, EntityTrait, JoinType, QueryFilter, QuerySelect, RelationTrait, Select,
};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct Query {
  pub id: HashSet<Uuid>,
  pub name: (String, bool),
  pub description: String,
  pub admin: HashSet<Uuid>,
  pub member: HashSet<Uuid>,
}

impl From<Query> for Select<Entity> {
  fn from(val: Query) -> Self {
    let Query { id, name, description, admin, member } = val;

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

    if !admin.is_empty() || !member.is_empty() {
      select = select.join_rev(JoinType::InnerJoin, journal_user::Relation::Journal.def()).filter(
        Condition::any().add_option(user_query(admin, true)).add_option(user_query(member, false)),
      );
    }

    select
  }
}

impl crate::Query for Query {
  type Column = Column;
  type Entity = Entity;
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
