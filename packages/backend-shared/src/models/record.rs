use std::collections::{HashMap, HashSet};

use chrono::NaiveDate;
use sea_orm::{entity::prelude::*, ConnectionTrait};

use serde::{Deserialize, Serialize};

use crate::services::RecordService;

use super::{record_item, IntoPresentation, Journal, RecordItem, RecordTag};

pub const TYPE: &str = "record";
pub const MULTIPLE: &str = "records";

#[derive(Clone, Debug, Eq, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "records")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: uuid::Uuid,
  #[sea_orm(indexed)]
  pub journal_id: uuid::Uuid,
  #[sea_orm(unique, indexed)]
  pub name: String,
  pub description: String,
  #[sea_orm(column_name = "type")]
  pub typ: Type,
  pub date: NaiveDate,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "i8", db_type = "TinyInteger")]
pub enum Type {
  Record = 0,
  Check = 1,
}

impl Related<Journal> for Entity {
  fn to() -> RelationDef {
    Relation::Journal.def()
  }
}

impl Related<RecordTag> for Entity {
  fn to() -> RelationDef {
    Relation::Tag.def()
  }
}

impl Related<RecordItem> for Entity {
  fn to() -> RelationDef {
    Relation::Item.def()
  }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(has_many = "super::RecordTag")]
  Tag,
  #[sea_orm(
    belongs_to = "super::Journal",
    from = "Column::JournalId",
    to = "super::journal::Column::Id"
  )]
  Journal,
  #[sea_orm(has_many = "super::RecordItem")]
  Item,
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RecordState {
  Record(RecordStateItem),
  Check(HashMap<uuid::Uuid, RecordStateItem>),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RecordStateItem {
  Valid(Decimal),
  Invalid(Decimal, Decimal),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Presentation {
  pub id: uuid::Uuid,
  #[serde(rename = "journalId")]
  pub journal_id: uuid::Uuid,
  pub name: String,
  pub description: String,
  #[serde(rename = "type")]
  pub typ: Type,
  pub date: NaiveDate,
  pub tags: HashSet<String>,
  pub items: HashSet<record_item::Presentation>,
  pub state: RecordState,
}

#[async_trait::async_trait]
impl IntoPresentation for Model {
  type Presentation = Presentation;

  async fn into_presentation(self, conn: &impl ConnectionTrait) -> crate::Result<Self::Presentation> {
    let state = RecordService::state(conn, &self).await?;
    let tags = self
      .find_related(RecordTag)
      .all(conn)
      .await?
      .into_iter()
      .map(|item| item.tag)
      .collect();
    let items = self
      .find_related(RecordItem)
      .all(conn)
      .await?
      .into_iter()
      .map(|item| item.into())
      .collect();

    let Model {
      id,
      journal_id,
      name,
      description,
      typ,
      date,
    } = self;
    Ok(Presentation {
      id,
      journal_id,
      name,
      description,
      typ,
      date,
      tags,
      items,
      state,
    })
  }
}

impl From<Presentation> for Model {
  fn from(
    Presentation {
      id,
      journal_id,
      name,
      description,
      typ,
      date,
      ..
    }: Presentation,
  ) -> Self {
    Self {
      id,
      journal_id,
      name,
      description,
      typ,
      date,
    }
  }
}
