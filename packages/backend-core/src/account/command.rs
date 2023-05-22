use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub enum Command {
  Create(CommandCreate),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandCreate {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub id: Option<String>,
  pub name: String,
  pub description: String,
  pub tags: HashSet<String>,
  pub journal: Uuid,
  pub parent: Option<Uuid>,
}
