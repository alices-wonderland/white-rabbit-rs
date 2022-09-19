use crate::models;

pub mod account;
pub mod group;
pub mod journal;
pub mod read_service;
pub mod user;
pub mod write_service;

const FIELD_ADMINS: &str = "admins";
const FIELD_MEMBERS: &str = "members";
const FIELD_NAME: &str = "name";
const FIELD_DESCRIPTION: &str = "description";
const FIELD_TAG: &str = "tag";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AuthUser {
  Id((String, String)),
  User(models::user::Model),
}

pub use account::AccountService;
pub use group::GroupService;
pub use journal::JournalService;
pub use user::UserService;
