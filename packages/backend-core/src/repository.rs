use crate::user::User;
use crate::{utils, AggregateRoot, Presentation, Result};
use sea_orm::entity::prelude::*;
use sea_orm::sea_query::{Expr, SimpleExpr};
use sea_orm::{QueryOrder, QuerySelect, StreamTrait};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashSet;
use std::fmt::Debug;

use std::marker::PhantomData;

use uuid::Uuid;

const CHUNK_SIZE: usize = 100;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum Order {
  Asc,
  Desc,
}

pub type Sort = Vec<(String, Order)>;

pub trait Query:
  Default + Clone + Debug + Send + Sync + Into<Select<Self::Entity>> + From<HashSet<Uuid>>
{
  type Entity: EntityTrait<Column = Self::Column>;
  type Column: ColumnTrait;

  fn id_expr(column: Self::Column, id: HashSet<Uuid>) -> Option<SimpleExpr> {
    if id.is_empty() {
      None
    } else {
      Some(column.is_in(id))
    }
  }

  fn text_expr(
    column: Self::Column,
    (name, fulltext): (impl ToString, bool),
  ) -> Option<SimpleExpr> {
    match name.to_string().trim() {
      "" => None,
      value => Some(if fulltext { column.like(&format!("%{}%", value)) } else { column.eq(value) }),
    }
  }
}

#[derive(Debug, Default, Clone)]
pub struct FindAllArgs<Q>
where
  Q: Query,
{
  pub query: Q,
  pub sort: Option<Sort>,
  pub size: Option<usize>,
}

