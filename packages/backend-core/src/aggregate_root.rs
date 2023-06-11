use crate::user::User;
use crate::{Error, Query, Result};
use sea_orm::entity::prelude::*;
use sea_orm::{IntoActiveModel, StreamTrait};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::Debug;
use uuid::Uuid;

pub const FIELD_ID: &str = "id";
pub const FIELD_NAME: &str = "name";
pub const FIELD_NAME_LENGTH: &str = "name.length";

#[async_trait::async_trait]
pub trait AggregateRoot: Debug + Clone + Send + Sync + Into<Self::Model> {
  type Model: ModelTrait<Entity = Self::Entity> + IntoActiveModel<Self::ActiveModel> + Send;
  type ActiveModel: ActiveModelTrait<Entity = Self::Entity> + Send;
  type Entity: EntityTrait<
    Column = Self::Column,
    Model = Self::Model,
    PrimaryKey = Self::PrimaryKey,
  >;
  type Presentation: Presentation<AggregateRoot = Self>;
  type PrimaryKey: PrimaryKeyTrait<ValueType = Uuid> + PrimaryKeyToColumn<Column = Self::Column>;
  type Query: Query<Column = Self::Column, Entity = Self::Entity>;
  type Column: ColumnTrait;
  type Command;

  fn typ() -> &'static str;

  fn id(&self) -> Uuid;

  fn primary_column() -> Self::Column;

  fn sortable_column(field: impl ToString) -> Option<Self::Column>;

  fn compare_by_field(&self, other: &Self, field: impl ToString) -> Option<Ordering>;

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

  async fn check_writeable(
    db: &impl ConnectionTrait,
    operator: Option<&User>,
    models: &[Self],
  ) -> Result<()> {
    let permissions = Self::get_permission(db, operator, models).await?;
    for model in models {
      if permissions.get(&model.id()) != Some(&Permission::ReadWrite) {
        return Err(Error::no_write_permission(operator, model));
      }
    }
    Ok(())
  }

  async fn pre_save(_db: &impl ConnectionTrait, _models: &[Self]) -> Result<()> {
    Ok(())
  }

  async fn pre_delete(_db: &impl ConnectionTrait, _models: &[Self]) -> Result<()> {
    Ok(())
  }
}

#[async_trait::async_trait]
pub trait Presentation: Serialize + for<'a> Deserialize<'a> + Eq + PartialEq + Send + Sync {
  type AggregateRoot: AggregateRoot<Presentation = Self>;

  async fn from(
    db: &(impl ConnectionTrait + StreamTrait),
    operator: Option<&User>,
    roots: Vec<Self::AggregateRoot>,
  ) -> Result<Vec<Self>>;
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum Permission {
  ReadOnly,
  ReadWrite,
}
