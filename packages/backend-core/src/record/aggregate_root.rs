use crate::account::Account;
use crate::journal::Journal;
use crate::record::{
  record_item, record_tag, ActiveModel, Column, Command, Entity, Model, Presentation, PrimaryKey,
  Query, Type,
};
use crate::user::User;
use crate::{AggregateRoot, Permission, Repository, Result};
use chrono::NaiveDate;
use itertools::Itertools;
use rust_decimal::Decimal;
use sea_orm::{ConnectionTrait, EntityTrait, IntoActiveModel, LoaderTrait, StreamTrait};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Record {
  pub id: Uuid,
  pub journal: Uuid,
  pub name: String,
  pub description: String,
  pub typ: Type,
  pub date: NaiveDate,
  pub tags: HashSet<String>,
  pub items: HashSet<RecordItem>,
}

impl Record {
  pub fn new(
    journal: &Journal,
    name: impl ToString,
    description: impl ToString,
    typ: Type,
    date: NaiveDate,
    tags: impl IntoIterator<Item = impl ToString>,
    items: impl IntoIterator<Item = RecordItem>,
  ) -> Record {
    Self {
      id: Uuid::new_v4(),
      journal: journal.id(),
      name: name.to_string(),
      description: description.to_string(),
      typ,
      date,
      tags: tags.into_iter().map(|tag| tag.to_string()).collect(),
      items: HashSet::from_iter(items),
    }
  }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecordItem {
  pub account: Uuid,
  pub amount: Decimal,
  pub price: Option<Decimal>,
}

impl RecordItem {
  pub fn new(account: &Account, amount: Decimal, price: Option<Decimal>) -> RecordItem {
    Self { account: account.id(), amount, price }
  }
}

impl From<Record> for Model {
  fn from(Record { id, journal, name, description, typ, date, .. }: Record) -> Self {
    Model { id, journal_id: journal, name, description, typ, date }
  }
}

impl From<Record> for HashSet<record_tag::Model> {
  fn from(Record { id, tags, .. }: Record) -> Self {
    tags.into_iter().map(|tag| record_tag::Model { record_id: id, tag }).collect()
  }
}

impl From<Record> for HashSet<record_item::Model> {
  fn from(Record { id, items, .. }: Record) -> Self {
    items
      .into_iter()
      .map(|RecordItem { account, amount, price }| record_item::Model {
        record_id: id,
        account_id: account,
        amount,
        price,
      })
      .collect()
  }
}

#[async_trait::async_trait]
impl AggregateRoot for Record {
  type Model = Model;
  type ActiveModel = ActiveModel;
  type Entity = Entity;
  type Presentation = Presentation;
  type PrimaryKey = PrimaryKey;
  type Query = Query;
  type Column = Column;
  type Command = Command;

  fn typ() -> &'static str {
    "Record"
  }

  fn id(&self) -> Uuid {
    self.id
  }

  fn primary_column() -> Self::Column {
    Column::Id
  }

  fn compare_by_field(&self, other: &Self, field: impl ToString) -> Option<Ordering> {
    match field.to_string().as_str() {
      "id" => Some(self.id.cmp(&other.id)),
      "name" => Some(self.name.cmp(&other.name)),
      "journalId" => Some(self.journal.cmp(&other.journal)),
      "type" => Some(self.typ.cmp(&other.typ)),
      "date" => Some(self.date.cmp(&other.date)),
      _ => None,
    }
  }

  async fn from_models(db: &impl ConnectionTrait, models: Vec<Self::Model>) -> Result<Vec<Self>> {
    let mut results = Vec::new();
    let tags = models.load_many(record_tag::Entity, db).await?;
    let items = models.load_many(record_item::Entity, db).await?;
    for ((record, tags), items) in models.into_iter().zip(tags.into_iter()).zip(items) {
      results.push(Self {
        id: record.id,
        journal: record.journal_id,
        name: record.name,
        description: record.description,
        typ: record.typ,
        date: record.date,
        tags: tags.into_iter().map(|u| u.tag).collect::<HashSet<_>>(),
        items: items
          .into_iter()
          .map(|u| RecordItem { account: u.account_id, amount: u.amount, price: u.price })
          .collect::<HashSet<_>>(),
      });
    }

    Ok(results)
  }

  async fn do_save(db: &impl ConnectionTrait, roots: Vec<Self>) -> Result<()> {
    let records = roots
      .iter()
      .unique_by(|root| root.id)
      .map(|root| Model::from(root.clone()).into_active_model())
      .collect::<Vec<_>>();
    Entity::insert_many(records).exec(db).await?;
    let record_tags = roots
      .iter()
      .flat_map(|root| HashSet::<record_tag::Model>::from(root.clone()))
      .unique()
      .map(|model| model.into_active_model())
      .collect::<Vec<_>>();
    record_tag::Entity::insert_many(record_tags).exec(db).await?;

    let record_items = roots
      .iter()
      .flat_map(|root| HashSet::<record_item::Model>::from(root.clone()))
      .unique()
      .map(|model| model.into_active_model())
      .collect::<Vec<_>>();
    record_item::Entity::insert_many(record_items).exec(db).await?;

    Ok(())
  }

  async fn handle(
    _db: &(impl ConnectionTrait + StreamTrait),
    _operator: Option<&User>,
    _command: Self::Command,
  ) -> Result<Vec<Self>> {
    todo!()
  }

  async fn get_permission(
    db: &impl ConnectionTrait,
    operator: Option<&User>,
    models: &[Self],
  ) -> Result<HashMap<Uuid, Permission>> {
    let ids = models.iter().map(|model| model.journal).collect::<HashSet<_>>();
    let journals = Repository::<Journal>::find_by_ids(db, ids).await?;
    let permissions = Journal::get_permission(db, operator, &journals).await?;
    Ok(
      models
        .iter()
        .filter_map(|model| {
          permissions.get(&model.journal).map(|permission| (model.id(), *permission))
        })
        .collect(),
    )
  }
}
