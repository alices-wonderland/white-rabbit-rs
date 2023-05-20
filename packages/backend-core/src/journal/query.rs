use crate::journal::{journal_users, Column, Journal};
use crate::{user, Query as _, Result};

use sea_orm::sea_query::{Expr, SelectStatement, SimpleExpr};
use sea_orm::{sea_query, ColumnTrait, Condition};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug, Default, Eq, PartialEq)]
pub struct Query {
  pub id: HashSet<Uuid>,
  pub name: (String, bool),
  pub description: String,
  pub admin: Option<user::Query>,
  pub member: Option<user::Query>,
}

async fn parse_sub_query(
  query: Option<user::Query>,
  field: journal_users::Field,
) -> Result<Option<SimpleExpr>> {
  let query = match query {
    Some(query) if query != user::Query::default() => query,
    _ => return Ok(None),
  };

  let mut stmt = sea_query::Query::select();

  stmt.distinct().column(journal_users::Column::JournalId).from(journal_users::Entity).inner_join(
    user::Entity,
    Condition::all()
      .add(
        Expr::col((journal_users::Entity, journal_users::Column::UserId))
          .equals((user::Entity, user::Column::Id)),
      )
      .add(journal_users::Column::Field.eq(field)),
  );

  query.parse(&mut stmt).await?;

  Ok(Some(Column::Id.in_subquery(stmt)))
}

#[async_trait::async_trait]
impl crate::Query for Query {
  type AggregateRoot = Journal;

  async fn parse(self, stmt: &mut SelectStatement) -> Result<()> {
    let Query { id, name: (name, name_fulltext), description, admin, member } = self;

    let name = name.trim();
    let description = description.trim();
    stmt
      .and_where_option(if id.is_empty() { None } else { Some(Column::Id.is_in(id)) })
      .and_where_option(if name.is_empty() {
        None
      } else if name_fulltext {
        Some(Column::Name.like(&format!("%{}%", name)))
      } else {
        Some(Column::Name.eq(name))
      })
      .and_where_option(if description.is_empty() {
        None
      } else {
        Some(Column::Description.like(&format!("%{}%", description)))
      })
      .and_where_option(parse_sub_query(admin, journal_users::Field::Admin).await?)
      .and_where_option(parse_sub_query(member, journal_users::Field::Member).await?);

    Ok(())
  }
}
