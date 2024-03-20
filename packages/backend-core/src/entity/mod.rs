use sea_orm::ConnectionTrait;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

pub mod account;
pub mod account_tag;
pub mod entry;
pub mod entry_item;
pub mod entry_tag;
pub mod hierarchy_report;
pub mod journal;
pub mod journal_tag;

pub const FIELD_ID: &str = "id";
pub const FIELD_NAME: &str = "name";
pub const FIELD_DESCRIPTION: &str = "description";
pub const FIELD_TAGS: &str = "tags";
pub const FIELD_TAG_EACH: &str = "tags.each";
pub const FIELD_UNIT: &str = "unit";
pub const FIELD_JOURNAL: &str = "journal";
pub const FIELD_TYPE: &str = "type";

pub const MIN_NAME_LENGTH: usize = 6;
pub const MAX_NAME_LENGTH: usize = 63;
pub const MAX_DESCRIPTION_LENGTH: usize = 1023;
pub const MIN_SHORT_TEXT_LENGTH: usize = 2;
pub const MAX_SHORT_TEXT_LENGTH: usize = 15;
pub const MAX_TAGS_LENGTH: usize = 7;

pub trait ReadRoot: Sized {
  type Query;

  type Sort;

  fn id(&self) -> String;

  async fn find_all(
    db: &impl ConnectionTrait,
    query: Option<Self::Query>,
    limit: Option<u64>,
    sort: Option<Self::Sort>,
  ) -> crate::Result<Vec<Self>>;

  async fn find_one(
    db: &impl ConnectionTrait,
    query: Option<Self::Query>,
  ) -> crate::Result<Option<Self>> {
    Ok(Self::find_all(db, query, Some(1), None).await?.into_iter().next())
  }
}

pub trait WriteRoot: ReadRoot {
  type Model;

  async fn from_model(
    db: &impl ConnectionTrait,
    models: impl IntoIterator<Item = Self::Model>,
  ) -> crate::Result<Vec<Self>>;

  async fn save(
    db: &impl ConnectionTrait,
    roots: impl IntoIterator<Item = Self>,
  ) -> crate::Result<Vec<Self>>;

  async fn delete(
    db: &impl ConnectionTrait,
    ids: impl IntoIterator<Item = Uuid>,
  ) -> crate::Result<()>;
}

pub trait Presentation: Sized + Serialize + for<'a> Deserialize<'a> {
  type R: ReadRoot;

  async fn from_roots(db: &impl ConnectionTrait, roots: Vec<Self::R>) -> crate::Result<Vec<Self>>;
}

pub(crate) fn normalize_name(typ: impl ToString, value: impl ToString) -> crate::Result<String> {
  let value = value.to_string().trim().to_string();
  if value.len() < MIN_NAME_LENGTH || value.len() > MAX_NAME_LENGTH {
    Err(crate::Error::OutOfRange {
      typ: typ.to_string(),
      field: FIELD_NAME.to_string(),
      start: Some(MIN_NAME_LENGTH.to_string()),
      end: Some(MAX_NAME_LENGTH.to_string()),
    })
  } else {
    Ok(value)
  }
}

pub(crate) fn normalize_description(
  typ: impl ToString,
  value: impl ToString,
) -> crate::Result<String> {
  let value = value.to_string().trim().to_string();
  if value.len() > MAX_DESCRIPTION_LENGTH {
    Err(crate::Error::OutOfRange {
      typ: typ.to_string(),
      field: FIELD_DESCRIPTION.to_string(),
      start: None,
      end: Some(MAX_NAME_LENGTH.to_string()),
    })
  } else {
    Ok(value)
  }
}

pub(crate) fn normalize_unit(typ: impl ToString, value: impl ToString) -> crate::Result<String> {
  let value = value.to_string().trim().to_string();
  if value.len() < MIN_SHORT_TEXT_LENGTH || value.len() > MAX_SHORT_TEXT_LENGTH {
    Err(crate::Error::OutOfRange {
      typ: typ.to_string(),
      field: FIELD_UNIT.to_string(),
      start: Some(MIN_SHORT_TEXT_LENGTH.to_string()),
      end: Some(MAX_SHORT_TEXT_LENGTH.to_string()),
    })
  } else {
    Ok(value)
  }
}

pub(crate) fn normalize_tags(
  typ: impl ToString,
  values: impl IntoIterator<Item = impl ToString>,
) -> crate::Result<HashSet<String>> {
  let mut trimmed = HashSet::new();

  for value in values {
    let value = value.to_string().trim().to_string();
    if value.len() < MIN_SHORT_TEXT_LENGTH || value.len() > MAX_SHORT_TEXT_LENGTH {
      return Err(crate::Error::OutOfRange {
        typ: typ.to_string(),
        field: FIELD_TAG_EACH.to_string(),
        start: Some(MIN_SHORT_TEXT_LENGTH.to_string()),
        end: Some(MAX_SHORT_TEXT_LENGTH.to_string()),
      });
    }

    trimmed.insert(value);
  }

  if trimmed.len() > MAX_TAGS_LENGTH {
    Err(crate::Error::OutOfRange {
      typ: typ.to_string(),
      field: FIELD_TAGS.to_string(),
      start: None,
      end: Some(MAX_TAGS_LENGTH.to_string()),
    })
  } else {
    Ok(trimmed)
  }
}
