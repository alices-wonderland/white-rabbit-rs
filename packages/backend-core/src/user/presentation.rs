use crate::user::User;
use crate::Presentation;

use sea_orm::entity::prelude::*;

#[async_trait::async_trait]
impl Presentation for User {
  type AggregateRoot = User;

  async fn from(_db: &impl ConnectionTrait, models: Vec<Self::AggregateRoot>) -> Vec<Self> {
    models
  }
}
