use crate::journal::{
  journal_users, ActiveModel, Column, Command, Entity, Model, Presentation, PrimaryKey, Query,
};
use crate::user::{Role, User};
use crate::{AggregateRoot, Permission};

use itertools::Itertools;
use sea_orm::{ConnectionTrait, EntityTrait, IntoActiveModel, LoaderTrait};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Journal {
  pub id: Uuid,
  pub name: String,
  pub description: String,
  pub admins: HashSet<Uuid>,
  pub members: HashSet<Uuid>,
}

impl Journal {
  pub fn new(
    name: impl ToString,
    description: impl ToString,
    admins: &[User],
    members: &[User],
  ) -> Journal {
    Journal {
      id: Uuid::new_v4(),
      name: name.to_string(),
      description: description.to_string(),
      admins: admins.iter().map(|user| user.id()).collect::<HashSet<_>>(),
      members: members.iter().map(|user| user.id()).collect::<HashSet<_>>(),
    }
  }
}

impl From<Journal> for Model {
  fn from(Journal { id, name, description, .. }: Journal) -> Self {
    Self { id, name, description }
  }
}

impl From<Journal> for HashSet<journal_users::Model> {
  fn from(Journal { id, admins, members, .. }: Journal) -> Self {
    let mut result = HashMap::<Uuid, journal_users::Field>::default();
    for id in admins {
      result.insert(id, journal_users::Field::Admin);
    }
    for id in members {
      result.insert(id, journal_users::Field::Member);
    }

    result
      .into_iter()
      .map(|(user_id, field)| journal_users::Model { journal_id: id, user_id, field })
      .collect::<HashSet<_>>()
  }
}

#[async_trait::async_trait]
impl AggregateRoot for Journal {
  type Model = Model;
  type ActiveModel = ActiveModel;
  type Entity = Entity;
  type Presentation = Presentation;
  type PrimaryKey = PrimaryKey;
  type Query = Query;
  type Column = Column;
  type Command = Command;

  fn typ() -> &'static str {
    "Journal"
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
    let journal_users = models.load_many(journal_users::Entity, db).await?;
    for (journal, users) in models.into_iter().zip(journal_users.into_iter()) {
      results.push(Self {
        id: journal.id,
        name: journal.name,
        description: journal.description,
        admins: users
          .iter()
          .filter(|u| u.field == journal_users::Field::Admin)
          .map(|u| u.user_id)
          .collect::<HashSet<_>>(),
        members: users
          .iter()
          .filter(|u| u.field == journal_users::Field::Member)
          .map(|u| u.user_id)
          .collect::<HashSet<_>>(),
      });
    }

    Ok(results)
  }

  async fn do_save(db: &impl ConnectionTrait, roots: Vec<Self>) -> crate::Result<()> {
    let journals = roots
      .iter()
      .unique_by(|root| root.id)
      .map(|root| Model::from(root.clone()).into_active_model())
      .collect::<Vec<_>>();
    Entity::insert_many(journals).exec(db).await?;
    let journal_users = roots
      .iter()
      .flat_map(|root| HashSet::<journal_users::Model>::from(root.clone()))
      .unique()
      .map(|model| model.into_active_model())
      .collect::<Vec<_>>();
    journal_users::Entity::insert_many(journal_users).exec(db).await?;
    Ok(())
  }

  async fn handle(
    _db: &impl ConnectionTrait,
    _operator: Option<&User>,
    _command: Self::Command,
  ) -> crate::Result<Vec<Self>> {
    todo!()
  }

  async fn get_permission(
    _db: &impl ConnectionTrait,
    operator: Option<&User>,
    models: &[Self],
  ) -> crate::Result<HashMap<Uuid, Permission>> {
    Ok(if let Some(operator) = operator {
      models
        .iter()
        .filter_map(|model| {
          if operator.role == Role::Admin || model.admins.contains(&operator.id) {
            Some((model.id(), Permission::ReadWrite))
          } else if model.members.contains(&operator.id) {
            Some((model.id(), Permission::ReadOnly))
          } else {
            None
          }
        })
        .collect::<HashMap<_, _>>()
    } else {
      models.iter().map(|model| (model.id(), Permission::ReadWrite)).collect::<HashMap<_, _>>()
    })
  }
}
