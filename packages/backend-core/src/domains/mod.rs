mod user;

pub use user::*;

use crate::Result;

use sea_orm::sea_query::IntoCondition;
use sea_orm::{
  ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, IntoActiveModel, ModelTrait, PrimaryKeyToColumn,
  PrimaryKeyTrait, StreamTrait, TryIntoModel,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use uuid::Uuid;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum Order {
  Asc,
  Desc,
}

pub trait Query: IntoCondition + Default + Debug + Send {}

impl<Q> Query for Q where Q: IntoCondition + Default + Debug + Send {}

#[async_trait::async_trait]
pub trait AggregateRoot<'a>:
  Debug + Clone + Send + Sync + TryIntoModel<Self::Model> + IntoActiveModel<Self::ActiveModel>
{
  type Model: ModelTrait<Entity = Self::Entity> + IntoActiveModel<Self::ActiveModel>;
  type ActiveModel: ActiveModelTrait<Entity = Self::Entity>;
  type Entity: EntityTrait<Column = Self::Column, Model = Self::Model, PrimaryKey = Self::PrimaryKey>;
  type Presentation: Presentation<'a>;
  type PrimaryKey: PrimaryKeyTrait<ValueType = Uuid> + PrimaryKeyToColumn<Column = Self::Column>;
  type Query: Query;
  type Column: ColumnTrait;
  type Command;

  fn typ() -> &'static str;

  fn id(&self) -> Uuid;

  fn primary_column() -> Self::Column;

  async fn from_models(db: &impl ConnectionTrait, models: Vec<Self::Model>) -> Result<Vec<Self>>;

  async fn handle(
    db: &(impl ConnectionTrait + StreamTrait),
    operator: Option<User>,
    command: Self::Command,
  ) -> Result<Vec<Self>>;

  async fn get_permission(
    db: &impl ConnectionTrait,
    operator: Option<&User>,
    models: &[Self],
  ) -> Result<HashMap<Uuid, Permission>>;

  async fn pre_save(_db: &impl ConnectionTrait, models: &[Self]) -> Result<()> {
    log::info!("pre_save: {:#?}", models);
    Ok(())
  }

  async fn pre_delete(_db: &impl ConnectionTrait, models: &[Self]) -> Result<()> {
    log::info!("pre_delete: {:#?}", models);
    Ok(())
  }
}

#[async_trait::async_trait]
pub trait Presentation<'a>: Serialize + Deserialize<'a> + Send + Sync {
  type AggregateRoot: AggregateRoot<'a, Presentation = Self>;

  async fn from(db: &impl ConnectionTrait, aggregate_roots: Vec<Self::AggregateRoot>) -> Vec<Self>;
}

pub enum Permission {
  ReadOnly,
  ReadWrite,
}
