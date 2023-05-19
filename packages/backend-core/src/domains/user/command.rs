use crate::domains::user::Role;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum UserCommand {
  Create(UserCommandCreate),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserCommandCreate {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub id: Option<String>,
  pub name: String,
  pub role: Role,
}
