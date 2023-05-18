#![feature(async_fn_in_trait, impl_trait_projections)]

mod domains;
mod errors;

pub use domains::*;
pub use errors::{Error, Result};
use futures::{Stream, TryStreamExt};
use sea_orm::sea_query::Expr;
use sea_orm::{
  ActiveModelTrait, ConnectionTrait, EntityTrait, IntoActiveModel, Iterable, ModelTrait, PrimaryKeyToColumn,
  PrimaryKeyTrait, QueryFilter, StreamTrait,
};
use std::collections::HashSet;
use std::pin::Pin;
use uuid::Uuid;

pub trait Repository<'a> {
  type AggregateRoot: AggregateRoot<'a, Model = Self::Model, Presentation = Self::Presentation, ActiveModel = Self::ActiveModel>;

  type Model: ModelTrait<Entity = Self::Entity> + IntoActiveModel<Self::ActiveModel>;
  type ActiveModel: ActiveModelTrait<Entity = Self::Entity>;
  type Entity: EntityTrait<Model = Self::Model, PrimaryKey = Self::PrimaryKey>;
  type Presentation: Presentation<'a>;
  type PrimaryKey: PrimaryKeyTrait<ValueType = Uuid> + PrimaryKeyToColumn;

  async fn find_by_id(db: &impl ConnectionTrait, id: Uuid) -> Result<Option<Self::Model>> {
    Ok(Self::Entity::find_by_id(id).one(db).await?)
  }

  async fn find_by_ids(
    db: &impl ConnectionTrait,
    ids: impl IntoIterator<Item = Uuid>,
  ) -> Result<Vec<Self::AggregateRoot>> {
    let models = Self::Entity::find()
      .filter(Expr::col(Self::PrimaryKey::iter().last().unwrap().into_column()).is_in(ids))
      .all(db)
      .await?;
    Self::AggregateRoot::from_models(db, models).await
  }

  async fn find_all<C>(db: &'a C) -> Result<Pin<Box<dyn Stream<Item = Result<Self::Model>> + Send + 'a>>>
  where
    C: ConnectionTrait + StreamTrait,
  {
    Ok(Box::pin(Self::Entity::find().stream(db).await?.map_err(crate::Error::from)))
  }

  async fn pre_save<C>(_db: &C, _models: &[Self::AggregateRoot]) -> Result<()>
  where
    C: ConnectionTrait,
  {
    Ok(())
  }

  async fn save<C>(db: &C, aggregate_roots: Vec<Self::AggregateRoot>) -> Result<Vec<Self::AggregateRoot>>
  where
    C: ConnectionTrait,
  {
    if aggregate_roots.is_empty() {
      return Ok(Vec::default());
    }

    Self::pre_save(db, &aggregate_roots).await?;
    let ids = aggregate_roots.iter().map(|root| root.id()).collect::<HashSet<_>>();
    let _ = Self::Entity::insert_many(
      aggregate_roots.into_iter().map(IntoActiveModel::into_active_model).collect::<Vec<_>>(),
    )
    .exec(db)
    .await?;

    Self::find_by_ids(db, ids).await
  }
}

pub async fn create(db: &impl ConnectionTrait, command: UserCommandCreate) -> Result<Option<User>> {
  let user = User { id: Uuid::new_v4(), name: command.name, role: command.role };
  Ok(UserRepository::save(db, vec![user]).await?.first().cloned())
}
