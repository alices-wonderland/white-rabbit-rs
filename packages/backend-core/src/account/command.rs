use super::Type;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "commandType")]
pub enum Command {
  #[serde(rename = "accounts:create")]
  Create(CommandCreate),
  #[serde(rename = "accounts:update")]
  Update(CommandUpdate),
  #[serde(rename = "accounts:delete")]
  Delete(CommandDelete),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandCreate {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub id: Option<String>,
  pub journal: Uuid,
  pub name: String,
  pub description: String,
  pub unit: String,
  #[serde(rename = "type")]
  pub typ: Type,
  #[serde(default)]
  pub tags: HashSet<String>,
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
  #[serde(rename = "type")]
  pub typ: Option<Type>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub tags: Option<HashSet<String>>,
}

impl CommandUpdate {
  pub fn is_empty(&self) -> bool {
    self.name.is_none()
      && self.description.is_none()
      && self.unit.is_none()
      && self.typ.is_none()
      && self.tags.is_none()
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandDelete {
  pub id: HashSet<Uuid>,
}
