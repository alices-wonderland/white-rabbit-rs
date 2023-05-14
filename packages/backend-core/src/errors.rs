use sea_orm::DbErr;
use serde::Serialize;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug, Serialize)]
pub enum Error {
  #[error("Not found")]
  NotFound,
  #[error("Internal database error: {0}")]
  Database(
    #[from]
    #[serde(skip_serializing)]
    DbErr,
  ),
}

impl From<Error> for String {
  fn from(value: Error) -> Self {
    value.to_string()
  }
}
