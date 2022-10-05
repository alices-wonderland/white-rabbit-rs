use std::collections::HashSet;

use serde::Serialize;

use crate::{
  models::AccessItemType,
  services::{Permission, FIELD_TAG, FIELD_TAG_ITEM, MAX_TAG, MAX_TAG_ITEM},
};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Clone, Debug, Serialize)]
pub enum Error {
  #[error("User[{user}] does not have Permission[{permission}] on Entity[{entity}, id={id:?}]")]
  InvalidPermission {
    user: String,
    entity: String,
    id: Option<uuid::Uuid>,
    permission: Permission,
  },
  #[error("The length of Field[{field}] in Entity[{entity}] should be less than {value}")]
  MaxLength {
    entity: String,
    field: String,
    value: usize,
  },
  #[error("The length of Field[{field}] in Entity[{entity}] should between {min} and {max}")]
  LengthRange {
    entity: String,
    field: String,
    min: usize,
    max: usize,
  },
  #[error("Entity[{entity}] with Field[{field}, value={value}] already exists")]
  AlreadyExists {
    entity: String,
    field: String,
    value: String,
  },
  #[error("Entity[{entity}] with Field[{field}, value={value}] is not found")]
  NotFound {
    entity: String,
    field: String,
    value: String,
  },
  #[error("AccessItem[type={access_item_type}, id={access_item_id}] of Entity[{entity}, id={id:?}] cannot exist both in admins and members")]
  DuplicatedAccessItem {
    entity: String,
    id: uuid::Uuid,
    access_item_type: AccessItemType,
    access_item_id: uuid::Uuid,
  },
  #[error("Record[{id}] should contain at most 1 empty item")]
  RecordAtMostOneEmptyItem { id: uuid::Uuid },
  #[error("Item with Account[{account}] in Record[{id}] must contain price")]
  RecordItemMustContainPrice { id: uuid::Uuid, account: uuid::Uuid },
  #[error("Item with Account[{account}] in Record[{id}] forbids the price")]
  RecordItemForbidPrice { id: uuid::Uuid, account: uuid::Uuid },
  #[error("Item with Account[{account}] in Record[{id}] cannot only exist price without amount")]
  RecordItemOnlyPriceExist { id: uuid::Uuid, account: uuid::Uuid },
  #[error("Account[{account}] exists multiple times in Record[{id}]")]
  DuplicateAccountsInRecord { id: uuid::Uuid, account: uuid::Uuid },
  #[error("Account[{id}] is archived")]
  ArchivedAccount { id: uuid::Uuid },
  #[error("Multiple errors found")]
  Errors(Vec<Error>),

  #[error("Database Error: {}", 0)]
  Database(
    #[from]
    #[serde(skip_serializing)]
    sea_orm::DbErr,
  ),
  #[error("Base64 Error: {}", 0)]
  Base64(
    #[from]
    #[serde(skip_serializing)]
    base64::DecodeError,
  ),
  #[error("UTF8 Error: {}", 0)]
  Utf8(
    #[from]
    #[serde(skip_serializing)]
    std::string::FromUtf8Error,
  ),
  #[error("UUID Error: {}", 0)]
  Uuid(
    #[from]
    #[serde(skip_serializing)]
    uuid::Error,
  ),
}

impl Error {
  pub fn validate_tags(entity: &str, tags: &HashSet<String>) -> Vec<Error> {
    let mut errors = Vec::new();

    if tags.len() > MAX_TAG {
      errors.push(Error::MaxLength {
        entity: entity.to_owned(),
        field: FIELD_TAG.to_owned(),
        value: MAX_TAG,
      });
    }

    for tag in tags.iter().take(MAX_TAG) {
      if tag.len() > MAX_TAG_ITEM {
        errors.push(Error::MaxLength {
          entity: entity.to_owned(),
          field: FIELD_TAG_ITEM.to_owned(),
          value: MAX_TAG_ITEM,
        });
      }
    }

    errors
  }
}
