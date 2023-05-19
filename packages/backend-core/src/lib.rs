mod domains;
mod errors;

pub use domains::*;
pub use errors::{Error, Result};

use futures::{stream, Stream, StreamExt, TryStreamExt};
use sea_orm::sea_query::{Expr, IntoCondition};
use sea_orm::{ConnectionTrait, EntityTrait, IntoActiveModel, QueryFilter, QueryOrder, StreamTrait};
use std::collections::HashSet;
use std::marker::PhantomData;
use std::pin::Pin;
use std::str::FromStr;
use uuid::Uuid;

const CHUNK_SIZE: usize = 100;

#[derive(Debug, Default)]
pub struct FindAllArgs<Q>
where
  Q: Query,
{
  pub query: Q,
  pub sort: Vec<(String, Order)>,
}

pub struct Repository<'a, A>
where
  A: AggregateRoot<'a> + 'a,
{
  _marker: PhantomData<&'a A>,
}

impl<'a, A> Repository<'a, A>
where
  A: AggregateRoot<'a> + 'a,
{
  pub async fn find_by_id(db: &impl ConnectionTrait, id: Uuid) -> Result<Option<A>> {
    Ok(Self::find_by_ids(db, vec![id]).await?.into_iter().last())
  }

  pub async fn find_by_ids<I>(db: &impl ConnectionTrait, ids: I) -> Result<Vec<A>>
  where
    I: IntoIterator<Item = Uuid>,
  {
    let models = A::Entity::find().filter(Expr::col(A::primary_column()).is_in(ids)).all(db).await?;
    A::from_models(db, models).await
  }

  pub async fn find_all(
    db: &'a (impl ConnectionTrait + StreamTrait),
    args: FindAllArgs<A::Query>,
  ) -> Result<Pin<Box<dyn 'a + Send + Stream<Item = Result<A>>>>> {
    let mut filter = A::Entity::find().filter(args.query.into_condition());

    for (field, order) in args.sort {
      if let Ok(field) = A::Column::from_str(&field) {
        filter = match order {
          Order::Asc => filter.order_by_asc(field),
          Order::Desc => filter.order_by_desc(field),
        }
      }
    }

    let stream = filter
      .stream(db)
      .await?
      .map_err(Error::from)
      .try_chunks(CHUNK_SIZE)
      .map_err(|err| err.1)
      .and_then(|models| A::from_models(db, models))
      .flat_map(|result| {
        stream::iter(match result {
          Ok(roots) => roots.into_iter().map(|root| Ok(root)).collect::<Vec<_>>(),
          Err(err) => vec![Err(err)],
        })
      });
    Ok(Box::pin(stream))
  }

  pub async fn save(db: &impl ConnectionTrait, aggregate_roots: Vec<A>) -> Result<Vec<A>> {
    if aggregate_roots.is_empty() {
      return Ok(Vec::default());
    }
    A::pre_save(db, &aggregate_roots).await?;
    let ids = aggregate_roots.iter().map(|root| root.id()).collect::<HashSet<_>>();
    let _ =
      A::Entity::insert_many(aggregate_roots.into_iter().map(IntoActiveModel::into_active_model).collect::<Vec<_>>())
        .exec(db)
        .await?;

    Self::find_by_ids(db, ids).await
  }

  pub async fn delete(db: &impl ConnectionTrait, aggregate_roots: Vec<A>) -> Result<()> {
    if aggregate_roots.is_empty() {
      return Ok(());
    }
    A::pre_delete(db, &aggregate_roots).await?;
    let ids = aggregate_roots.iter().map(|root| root.id()).collect::<HashSet<_>>();
    let _ = A::Entity::delete_many().filter(Expr::col(A::primary_column()).is_in(ids)).exec(db).await?;
    Ok(())
  }
}
