use crate::account::{account_tags, Account, Column, Entity};
use crate::{journal, Result};

use sea_orm::sea_query::{Alias, Expr, SelectStatement};
use sea_orm::{ColumnTrait, Condition, JoinType};
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

#[async_trait::async_trait]
impl crate::Query for Query {
  type AggregateRoot = Account;

  async fn parse(self, stmt: &mut SelectStatement) -> Result<()> {
    let Query { id, name: (name, name_fulltext), description, tag, journal, parent } = self;

    let name = name.trim();
    let description = description.trim();
    let tag = tag.trim();

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

    if !tag.is_empty() {
      stmt
        .left_join(
          account_tags::Entity,
          Expr::col((Entity, Column::Id)).equals(account_tags::Column::AccountId),
        )
        .and_where(account_tags::Column::Tag.like(&format!("%{}%", tag)));
    }

    if !journal.is_empty() {
      let alias = Alias::new("journal");
      stmt
        .join_as(
          JoinType::InnerJoin,
          journal::Entity,
          alias.clone(),
          Expr::col(Column::JournalId).equals((alias.clone(), journal::Column::Id)),
        )
        .and_where(Expr::col((alias, journal::Column::Id)).is_in(journal));
    }

    if !parent.is_empty() {
      let contains_none = parent.contains(&None);
      let parent = parent.into_iter().flatten().collect::<HashSet<_>>();
      stmt.cond_where(
        Condition::any()
          .add_option(if contains_none { Some(Column::ParentId.is_null()) } else { None })
          .add_option(if parent.is_empty() { None } else { Some(Column::ParentId.is_in(parent)) }),
      );
    }

    Ok(())
  }
}
