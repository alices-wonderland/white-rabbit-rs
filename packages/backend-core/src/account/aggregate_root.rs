use crate::account::{
  account_tag, ActiveModel, Column, Command, Entity, Model, Presentation, PrimaryKey, Query, Type,
};
use crate::journal::Journal;
use crate::user::User;
use crate::{AggregateRoot, Permission, Repository, Result};
use itertools::Itertools;
use sea_orm::entity::prelude::*;
use sea_orm::{IntoActiveModel, StreamTrait};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Account {
  pub id: Uuid,
  pub name: String,
  pub description: String,
  pub unit: String,
  pub typ: Type,
  pub tags: HashSet<String>,
  pub journal: Uuid,
  pub parent: Option<Uuid>,
}

impl Account {
  pub fn new(
    name: impl ToString,
    description: impl ToString,
    unit: impl ToString,
    typ: Type,
    tags: impl IntoIterator<Item = impl ToString>,
    journal: &Journal,
    parent: Option<&Self>,
  ) -> Self {
    Self {
      id: Uuid::new_v4(),
      name: name.to_string(),
      description: description.to_string(),
      typ,
      unit: unit.to_string(),
      tags: tags.into_iter().map(|tag| tag.to_string()).collect(),
      journal: journal.id(),
      parent: parent.map(|parent| parent.id()),
    }
  }
}

impl From<Account> for Model {
  fn from(Account { id, name, description, unit, typ, journal, parent, .. }: Account) -> Self {
    Self { id, name, description, unit, typ, journal_id: journal, parent_id: parent }
  }
}

impl From<Account> for HashSet<account_tag::Model> {
  fn from(value: Account) -> Self {
    value
      .tags
      .iter()
      .map(|tag| account_tag::Model { account_id: value.id, tag: tag.clone() })
      .collect()
  }
}

#[async_trait::async_trait]
impl AggregateRoot for Account {
  type Model = Model;
  type ActiveModel = ActiveModel;
  type Entity = Entity;
  type Presentation = Presentation;
  type PrimaryKey = PrimaryKey;
  type Query = Query;
  type Column = Column;
  type Command = Command;

  fn typ() -> &'static str {
    "Account"
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
      "parentId" => Some(self.parent.cmp(&other.parent)),
      "unit" => Some(self.unit.cmp(&other.unit)),
      _ => None,
    }
  }

  async fn from_models(db: &impl ConnectionTrait, models: Vec<Self::Model>) -> Result<Vec<Self>> {
    let mut results = Vec::new();
    let tags = models.load_many(account_tag::Entity, db).await?;
    for (account, tags) in models.into_iter().zip(tags.into_iter()) {
      results.push(Self {
        id: account.id,
        name: account.name,
        description: account.description,
        unit: account.unit,
        typ: account.typ,
        tags: tags.into_iter().map(|u| u.tag).collect::<HashSet<_>>(),
        journal: account.journal_id,
        parent: account.parent_id,
      });
    }

    Ok(results)
  }

  async fn do_save(db: &impl ConnectionTrait, roots: Vec<Self>) -> Result<()> {
    let accounts = roots
      .iter()
      .unique_by(|root| root.id)
      .map(|root| Model::from(root.clone()).into_active_model())
      .collect::<Vec<_>>();
    Entity::insert_many(accounts).exec(db).await?;

    let tags = roots
      .iter()
      .flat_map(|root| HashSet::<account_tag::Model>::from(root.clone()))
      .unique()
      .map(|model| model.into_active_model())
      .collect::<Vec<_>>();
    account_tag::Entity::insert_many(tags).exec(db).await?;

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
