use crate::account::Account;
use crate::journal::Journal;
use crate::record::{
  record_item, record_tag, ActiveModel, Column, Command, CommandBatchUpdate, CommandCreate,
  CommandDelete, CommandUpdate, Entity, Model, Presentation, PrimaryKey, Query, Type,
};
use crate::user::User;
use crate::{
  account, journal, utils, AggregateRoot, Error, FindAllArgs, Permission, Repository, Result,
  FIELD_ID,
};
use chrono::NaiveDate;
use itertools::Itertools;
use rust_decimal::Decimal;
use sea_orm::sea_query::{Expr, OnConflict};
use sea_orm::{
  ColumnTrait, ConnectionTrait, EntityTrait, IntoActiveModel, LoaderTrait, QueryFilter, StreamTrait,
};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

pub const FIELD_ITEMS: &str = "items";
pub const MIN_ITEMS: usize = 2;
pub const MAX_ITEMS: usize = 8;

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
  #![allow(clippy::too_many_arguments)]
  pub fn new(
    id: Uuid,
    journal: &Journal,
    name: impl ToString,
    description: impl ToString,
    typ: Type,
    date: NaiveDate,
    tags: impl IntoIterator<Item = impl ToString>,
    items: impl IntoIterator<Item = RecordItem>,
  ) -> Record {
    Self {
      id,
      journal: journal.id(),
      name: name.to_string().trim().to_string(),
      description: description.to_string().trim().to_string(),
      typ,
      date,
      tags: tags
        .into_iter()
        .map(|tag| tag.to_string().trim().to_string())
        .filter(|tag| !tag.is_empty())
        .collect(),
      items: HashSet::from_iter(items),
    }
  }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecordItem {
  pub account: Uuid,
  #[serde(with = "rust_decimal::serde::arbitrary_precision")]
  pub amount: Decimal,
  #[serde(default, with = "rust_decimal::serde::arbitrary_precision_option")]
  pub price: Option<Decimal>,
}

impl RecordItem {
  pub fn new(
    journal: &Journal,
    account: &Account,
    amount: Decimal,
    price: Option<Decimal>,
  ) -> RecordItem {
    Self {
      account: account.id(),
      amount,
      price: price.and_then(|price| if journal.unit == account.unit { None } else { Some(price) }),
    }
  }

