use std::collections::HashSet;

use sea_orm::{entity::prelude::*, ConnectionTrait};

use serde::{Deserialize, Serialize};

use super::{journal, AccountTag, IntoPresentation, Journal};

pub const TYPE: &str = "account";
pub const MULTIPLE: &str = "accounts";

#[derive(Clone, Debug, Eq, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "accounts")]
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
  pub strategy: Strategy,
  pub unit: String,
  #[sea_orm(default_value = false)]
  pub is_archived: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "i8", db_type = "TinyInteger")]
pub enum Type {
  Income = 0,
  Expense = 1,
  Asset = 2,
  Liability = 3,
  Equity = 4,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "i8", db_type = "TinyInteger")]
pub enum Strategy {
  Fifo = 0,
  Average = 1,
}

impl Related<Journal> for Entity {
  fn to() -> RelationDef {
    Relation::Journal.def()
  }
}

impl Related<AccountTag> for Entity {
  fn to() -> RelationDef {
    Relation::Tag.def()
  }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(has_many = "AccountTag")]
  Tag,
  #[sea_orm(belongs_to = "Journal", from = "Column::JournalId", to = "journal::Column::Id")]
  Journal,
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Presentation {
  pub id: uuid::Uuid,
  #[serde(rename = "journalId")]
  pub journal_id: uuid::Uuid,
  pub name: String,
  pub description: String,
  pub typ: Type,
  pub strategy: Strategy,
  pub unit: String,
  #[serde(rename = "isArchived")]
  pub is_archived: bool,
  pub tags: HashSet<String>,
}

#[async_trait::async_trait]
impl IntoPresentation for Model {
  type Presentation = Presentation;

  async fn into_presentation(self, conn: &impl ConnectionTrait) -> anyhow::Result<Self::Presentation> {
    let tags = self
      .find_related(AccountTag)
      .all(conn)
      .await?
      .into_iter()
      .map(|item| item.tag)
      .collect();

    let Model {
      id,
      journal_id,
      name,
      description,
      typ,
      strategy,
      unit,
      is_archived,
    } = self;
    Ok(Presentation {
      id,
      journal_id,
      name,
      description,
      typ,
      strategy,
      unit,
      is_archived,
      tags,
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
      strategy,
      unit,
      is_archived,
      ..
    }: Presentation,
  ) -> Self {
    Self {
      id,
      journal_id,
      name,
      description,
      typ,
      strategy,
      unit,
      is_archived,
    }
  }
}
