use crate::user::User;
use crate::{Query, Result};

use sea_orm::{
  ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, IntoActiveModel, ModelTrait,
  PrimaryKeyToColumn, PrimaryKeyTrait, StreamTrait,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use uuid::Uuid;

#[async_trait::async_trait]
pub trait AggregateRoot: Debug + Clone + Send + Sync + Into<Self::Model> {
  type Model: ModelTrait<Entity = Self::Entity> + IntoActiveModel<Self::ActiveModel> + Send;
  type ActiveModel: ActiveModelTrait<Entity = Self::Entity> + Send;
  type Entity: EntityTrait<
    Column = Self::Column,
    Model = Self::Model,
    PrimaryKey = Self::PrimaryKey,
  >;
  type Presentation: Presentation;
  type PrimaryKey: PrimaryKeyTrait<ValueType = Uuid> + PrimaryKeyToColumn<Column = Self::Column>;
  type Query: Query<AggregateRoot = Self>;
  type Column: ColumnTrait;
  type Command;

  fn typ() -> &'static str;

  fn id(&self) -> Uuid;

  fn primary_column() -> Self::Column;

  async fn from_models(db: &impl ConnectionTrait, models: Vec<Self::Model>) -> Result<Vec<Self>>;

  async fn do_save(db: &impl ConnectionTrait, roots: Vec<Self>) -> Result<()> {
    Self::Entity::insert_many(
      roots.into_iter().map(|root| root.into().into_active_model()).collect::<Vec<_>>(),
    )
    .exec(db)
    .await?;

    Ok(())
  }

  async fn handle(
    db: &(impl ConnectionTrait + StreamTrait),
    operator: Option<&User>,
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
pub trait Presentation: Serialize + for<'a> Deserialize<'a> + Send + Sync {
  type AggregateRoot: AggregateRoot<Presentation = Self>;

  async fn from(db: &impl ConnectionTrait, aggregate_roots: Vec<Self::AggregateRoot>) -> Vec<Self>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Permission {
  ReadOnly,
  ReadWrite,
}
