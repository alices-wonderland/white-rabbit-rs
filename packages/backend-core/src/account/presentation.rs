use crate::account::Account;
use crate::user::User;
use crate::{AggregateRoot, Permission, Result};
use sea_orm::ConnectionTrait;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Presentation {
  pub id: Uuid,
  pub permission: Permission,
  pub name: String,
  pub description: String,
  pub tags: HashSet<String>,
  pub journal: Uuid,
  pub parent: Option<Uuid>,
}

#[async_trait::async_trait]
impl crate::Presentation for Presentation {
  type AggregateRoot = Account;

  async fn from(
    db: &impl ConnectionTrait,
    operator: Option<&User>,
    roots: Vec<Self::AggregateRoot>,
  ) -> Result<Vec<Self>> {
    let permissions = AggregateRoot::get_permission(db, operator, &roots).await?;
    Ok(
      roots
        .into_iter()
        .filter_map(|Account { id, name, description, tags, journal, parent }| {
          permissions.get(&id).map(|permission| Self {
            id,
            permission: *permission,
            name,
            description,
            tags,
            journal,
            parent,
          })
        })
        .collect(),
    )
  }
}
