use crate::user::{Column, Role, User};

use sea_orm::sea_query::SelectStatement;
use sea_orm::ColumnTrait;
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug, Default, Eq, PartialEq)]
pub struct Query {
  pub id: HashSet<Uuid>,
  pub name: (String, bool),
  pub role: Option<Role>,
}

#[async_trait::async_trait]
impl crate::Query for Query {
  type AggregateRoot = User;

  async fn parse(self, stmt: &mut SelectStatement) -> crate::Result<()> {
    let Query { id, name: (name, name_fulltext), role } = self;

    let name = name.trim();

    stmt
      .and_where_option(if id.is_empty() { None } else { Some(Column::Id.is_in(id)) })
      .and_where_option(if name.is_empty() {
        None
      } else if name_fulltext {
        Some(Column::Name.like(&format!("%{}%", name)))
      } else {
        Some(Column::Name.eq(name))
      })
      .and_where_option(role.map(|role| Column::Role.eq(role)));

    Ok(())
  }
}
