use crate::entity::entry;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
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
  pub typ: entry::Type,
  pub date: NaiveDate,
  pub tags: HashSet<String>,
  pub items: HashMap<Uuid, (Decimal, Option<Decimal>)>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommandUpdate {
  pub id: Uuid,
  pub name: String,
  pub description: Option<String>,
  pub typ: Option<entry::Type>,
  pub tags: Option<HashSet<String>>,
  pub items: HashMap<Uuid, (Decimal, Option<Decimal>)>,
}
