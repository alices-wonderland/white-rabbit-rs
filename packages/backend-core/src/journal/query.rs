use crate::journal::{journal_users, Column, Journal};
use crate::Result;

use sea_orm::sea_query::{Expr, SelectStatement, SimpleExpr};
use sea_orm::{ColumnTrait, Condition};
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

fn user_query(ids: HashSet<Uuid>, is_admin: bool) -> Option<SimpleExpr> {
  if ids.is_empty() {
    None
  } else {
    Some(
      journal_users::Column::Field
        .eq(if is_admin { journal_users::Field::Admin } else { journal_users::Field::Member })
        .and(journal_users::Column::UserId.is_in(ids)),
    )
  }
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
      });

    if !admin.is_empty() || !member.is_empty() {
      stmt
        .left_join(
          journal_users::Entity,
          Expr::col(journal_users::Column::JournalId).equals(Column::Id),
        )
        .cond_where(
          Condition::any()
            .add_option(user_query(admin, true))
            .add_option(user_query(member, false)),
        );
    }

    Ok(())
  }
}
