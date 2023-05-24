use crate::user::User;
use crate::{AggregateRoot, Error, Presentation, Result};
use futures::lock::Mutex;
use futures::{stream, Stream, StreamExt, TryStreamExt};
use sea_orm::entity::prelude::*;
use sea_orm::sea_query::Expr;
use sea_orm::{QuerySelect, StreamTrait};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashSet;
use std::fmt::Debug;
use std::future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;
use uuid::Uuid;

const CHUNK_SIZE: usize = 100;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum Order {
  Asc,
  Desc,
}

pub type Sort = Vec<(String, Order)>;

pub trait Query: Default + Clone + Debug + Send {}

#[derive(Debug, Default, Clone)]
pub struct FindAllArgs<'a, Q>
where
  Q: Query,
{
  pub operator: Option<&'a User>,
  pub query: Q,
  pub sort: Sort,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

  pub async fn find_all<'a>(
    db: &'a (impl ConnectionTrait + StreamTrait),
    args: FindAllArgs<'a, A::Query>,
  ) -> Result<Pin<Box<dyn 'a + Send + Stream<Item = Result<A>>>>> {
    let operator = Arc::new(Mutex::new(args.operator));
    let sub_db = Arc::new(Mutex::new(db));

    let select = A::Entity::find().distinct();
    let select = A::parse_join(select, &args.query, &args.sort);
    let select = A::parse_query(select, args.query);
    let select = A::parse_order(select, args.sort);

    let stream = select
      .stream(db)
      .await?
      .map_err(Error::from)
      .try_chunks(CHUNK_SIZE)
      .map_err(|err| err.1)
      .zip(stream::repeat((sub_db.clone(), operator.clone())))
      .map(|(result, (db, operator))| match result {
        Ok(result) => Ok((db, operator, result)),
        Err(err) => Err(err),
      })
      .and_then(|(db, operator, models)| async move {
        let db = db.lock().await;
        let operator = operator.lock().await;
        let results = A::from_models(*db, models).await?;
        let permissions = A::get_permission(*db, *operator, &results).await?;
        Ok(
          results
            .into_iter()
            .filter(|model| permissions.contains_key(&model.id()))
            .collect::<Vec<_>>(),
        )
      })
      .flat_map(|result| {
        stream::iter(match result {
          Ok(roots) => roots.into_iter().map(|root| Ok(root)).collect::<Vec<_>>(),
          Err(err) => vec![Err(err)],
        })
      });
    Ok(Box::pin(stream))
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

    let items = Self::find_all(
      db,
      FindAllArgs {
        operator,
        query,
        sort: sort
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
          .collect::<Vec<_>>(),
      },
    )
    .await?
    .try_filter(|model| {
      future::ready(filter_between(model, &sort, after.as_ref(), before.as_ref()))
    })
    .take(size + 1)
    .try_collect::<Vec<_>>()
    .await?;

    let (has_previous, has_next) = if after.is_some() {
      (true, items.len() > size)
    } else if before.is_some() {
      (items.len() > size, true)
    } else {
      (false, items.len() > size)
    };

    let mut items = Presentation::from(db, items.into_iter().take(size).collect::<Vec<_>>()).await;

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
    let ids = aggregate_roots.iter().map(|root| root.id()).collect::<HashSet<_>>();
    A::do_save(db, aggregate_roots).await?;

    Self::find_by_ids(db, ids).await
  }

  pub async fn delete(db: &impl ConnectionTrait, aggregate_roots: Vec<A>) -> Result<()> {
    if aggregate_roots.is_empty() {
      return Ok(());
    }
    A::pre_delete(db, &aggregate_roots).await?;
    let ids = aggregate_roots.iter().map(|root| root.id()).collect::<HashSet<_>>();
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
