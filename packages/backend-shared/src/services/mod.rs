use crate::models;
use serde::{Deserialize, Serialize};

mod account;
mod group;
mod journal;
mod read_service;
mod record;
mod user;
mod write_service;

pub use account::*;
pub use group::*;
pub use journal::*;
pub use read_service::*;
pub use record::*;
pub use user::*;
pub use write_service::*;

pub const FIELD_AUTH_IDS: &str = "auth_ids";
pub const FIELD_ADMINS: &str = "admins";
pub const FIELD_MEMBERS: &str = "members";
pub const FIELD_NAME: &str = "name";
pub const FIELD_ID: &str = "id";
pub const FIELD_DESCRIPTION: &str = "description";
pub const FIELD_TAG: &str = "tag";
pub const FIELD_TAG_ITEM: &str = "tag.item";
pub const FIELD_UNIT: &str = "unit";
pub const FIELD_RECORD_ITEMS: &str = "items";

pub const MIN_AUTH_IDS: usize = 1;
pub const MAX_AUTH_IDS: usize = 3;
pub const MIN_NAME: usize = 4;
pub const MAX_NAME: usize = 128;
pub const MAX_DESCRIPTION: usize = 1024;
pub const MAX_ACCESS_ITEM: usize = 8;
pub const MAX_TAG: usize = 16;
pub const MAX_TAG_ITEM: usize = 16;
pub const MAX_UNIT: usize = 16;
pub const MIN_RECORD_ITEMS: usize = 2;
pub const MAX_RECORD_ITEMS: usize = 16;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, strum_macros::Display)]
pub enum Permission {
  Read,
  Write,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AuthUser {
  Id((String, String)),
  User(models::user::Model),
}

impl From<AuthUser> for models::user::Model {
  fn from(user: AuthUser) -> Self {
    match user {
      AuthUser::Id(_) => unreachable!(),
      AuthUser::User(user) => user,
    }
  }
}

impl AuthUser {
  pub fn get_id(&self) -> String {
    match self {
      AuthUser::Id((_, id)) => id.clone(),
      AuthUser::User(models::user::Model { id, .. }) => id.to_string(),
    }
  }
}
