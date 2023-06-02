use crate::account::{Account, Type};
use crate::user::User;
use crate::{AggregateRoot, Permission, Result};
use sea_orm::{ConnectionTrait, StreamTrait};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Presentation {
  pub id: Uuid,
  pub permission: Permission,
  pub name: String,
  pub description: String,
  pub unit: String,
  #[serde(rename = "type")]
  pub typ: Type,
  pub tags: HashSet<String>,
  pub journal: Uuid,
  pub parent: Option<Uuid>,
}

#[async_trait::async_trait]
impl crate::Presentation for Presentation {
  type AggregateRoot = Account;

  async fn from(
    db: &(impl ConnectionTrait + StreamTrait),
    operator: Option<&User>,
    roots: Vec<Self::AggregateRoot>,
  ) -> Result<Vec<Self>> {
    let permissions = AggregateRoot::get_permission(db, operator, &roots).await?;
    Ok(
      roots
        .into_iter()
        .filter_map(|Account { id, name, description, unit, typ, tags, journal, parent }| {
          permissions.get(&id).map(|permission| Self {
            id,
            permission: *permission,
            name,
            description,
            unit,
            typ,
            tags,
            journal,
            parent,
          })
        })
        .collect(),
    )
  }
}
