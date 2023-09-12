use crate::entity::account;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Command {
  Create(CommandCreate),
  Update(CommandUpdate),
  Delete(HashSet<Uuid>),
  Batch(Vec<CommandCreate>, Vec<CommandUpdate>, HashSet<Uuid>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommandCreate {
  pub journal_id: Uuid,
  pub name: String,
  pub description: String,
  pub unit: String,
  pub typ: account::Type,
  pub tags: HashSet<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommandUpdate {
  pub id: Uuid,
  pub name: String,
  pub description: Option<String>,
  pub unit: String,
  pub typ: Option<account::Type>,
  pub tags: Option<HashSet<String>>,
}
