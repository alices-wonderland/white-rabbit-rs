use crate::user::Role;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub enum Command {
  Create(CommandCreate),
  Update(CommandUpdate),
  Delete(CommandDelete),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandCreate {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub id: Option<String>,
  pub name: String,
  pub role: Role,
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandDelete {
  pub id: HashSet<Uuid>,
}
