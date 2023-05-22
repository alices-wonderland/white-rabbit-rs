use crate::account::{
  account_tags, ActiveModel, Column, Command, Entity, Model, Presentation, PrimaryKey, Query,
};
use crate::journal::Journal;
use crate::user::User;
use crate::{AggregateRoot, Permission, Repository};
use itertools::Itertools;
use sea_orm::{ConnectionTrait, EntityTrait, IntoActiveModel, LoaderTrait, StreamTrait};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Account {
  pub id: Uuid,
  pub name: String,
  pub description: String,
  pub tags: HashSet<String>,
  pub journal: Uuid,
  pub parent: Option<Uuid>,
}

impl Account {
  pub fn new(
    name: impl ToString,
    description: impl ToString,
    tags: impl IntoIterator<Item = impl ToString>,
    journal: &Journal,
    parent: Option<&Self>,
  ) -> Self {
    Self {
      id: Uuid::new_v4(),
      name: name.to_string(),
      description: description.to_string(),
      tags: tags.into_iter().map(|tag| tag.to_string()).collect::<HashSet<_>>(),
      journal: journal.id(),
      parent: parent.map(|parent| parent.id()),
    }
  }
}

impl From<Account> for Model {
  fn from(Account { id, name, description, journal, parent, .. }: Account) -> Self {
    Self { id, name, description, journal_id: journal, parent_id: parent }
  }
}

impl From<Account> for HashSet<account_tags::Model> {
  fn from(value: Account) -> Self {
    value
      .tags
      .iter()
      .map(|tag| account_tags::Model { account_id: value.id, tag: tag.clone() })
      .collect::<HashSet<_>>()
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

  async fn from_models(
    db: &impl ConnectionTrait,
    models: Vec<Self::Model>,
  ) -> crate::Result<Vec<Self>> {
    let mut results = Vec::new();
    let tags = models.load_many(account_tags::Entity, db).await?;
    for (account, tags) in models.into_iter().zip(tags.into_iter()) {
      results.push(Self {
        id: account.id,
        name: account.name,
        description: account.description,
        tags: tags.into_iter().map(|u| u.tag).collect::<HashSet<_>>(),
        journal: account.journal_id,
        parent: account.parent_id,
      });
    }

    Ok(results)
  }

  async fn do_save(db: &impl ConnectionTrait, roots: Vec<Self>) -> crate::Result<()> {
    let accounts = roots
      .iter()
      .unique_by(|root| root.id)
      .map(|root| Model::from(root.clone()).into_active_model())
      .collect::<Vec<_>>();
    Entity::insert_many(accounts).exec(db).await?;
    let tags = roots
      .iter()
      .flat_map(|root| HashSet::<account_tags::Model>::from(root.clone()))
      .unique()
      .map(|model| model.into_active_model())
      .collect::<Vec<_>>();
    account_tags::Entity::insert_many(tags).exec(db).await?;
    Ok(())
  }

  async fn handle(
    _db: &(impl ConnectionTrait + StreamTrait),
    _operator: Option<&User>,
    _command: Self::Command,
  ) -> crate::Result<Vec<Self>> {
    todo!()
  }

  async fn get_permission(
    db: &impl ConnectionTrait,
    operator: Option<&User>,
    models: &[Self],
  ) -> crate::Result<HashMap<Uuid, Permission>> {
    let ids = models.iter().map(|model| model.journal).collect::<HashSet<_>>();
    let journals = Repository::<Journal>::find_by_ids(db, ids).await?;
    let permissions = Journal::get_permission(db, operator, &journals).await?;
    Ok(
      models
        .iter()
        .filter_map(|model| {
          permissions.get(&model.journal).map(|permission| (model.id(), permission.clone()))
        })
        .collect(),
    )
  }
}
