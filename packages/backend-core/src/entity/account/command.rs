use crate::entity::account;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "commandType")]
pub enum Command {
  #[serde(rename = "accounts:create")]
  Create(CommandCreate),
  #[serde(rename = "accounts:update")]
  Update(CommandUpdate),
  #[serde(rename = "accounts:delete")]
  Delete(CommandDelete),
  #[serde(rename = "accounts:batch")]
  Batch(CommandBatch),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommandCreate {
  pub journal_id: Uuid,
  pub name: String,
  #[serde(default)]
  pub description: String,
  pub unit: String,
  #[serde(rename = "type")]
  pub typ: account::Type,
  #[serde(default)]
  pub tags: HashSet<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommandUpdate {
  pub id: Uuid,
  #[serde(default)]
  pub name: String,
  #[serde(default)]
  pub description: Option<String>,
  #[serde(default)]
  pub unit: String,
  #[serde(rename = "type")]
  #[serde(default)]
  pub typ: Option<account::Type>,
  #[serde(default)]
  pub tags: Option<HashSet<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommandDelete {
  #[serde(default)]
  pub id: HashSet<Uuid>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommandBatch {
  #[serde(default)]
  pub create: Vec<CommandCreate>,
  #[serde(default)]
  pub update: Vec<CommandUpdate>,
  #[serde(default)]
  pub delete: HashSet<Uuid>,
}
