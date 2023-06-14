use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub enum Command {
  #[serde(rename = "journals:create")]
  Create(CommandCreate),
  #[serde(rename = "journals:update")]
  Update(CommandUpdate),
  #[serde(rename = "journals:delete")]
  Delete(CommandDelete),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandCreate {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub id: Option<String>,
  pub name: String,
  pub description: String,
  pub unit: String,
  #[serde(default)]
  pub admins: HashSet<Uuid>,
  #[serde(default)]
  pub members: HashSet<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandUpdate {
  pub id: Uuid,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub unit: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub admins: Option<HashSet<Uuid>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub members: Option<HashSet<Uuid>>,
}

impl CommandUpdate {
  pub fn is_empty(&self) -> bool {
    self.name.is_none()
      && self.description.is_none()
      && self.unit.is_none()
      && self.admins.is_none()
      && self.members.is_none()
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandDelete {
  pub id: HashSet<Uuid>,
}
