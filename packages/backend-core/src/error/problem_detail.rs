use crate::error::ErrorNotFound;
use crate::Error;
use http::StatusCode;
use serde::de::{Error as _, Unexpected};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub trait ProblemDetail: Serialize + for<'a> Deserialize<'a> + Sized + Sync {
  fn typ(&self) -> &'static str;

  fn title(&self) -> &'static str;

  fn status(&self) -> StatusCode;

  fn detail(&self) -> String;
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct ProblemDetailDef {
  #[serde(rename = "type")]
  pub typ: String,
  pub title: String,
  pub status: u16,
  pub detail: String,
  #[serde(flatten)]
  pub extra: Value,
}

impl From<ProblemDetailDef> for Option<crate::Error> {
  fn from(value: ProblemDetailDef) -> Self {
    if value.typ == "urn:white-rabbit:error:not-found" {
      if let Ok(err) = serde_json::from_value::<ErrorNotFound>(value.extra) {
        return Some(crate::Error::NotFound(err));
      }
    }

    None
  }
}

impl TryFrom<crate::Error> for ProblemDetailDef {
  type Error = serde_json::Error;

  fn try_from(value: crate::Error) -> Result<Self, Self::Error> {
    match value {
      Error::NotFound(err) => Ok(ProblemDetailDef {
        typ: err.typ().to_string(),
        title: err.title().to_string(),
        status: err.status().as_u16(),
        detail: err.detail(),
        extra: serde_json::to_value(err)?,
      }),
      _ => Err(serde_json::error::Error::invalid_type(
        Unexpected::StructVariant,
        &"Error structs with ProblemDetail trait",
      )),
    }
  }
}

#[cfg(test)]
mod test {
  use crate::entity::{journal, FIELD_ID, FIELD_NAME};
  use crate::error::{ErrorNotFound, ProblemDetailDef};

  #[test]
  fn test_serde() -> anyhow::Result<()> {
    let err = crate::Error::NotFound(ErrorNotFound {
      entity: journal::TYPE.to_string(),
      values: vec![
        (FIELD_ID.to_string(), "ID1".to_string()),
        (FIELD_NAME.to_string(), "Journal 1".to_string()),
      ],
    });

    let prob: ProblemDetailDef = err.try_into()?;
    let serded = serde_json::to_string_pretty(&prob)?;
    println!("Serded: {}", serded);

    let deserded: ProblemDetailDef = serde_json::from_str(&serded)?;
    assert_eq!(prob, deserded);
    println!("Desered: {:#?}", deserded);

    let deserded_err: Option<crate::Error> = deserded.into();
    println!("Desered Err: {:#?}", deserded_err);

    assert_eq!(
      crate::Error::NotFound(ErrorNotFound {
        entity: journal::TYPE.to_string(),
        values: vec![
          (FIELD_ID.to_string(), "ID1".to_string()),
          (FIELD_NAME.to_string(), "Journal 1".to_string()),
        ],
      }),
      deserded_err.unwrap()
    );

    Ok(())
  }
}
