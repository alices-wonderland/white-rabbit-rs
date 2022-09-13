use serde::{Deserialize, Serialize};

use crate::models::account;

use super::read_service::{IdQuery, TextQuery};

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct AccountQuery {
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(default)]
  pub id: Option<IdQuery>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(default)]
  pub journal: Option<uuid::Uuid>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(default)]
  pub name: Option<TextQuery>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(default)]
  pub description: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(default)]
  pub typ: Option<account::Type>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(default)]
  pub strategy: Option<account::Strategy>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(default)]
  pub unit: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(default)]
  pub include_archived: Option<bool>,
}
