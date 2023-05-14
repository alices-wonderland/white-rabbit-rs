#![feature(async_fn_in_trait, impl_trait_projections)]

mod domains;
mod errors;

pub use domains::*;
pub use errors::{Error, Result};
use futures::{Stream, TryStreamExt};
use sea_orm::ActiveValue::Set;
use sea_orm::{ConnectionTrait, EntityTrait, ModelTrait, PrimaryKeyToColumn, PrimaryKeyTrait, StreamTrait};
use std::pin::Pin;
use uuid::Uuid;

pub trait Repository<'a> {
  type AggregateRoot: AggregateRoot<'a>;

  type Model: ModelTrait<Entity = Self::Entity>;
  type Entity: EntityTrait<Model = Self::Model, PrimaryKey = Self::PrimaryKey>;
  type Presentation: Presentation<'a>;
  type PrimaryKey: PrimaryKeyTrait<ValueType = Uuid> + PrimaryKeyToColumn;

  async fn find_by_id<C: ConnectionTrait>(db: &C, id: Uuid) -> Result<Option<Self::Model>> {
    Ok(Self::Entity::find_by_id(id).one(db).await?)
  }

  async fn find_all<C>(db: &'a C) -> Result<Pin<Box<dyn Stream<Item = Result<Self::Model>> + Send + 'a>>>
  where
    C: ConnectionTrait + StreamTrait,
  {
    Ok(Box::pin(Self::Entity::find().stream(db).await?.map_err(crate::Error::from)))
  }
}

pub async fn create(db: &impl ConnectionTrait, command: UserCommandCreate) -> Result<Option<User>> {
  let id = UserEntity::insert_many((0..100).map(|i| UserActiveModel {
    id: Set(Uuid::new_v4()),
    name: Set(format!("{} - {}", command.name, i)),
    role: Set(command.role.clone()),
  }))
  .exec(db)
  .await?;
  Ok(UserEntity::find_by_id(id.last_insert_id).one(db).await?)
}
