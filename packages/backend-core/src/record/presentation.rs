use crate::record::Record;
use crate::user::User;
use sea_orm::ConnectionTrait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Presentation {}

#[async_trait::async_trait]
impl crate::Presentation for Presentation {
  type AggregateRoot = Record;

  async fn from(
    _db: &impl ConnectionTrait,
    _operator: Option<&User>,
    _roots: Vec<Self::AggregateRoot>,
  ) -> crate::Result<Vec<Self>> {
    Ok(vec![])
  }
}
