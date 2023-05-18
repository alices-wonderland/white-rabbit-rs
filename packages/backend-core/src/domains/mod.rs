mod user;

pub use user::*;

use crate::Result;
use sea_orm::{ActiveModelTrait, ConnectionTrait, IntoActiveModel, ModelTrait, TryIntoModel};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use uuid::Uuid;

pub trait AggregateRoot<'a>:
  Debug
  + Send
  + Sync
  + IntoPresentation<'a, Self::Presentation>
  + TryIntoModel<Self::Model>
  + IntoActiveModel<Self::ActiveModel>
{
  type Model: ModelTrait;
  type ActiveModel: ActiveModelTrait;
  type Presentation: Presentation<'a, AggregateRoot = Self>;

  fn id(&self) -> Uuid;
  async fn from_models(db: &impl ConnectionTrait, models: Vec<Self::Model>) -> Result<Vec<Self>>;
}

pub trait Presentation<'a>: Serialize + Deserialize<'a> {
  type AggregateRoot: AggregateRoot<'a, Presentation = Self>;

  async fn from(db: &impl ConnectionTrait, aggregate_roots: Vec<Self::AggregateRoot>) -> Result<Vec<Self>>;
}

pub trait IntoPresentation<'a, P>
where
  P: Presentation<'a>,
  Self: Sized,
{
  async fn into(db: &impl ConnectionTrait, models: Vec<Self>) -> Result<Vec<P>>;
}

impl<'a, A, P> IntoPresentation<'a, P> for A
where
  A: AggregateRoot<'a>,
  P: Presentation<'a, AggregateRoot = A>,
{
  async fn into(db: &impl ConnectionTrait, aggregate_roots: Vec<A>) -> Result<Vec<P>> {
    P::from(db, aggregate_roots).await
  }
}
