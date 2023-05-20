use crate::journal::Journal;

use sea_orm::ConnectionTrait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Presentation {}

#[async_trait::async_trait]
impl crate::Presentation for Presentation {
  type AggregateRoot = Journal;

  async fn from(_db: &impl ConnectionTrait, _models: Vec<Self::AggregateRoot>) -> Vec<Self> {
    Vec::default()
  }
}
