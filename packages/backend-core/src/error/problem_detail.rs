use http::StatusCode;
use serde::{Deserialize, Serialize};

pub trait ProblemDetail: Serialize + for<'a> Deserialize<'a> + Sized + Sync {
  fn typ() -> &'static str;

  fn title() -> &'static str;

  fn status() -> StatusCode;

  fn detail(&self) -> String;
}

mod status_serde {
  use http::StatusCode;
  use serde::{Deserialize, Deserializer, Serializer};

  pub fn serialize<S>(value: &StatusCode, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_u16(value.as_u16())
  }

  // The signature of a deserialize_with function must follow the pattern:
  //
  //    fn deserialize<'de, D>(D) -> Result<T, D::Error>
  //    where
  //        D: Deserializer<'de>
  //
  // although it may also be generic over the output types T.
  pub fn deserialize<'de, D>(deserializer: D) -> Result<StatusCode, D::Error>
  where
    D: Deserializer<'de>,
  {
    let s = u16::deserialize(deserializer)?;
    let dt = StatusCode::from_u16(s).map_err(serde::de::Error::custom)?;
    Ok(dt)
  }
}
