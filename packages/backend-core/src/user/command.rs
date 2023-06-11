use crate::user::Role;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "commandType")]
pub enum Command {
  #[serde(rename = "users:create")]
  Create(CommandCreate),
  #[serde(rename = "users:update")]
  Update(CommandUpdate),
  #[serde(rename = "users:delete")]
  Delete(CommandDelete),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandCreate {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub id: Option<String>,
  pub name: String,
  pub role: Role,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandUpdate {
  pub id: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub role: Option<Role>,
}

impl CommandUpdate {
  pub fn is_empty(&self) -> bool {
    self.name.is_none() && self.role.is_none()
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandDelete {
  pub id: HashSet<Uuid>,
}

#[cfg(test)]
mod tests {
  use serde_json::json;

  use crate::user;

  use super::{Command, CommandCreate};

  #[test]
  fn test_serde() {
    let command = Command::Create(CommandCreate {
      id: Some("new_id".to_string()),
      name: "new_name".to_string(),
      role: user::Role::Admin,
    });
    let json = serde_json::to_value(&command).unwrap();
    assert_eq!(
      json,
      json!({
        "commandType": "users:create",
        "id": "new_id",
        "name": "new_name",
        "role": "Admin"
      })
    );
  }
}
