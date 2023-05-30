use crate::account::{account_tags, Column, Entity};
use crate::Query as _;
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
  pub tag: String,
  pub journal: HashSet<Uuid>,
  pub parent: HashSet<Option<Uuid>>,
}

impl From<Query> for Select<Entity> {
  fn from(val: Query) -> Self {
    let Query { id, name, description, tag, journal, parent } = val;

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
        .join_rev(JoinType::InnerJoin, account_tags::Relation::Account.def())
        .filter(account_tags::Column::Tag.contains(tag));
    }

    if !journal.is_empty() {
      select = select.filter(Column::JournalId.is_in(journal));
    }

    if !parent.is_empty() {
      let contains_none = parent.contains(&None);
      let parent = parent.into_iter().flatten().collect::<HashSet<_>>();
      select = select.filter(
        Condition::any()
          .add_option(if contains_none { Some(Column::ParentId.is_null()) } else { None })
          .add_option(if parent.is_empty() { None } else { Some(Column::ParentId.is_in(parent)) }),
      );
    }

    select
  }
}

impl crate::Query for Query {
  type Column = Column;
  type Entity = Entity;
}
