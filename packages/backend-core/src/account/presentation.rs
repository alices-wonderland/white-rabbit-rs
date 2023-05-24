use crate::account::Account;
use sea_orm::ConnectionTrait;

#[async_trait::async_trait]
impl crate::Presentation for Account {
  type AggregateRoot = Self;

  async fn from(_db: &impl ConnectionTrait, roots: Vec<Self::AggregateRoot>) -> Vec<Self> {
    roots
  }
}
