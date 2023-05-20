use crate::AggregateRoot;

use crate::user::User;
use itertools::Itertools;
use sea_orm::{DbErr, TransactionError};
use serde::Serialize;
use thiserror::Error;
use uuid::Uuid;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug, Serialize)]
pub enum Error {
  #[error("Not found")]
  NotFound,
  #[error("{typ}[{}] already exists", .field_values.iter().map(|(field, value)| format!("{} = {}", field, value)).join(", "))]
  AlreadyExist { typ: String, field_values: Vec<(String, String)> },
  #[error("User[id = {operator_id:?}] has no permission to write {typ}[id = {id}]")]
  NoWritePermission { operator_id: Option<Uuid>, typ: String, id: Uuid },
  #[error("Field[{field}] is not in Range[begin = {begin}, end = {end}]")]
  NotInRange { field: String, begin: usize, end: usize },
  #[error("Internal database error: {0}")]
  Database(
    #[from]
    #[serde(skip_serializing)]
    DbErr,
  ),
}

impl Error {
  pub fn already_exists<A>(
    field_values: impl IntoIterator<Item = (impl ToString, impl ToString)>,
  ) -> Error
  where
    A: AggregateRoot,
  {
    Error::AlreadyExist {
      typ: A::typ().to_string(),
      field_values: field_values
        .into_iter()
        .map(|(field, value)| (field.to_string(), value.to_string()))
        .collect::<Vec<_>>(),
    }
  }

  pub fn no_write_permission<A>(operator: Option<&User>, aggregate_root: &A) -> Error
  where
    A: AggregateRoot,
  {
    Error::NoWritePermission {
      typ: A::typ().to_string(),
      operator_id: operator.map(User::id),
      id: aggregate_root.id(),
    }
  }
}

impl From<Error> for String {
  fn from(value: Error) -> Self {
    value.to_string()
  }
}

impl From<TransactionError<Error>> for Error {
  fn from(value: TransactionError<Error>) -> Self {
    match value {
      TransactionError::Connection(err) => err.into(),
      TransactionError::Transaction(err) => err,
    }
  }
}
