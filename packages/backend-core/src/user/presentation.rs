use crate::user::{Role, User};
use crate::{AggregateRoot, Permission, Result};
use sea_orm::{ConnectionTrait, StreamTrait};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Presentation {
  pub id: Uuid,
  pub permission: Permission,
  pub model_type: String,
  pub name: String,
  pub role: Role,
}

#[async_trait::async_trait]
impl crate::Presentation for Presentation {
  type AggregateRoot = User;

  async fn from_aggregate_roots(
    db: &(impl ConnectionTrait + StreamTrait),
    operator: Option<&User>,
    roots: Vec<Self::AggregateRoot>,
  ) -> Result<Vec<Self>> {
    let permissions = AggregateRoot::get_permission(db, operator, &roots).await?;
    Ok(
      roots
        .into_iter()
        .filter_map(|User { id, name, role }| {
          permissions.get(&id).map(|permission| Self {
            id,
            permission: *permission,
            model_type: User::typ().to_string(),
            name,
            role,
          })
        })
        .collect(),
    )
  }
}
