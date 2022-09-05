use crate::models;

pub mod read_service;
pub mod user;
pub mod write_service;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AuthUser {
  Id((String, String)),
  User(models::user::Model),
}