  pub fn of(
    journal: &Journal,
    accounts: &HashMap<Uuid, Account>,
    items: impl IntoIterator<Item = RecordItem>,
  ) -> HashSet<RecordItem> {
    items
      .into_iter()
      .map(|item: RecordItem| {
        RecordItem::new(journal, accounts.get(&item.account).unwrap(), item.amount, item.price)
      })
      .collect()
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

  fn sortable_column(field: impl ToString) -> Option<Self::Column> {
    match field.to_string().as_str() {
      "journal" => Some(Column::JournalId),
      "name" => Some(Column::Name),
      "type" => Some(Column::Typ),
      "date" => Some(Column::Date),
      _ => None,
    }
  }

  fn compare_by_field(&self, other: &Self, field: impl ToString) -> Option<Ordering> {
    match field.to_string().as_str() {
      "id" => Some(self.id.cmp(&other.id)),
      "journal" => Some(self.journal.cmp(&other.journal)),
      "name" => Some(self.name.cmp(&other.name)),
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
        tags: tags.into_iter().map(|u| u.tag).collect(),
        items: items
          .into_iter()
          .map(|u| RecordItem { account: u.account_id, amount: u.amount, price: u.price })
          .collect(),
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
    let mut on_conflict = OnConflict::column(Self::primary_column());
    on_conflict.update_columns([
      Column::JournalId,
      Column::Name,
      Column::Description,
      Column::Typ,
      Column::Date,
    ]);
    Entity::insert_many(records).on_conflict(on_conflict).exec(db).await?;

    let record_ids = utils::get_ids(&roots);

    record_tag::Entity::delete_many()
      .filter(record_tag::Column::RecordId.is_in(record_ids.clone()))
      .exec(db)
      .await?;
    let record_tags = roots
      .iter()
      .flat_map(|root| HashSet::<record_tag::Model>::from(root.clone()))
      .unique()
      .map(|model| model.into_active_model())
      .collect::<Vec<_>>();
    record_tag::Entity::insert_many(record_tags).exec(db).await?;

    record_item::Entity::delete_many()
      .filter(record_item::Column::RecordId.is_in(record_ids.clone()))
      .exec(db)
      .await?;
    let record_items = roots
      .iter()
      .flat_map(|root| HashSet::<record_item::Model>::from(root.clone()))
      .unique()
      .map(|model| model.into_active_model())
      .collect::<Vec<_>>();
    record_item::Entity::insert_many(record_items).exec(db).await?;

    Ok(())
  }

  async fn do_delete(db: &impl ConnectionTrait, roots: Vec<Self>) -> Result<()> {
    let ids = utils::get_ids(&roots);

    let _ = record_tag::Entity::delete_many()
      .filter(Expr::col(record_tag::Column::RecordId).is_in(ids.clone()))
      .exec(db)
      .await?;

    let _ = record_item::Entity::delete_many()
      .filter(Expr::col(record_item::Column::RecordId).is_in(ids.clone()))
      .exec(db)
      .await?;

    let _ = Self::Entity::delete_many()
      .filter(Expr::col(Self::primary_column()).is_in(ids.clone()))
      .exec(db)
      .await?;
    Ok(())
  }

  async fn handle(
    db: &(impl ConnectionTrait + StreamTrait),
    operator: Option<&User>,
    command: Self::Command,
  ) -> Result<Vec<Self>> {
    match command {
      Command::Create(command) => {
        Record::batch_update(
          db,
          operator,
          CommandBatchUpdate { create: vec![command], ..Default::default() },
        )
        .await
      }
      Command::Update(command) => {
        Record::batch_update(
          db,
          operator,
          CommandBatchUpdate { update: vec![command], ..Default::default() },
        )
        .await
      }
      Command::BatchUpdate(command) => Record::batch_update(db, operator, command).await,
      Command::Delete(CommandDelete { id }) => {
        Record::delete(db, operator, id).await?;
        Ok(Vec::default())
      }
    }
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

impl Record {
  fn validate_items(
    record_id: Uuid,
    items: &HashSet<RecordItem>,
    journal: &Journal,
    accounts: &HashMap<Uuid, Account>,
  ) -> Result<()> {
    let mut account_ids = HashSet::new();

    for item in items {
      if let Some(account) = accounts.get(&item.account) {
        if account.journal != journal.id() {
          return Err(Error::not_related_entity::<Account, Journal>(
            vec![(FIELD_ID, account.id)],
            vec![(FIELD_ID, journal.id)],
          ));
        } else if account_ids.contains(&account.id) {
          return Err(Error::RecordItemAccountAlreadyExist { id: record_id, account: account.id });
        } else {
          account_ids.insert(account.id);
        }
      } else {
        return Err(Error::not_found::<Account>([(FIELD_ID, item.account)]));
      }
    }

    if items.len() < MIN_ITEMS || items.len() > MAX_ITEMS {
      return Err(Error::NotInRange {
        field: FIELD_ITEMS.to_string(),
        begin: MIN_ITEMS,
        end: MAX_ITEMS,
      });
    }

    Ok(())
  }

  fn do_create(
    command: CommandCreate,
    journal: &Journal,
    accounts: &HashMap<Uuid, Account>,
  ) -> Result<Record> {
    let id = command.id.and_then(|s| Uuid::try_parse(&s).ok()).unwrap_or_else(Uuid::new_v4);

    Self::validate_items(id, &command.items, journal, accounts)?;

    Ok(Record::new(
      id,
      journal,
      command.name,
      command.description,
      command.typ,
      command.date,
      command.tags,
      RecordItem::of(journal, accounts, command.items),
    ))
  }

  fn do_update(
    command: CommandUpdate,
    record: Record,
    journal: &Journal,
    accounts: &HashMap<Uuid, Account>,
  ) -> Result<Record> {
    if let Some(items) = &command.items {
      Self::validate_items(command.id, items, journal, accounts)?;
    }

    Ok(Record::new(
      record.id,
      journal,
      command.name.unwrap_or(record.name),
      command.description.unwrap_or(record.description),
      command.typ.unwrap_or(record.typ),
      command.date.unwrap_or(record.date),
      command.tags.unwrap_or(record.tags),
      command.items.map(|items| RecordItem::of(journal, accounts, items)).unwrap_or(record.items),
    ))
  }

  async fn batch_update(
    db: &(impl ConnectionTrait + StreamTrait),
    operator: Option<&User>,
    command: CommandBatchUpdate,
  ) -> Result<Vec<Record>> {
    let mut result = Vec::new();

    let record_ids: HashSet<Uuid> = command.update.iter().map(|r| r.id).collect();
    let records = utils::into_map(
      Repository::<Record>::find_all(
        db,
        operator,
        FindAllArgs { query: Query::from(record_ids), ..Default::default() },
      )
      .await?,
    );
    let mut account_ids = HashSet::<Uuid>::new();
    let mut journal_ids = HashSet::<Uuid>::new();
    for create in &command.create {
      journal_ids.insert(create.journal);
      for item in &create.items {
        account_ids.insert(item.account);
      }
    }
    for update in &command.update {
      if let Some(items) = &update.items {
        for item in items {
          account_ids.insert(item.account);
        }
      }
    }
    let accounts = utils::into_map(
      Repository::<Account>::find_all(
        db,
        operator,
        FindAllArgs { query: account::Query::from(account_ids), ..Default::default() },
      )
      .await?,
    );

    for record in records.values() {
      journal_ids.insert(record.journal);
    }

    let journals = Repository::<Journal>::find_all(
      db,
      operator,
      FindAllArgs { query: journal::Query::from(journal_ids), ..Default::default() },
    )
    .await?;
    Journal::check_writeable(db, operator, &journals).await?;

    let journals = utils::into_map(journals);

    for command in command.update {
      if let Some(record) = records.get(&command.id) {
        if let Some(journal) = journals.get(&record.journal) {
          result.push(Self::do_update(command, record.clone(), journal, &accounts)?);
        } else {
          return Err(Error::not_found::<Journal>([(FIELD_ID, record.journal)]));
        }
      } else {
        return Err(Error::not_found::<Record>([(FIELD_ID, command.id)]));
      }
    }

    Self::check_writeable(db, operator, &result).await?;

    for command in command.create {
      if let Some(journal) = journals.get(&command.journal) {
        result.push(Self::do_create(command, journal, &accounts)?);
      } else {
        return Err(Error::not_found::<Account>([(FIELD_ID, command.journal)]));
      }
    }

    Repository::<Self>::save(db, result).await
  }

  async fn delete(
    db: &(impl ConnectionTrait + StreamTrait),
    operator: Option<&User>,
    ids: HashSet<Uuid>,
  ) -> Result<()> {
    let models = Repository::find_by_ids(db, ids).await?;
    Self::check_writeable(db, operator, &models).await?;
    Repository::delete(db, models).await?;
    Ok(())
  }
}
