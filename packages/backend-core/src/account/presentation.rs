use crate::account::{Account, Type};
use crate::user::User;
use crate::{AggregateRoot, Permission, Result};
use sea_orm::{ConnectionTrait, StreamTrait};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Presentation {
  pub id: Uuid,
  pub permission: Permission,
  pub model_type: String,
  pub name: String,
  pub description: String,
  pub unit: String,
  #[serde(rename = "type")]
  pub typ: Type,
  pub tags: HashSet<String>,
  pub journal: Uuid,
}

#[async_trait::async_trait]
impl crate::Presentation for Presentation {
  type AggregateRoot = Account;

  async fn from_aggregate_roots(
    db: &(impl ConnectionTrait + StreamTrait),
    operator: Option<&User>,
    roots: Vec<Self::AggregateRoot>,
  ) -> Result<Vec<Self>> {
    let permissions = AggregateRoot::get_permission(db, operator, &roots).await?;
    Ok(
      roots
        .into_iter()
        .filter_map(|Account { id, name, description, unit, typ, tags, journal }| {
          permissions.get(&id).map(|permission| Self {
            id,
            permission: *permission,
            model_type: Account::typ().to_string(),
            name,
            description,
            unit,
            typ,
            tags,
            journal,
          })
        })
        .collect(),
    )
  }
}
