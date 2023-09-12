use itertools::Itertools;

#[derive(thiserror::Error, Debug)]
pub enum Error {
  #[error("Entity[{typ}, {}] is existing", .values.iter().map(|(f, v)| format!("{} = {}", f, v)).join(", "))]
  ExistingEntity { typ: String, values: Vec<(String, String)> },

  #[error("Field[{field}] of Entity[{typ}] should in Range[start = {start:?}, end = {end:?}]")]
  OutOfRange { typ: String, field: String, start: Option<String>, end: Option<String> },
}

pub type Result<T> = std::result::Result<T, Error>;