#[derive(Debug, Default, Clone)]
pub struct FindPageArgs<'a, Q>
where
  Q: Query,
{
  pub operator: Option<&'a User>,
  pub query: Q,
  pub sort: Sort,
  pub after: Option<Uuid>,
  pub before: Option<Uuid>,
  pub size: usize,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Page<A>
where
  A: AggregateRoot,
{
  pub items: Vec<A::Presentation>,
  pub has_previous: bool,
  pub has_next: bool,
}

pub struct Repository<A>
where
  A: AggregateRoot,
{
  _marker: PhantomData<A>,
}

impl<A> Repository<A>
where
  for<'a> A: 'a + AggregateRoot,
{
  pub async fn find_by_id(db: &impl ConnectionTrait, id: Uuid) -> Result<Option<A>> {
    Ok(Self::find_by_ids(db, vec![id]).await?.into_iter().last())
  }

  pub async fn find_by_ids<I>(db: &impl ConnectionTrait, ids: I) -> Result<Vec<A>>
  where
    I: IntoIterator<Item = Uuid>,
  {
    let models =
      A::Entity::find().filter(Expr::col(A::primary_column()).is_in(ids)).all(db).await?;
    A::from_models(db, models).await
  }

  pub async fn do_find_all(
    db: &(impl ConnectionTrait + StreamTrait),
    args: FindAllArgs<A::Query>,
    offset: Option<usize>,
  ) -> Result<Vec<A>> {
    let mut select: Select<A::Entity> = args.query.into();

    if let Some(size) = args.size {
      select = select.offset(offset.map(|n| n as u64)).limit(size as u64);
    }

    if let Some(sort) = args.sort {
      for (field, order) in sort {
        if let Some(column) = A::sortable_column(field) {
          select = match order {
            Order::Asc => select.order_by_asc(column),
            Order::Desc => select.order_by_desc(column),
          }
        }
      }
    }

    A::from_models(db, select.all(db).await?).await
  }

  pub async fn find_all<'a>(
    db: &'a (impl ConnectionTrait + StreamTrait),
    operator: Option<&'a User>,
    args: FindAllArgs<A::Query>,
  ) -> Result<Vec<A>> {
    let mut results = Vec::default();
    let size = args.size.unwrap_or(CHUNK_SIZE);
    let mut count = 0;

    while results.len() <= size {
      let roots =
        Self::do_find_all(db, FindAllArgs { size: Some(size), ..args.clone() }, Some(count * size))
          .await?;
      if roots.is_empty() {
        break;
      }

      count += 1;
      let permissions = A::get_permission(db, operator, &roots).await?;
      for root in roots {
        if permissions.contains_key(&root.id()) {
          results.push(root);
        }
      }
    }

    Ok(results.into_iter().take(size).collect())
  }

  pub async fn find_page<'a>(
    db: &'a (impl ConnectionTrait + StreamTrait),
    FindPageArgs { operator, query, sort, after, before, size }: FindPageArgs<'a, A::Query>,
  ) -> Result<Page<A>> {
    let after = if let Some(after) = after { Self::find_by_id(db, after).await? } else { None };
    let before = if let Some(before) = before { Self::find_by_id(db, before).await? } else { None };
    let should_reverse = after.is_none() && before.is_some();

    let sort = if !sort.iter().any(|(field, _)| field == "id") {
      let mut sort = Vec::from_iter(sort);
      sort.push(("id".to_string(), Order::Asc));
      sort
    } else {
      sort
    };

    let mut items = Vec::default();
    let mut count = 0;

    while items.len() < size + 1 {
      let roots = Self::do_find_all(
        db,
        FindAllArgs {
          size: Some(size + 1),
          query: query.clone(),
          sort: Some(
            sort
              .iter()
              .map(|(field, order)| {
                (
                  field.to_string(),
                  if should_reverse {
                    match order {
                      Order::Asc => Order::Desc,
                      Order::Desc => Order::Asc,
                    }
                  } else {
                    order.clone()
                  },
                )
              })
              .collect(),
          ),
        },
        Some(count * (size + 1)),
      )
      .await?;
      if roots.is_empty() {
        break;
      }

      count += 1;
      let permissions = A::get_permission(db, operator, &roots).await?;
      for root in roots {
        if permissions.contains_key(&root.id())
          && filter_between(&root, &sort, after.as_ref(), before.as_ref())
        {
          items.push(root);
        }
      }
    }

    let (has_previous, has_next) = if after.is_some() {
      (true, items.len() > size)
    } else if before.is_some() {
      (items.len() > size, true)
    } else {
      (false, items.len() > size)
    };

    let mut items = Presentation::from_aggregate_roots(
      db,
      operator,
      items.into_iter().take(size).collect::<Vec<_>>(),
    )
    .await?;

    if should_reverse {
      items.reverse();
    }

    Ok(Page { items, has_previous, has_next })
  }

  pub async fn save(db: &impl ConnectionTrait, aggregate_roots: Vec<A>) -> Result<Vec<A>> {
    if aggregate_roots.is_empty() {
      return Ok(Vec::default());
    }
    A::pre_save(db, &aggregate_roots).await?;
    let ids = utils::get_ids(&aggregate_roots);
    A::do_save(db, aggregate_roots).await?;

    Self::find_by_ids(db, ids).await
  }

  pub async fn delete(db: &impl ConnectionTrait, aggregate_roots: Vec<A>) -> Result<()> {
    if aggregate_roots.is_empty() {
      return Ok(());
    }
    A::pre_delete(db, &aggregate_roots).await?;
    let ids = utils::get_ids(&aggregate_roots);
    let _ =
      A::Entity::delete_many().filter(Expr::col(A::primary_column()).is_in(ids)).exec(db).await?;
    Ok(())
  }
}

fn compare<A: AggregateRoot>(
  root: &A,
  other: Option<&A>,
  field: impl ToString,
  expected: Ordering,
) -> bool {
  if let Some(other) = other {
    root.compare_by_field(other, field).map(|ordering| ordering == expected).unwrap_or(false)
  } else {
    true
  }
}

fn filter_between<A: AggregateRoot>(
  root: &A,
  sort: &[(String, Order)],
  after: Option<&A>,
  before: Option<&A>,
) -> bool {
  if let Some(((field, order), head)) = sort.split_last() {
    let head_matches = head.iter().all(|(field, _)| {
      compare(root, after, field, Ordering::Equal) && compare(root, before, field, Ordering::Equal)
    });
    let last_matches = if order == &Order::Asc {
      compare(root, after, field, Ordering::Greater) && compare(root, before, field, Ordering::Less)
    } else {
      compare(root, after, field, Ordering::Less) && compare(root, after, field, Ordering::Greater)
    };

    (head_matches && last_matches) || filter_between(root, head, after, before)
  } else {
    false
  }
}
