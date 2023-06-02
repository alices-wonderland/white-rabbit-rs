use crate::journal::{
  journal_user, ActiveModel, Column, Command, Entity, Model, Presentation, PrimaryKey, Query,
};
use crate::user::{Role, User};
use crate::{utils, AggregateRoot, Permission};
use itertools::Itertools;
use sea_orm::entity::prelude::*;
use sea_orm::IntoActiveModel;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

use uuid::Uuid;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Journal {
  pub id: Uuid,
  pub name: String,
  pub description: String,
  pub unit: String,
  pub admins: HashSet<Uuid>,
  pub members: HashSet<Uuid>,
}

impl Journal {
  pub fn new(
    name: impl ToString,
    description: impl ToString,
    unit: impl ToString,
    admins: &[User],
    members: &[User],
  ) -> Journal {
    Journal {
      id: Uuid::new_v4(),
      name: name.to_string(),
      description: description.to_string(),
      unit: unit.to_string(),
      admins: utils::get_ids(admins),
      members: utils::get_ids(members),
    }
  }
}

impl From<Journal> for Model {
  fn from(Journal { id, name, description, unit, .. }: Journal) -> Self {
    Self { id, name, description, unit }
  }
}

impl From<Journal> for HashSet<journal_user::Model> {
  fn from(Journal { id, admins, members, .. }: Journal) -> Self {
    let mut result = HashMap::<Uuid, journal_user::Field>::default();
    for id in admins {
      result.insert(id, journal_user::Field::Admin);
    }
    for id in members {
      result.insert(id, journal_user::Field::Member);
    }

    result
      .into_iter()
      .map(|(user_id, field)| journal_user::Model { journal_id: id, user_id, field })
      .collect()
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

  fn compare_by_field(&self, other: &Self, field: impl ToString) -> Option<Ordering> {
    match field.to_string().as_str() {
      "id" => Some(self.id.cmp(&other.id)),
      "name" => Some(self.name.cmp(&other.name)),
      _ => None,
    }
  }

  async fn from_models(
    db: &impl ConnectionTrait,
    models: Vec<Self::Model>,
  ) -> crate::Result<Vec<Self>> {
    let mut results = Vec::new();
    let journal_users = models.load_many(journal_user::Entity, db).await?;
    for (journal, users) in models.into_iter().zip(journal_users.into_iter()) {
      results.push(Self {
        id: journal.id,
        name: journal.name,
        description: journal.description,
        unit: journal.unit,
        admins: users
          .iter()
          .filter(|u| u.field == journal_user::Field::Admin)
          .map(|u| u.user_id)
          .collect(),
        members: users
          .iter()
          .filter(|u| u.field == journal_user::Field::Member)
          .map(|u| u.user_id)
          .collect(),
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
      .flat_map(|root| HashSet::<journal_user::Model>::from(root.clone()))
      .unique()
      .map(|model| model.into_active_model())
      .collect::<Vec<_>>();
    journal_user::Entity::insert_many(journal_users).exec(db).await?;

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
        .collect()
    } else {
      HashMap::default()
    })
  }
}
