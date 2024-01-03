use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct Query {
  #[serde(default)]
  pub id: HashSet<String>,
  #[serde(default)]
  pub journal_id: HashSet<Uuid>,
  #[serde(default)]
  pub start: Option<NaiveDate>,
  #[serde(default)]
  pub end: Option<NaiveDate>,
}
