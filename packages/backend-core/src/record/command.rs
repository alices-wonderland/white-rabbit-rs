use std::collections::HashSet;

use super::{RecordItem, Type};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "commandType")]
pub enum Command {
  #[serde(rename = "records:delete")]
  Create(CommandCreate),
  #[serde(rename = "records:update")]
  Update(CommandUpdate),
  #[serde(rename = "records:batchUpdate")]
  BatchUpdate(CommandBatchUpdate),
  #[serde(rename = "records:delete")]
  Delete(CommandDelete),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandCreate {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub id: Option<String>,
  pub journal: Uuid,
  pub name: String,
  pub description: String,
  #[serde(rename = "type")]
  pub typ: Type,
  pub date: NaiveDate,
  #[serde(default)]
  pub tags: HashSet<String>,
  #[serde(default)]
  pub items: HashSet<RecordItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandUpdate {
  pub id: Uuid,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "type")]
  pub typ: Option<Type>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub date: Option<NaiveDate>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub tags: Option<HashSet<String>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub items: Option<HashSet<RecordItem>>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct CommandBatchUpdate {
  #[serde(skip_serializing_if = "Vec::is_empty")]
  #[serde(default)]
  pub create: Vec<CommandCreate>,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  #[serde(default)]
  pub update: Vec<CommandUpdate>,
}

impl CommandUpdate {
  pub fn is_empty(&self) -> bool {
    self.name.is_none()
      && self.description.is_none()
      && self.typ.is_none()
      && self.date.is_none()
      && self.tags.is_none()
      && self.items.is_none()
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandDelete {
  pub id: HashSet<Uuid>,
}
