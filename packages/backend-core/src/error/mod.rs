mod problem_detail;

pub use problem_detail::*;

use http::StatusCode;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum Error {
  #[error("{}", .0.detail())]
  NotFound(ErrorNotFound),

  #[error("Entity[{typ}, {}] is existing", .values.iter().map(|(f, v)| format!("{} = {}", f, v)).join(", "))]
  ExistingEntity { typ: String, values: Vec<(String, String)> },

  #[error("Field[{field}] of Entity[{typ}] should in Range[start = {start:?}, end = {end:?}]")]
  OutOfRange { typ: String, field: String, start: Option<String>, end: Option<String> },

  #[error("Field[{field}] of Entity[{typ}] is required")]
  RequiredField { typ: String, field: String },

  #[error("Database Error: {0}")]
  Database(#[from] sea_orm::DbErr),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ErrorNotFound {
  pub entity: String,
  pub values: Vec<(String, String)>,
}

impl ProblemDetail for ErrorNotFound {
  fn typ(&self) -> &'static str {
    "urn:white-rabbit:error:not-found"
  }

  fn title(&self) -> &'static str {
    "Entity Not Found"
  }

  fn status(&self) -> StatusCode {
    StatusCode::NOT_FOUND
  }

  fn detail(&self) -> String {
    format!(
      "Entity[{}, {}] not found",
      self.entity,
      self.values.iter().map(|(f, v)| format!("{} = {}", f, v)).join(", ")
    )
  }
}
