use crate::journal::{
  journal_user, ActiveModel, Column, Command, Entity, Model, Presentation, PrimaryKey, Query,
};
use crate::user::{Role, User};
use crate::{utils, AggregateRoot, Permission, Result};
use itertools::Itertools;
use sea_orm::entity::prelude::*;
use sea_orm::sea_query::{Expr, OnConflict};
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

  fn sortable_column(field: impl ToString) -> Option<Self::Column> {
    match field.to_string().as_str() {
      "name" => Some(Column::Name),
      "unit" => Some(Column::Unit),
      _ => None,
    }
  }

  fn compare_by_field(&self, other: &Self, field: impl ToString) -> Option<Ordering> {
    match field.to_string().as_str() {
      "id" => Some(self.id.cmp(&other.id)),
      "name" => Some(self.name.cmp(&other.name)),
      "unit" => Some(self.unit.cmp(&other.unit)),
      _ => None,
    }
  }

  async fn from_models(
    db: &impl ConnectionTrait,
    models: Vec<Self::Model>,
  ) -> crate::Result<Vec<Self>> {
    let model_ids = models.iter().map(|model| model.id).collect::<HashSet<_>>();
    let mut results = Vec::new();
    let users = journal_user::Entity::find()
      .filter(journal_user::Column::JournalId.is_in(model_ids))
      .all(db)
      .await?;
    for journal in models {
      let admins = users
        .iter()
        .filter_map(|user| {
          if user.journal_id == journal.id && user.field == journal_user::Field::Admin {
            Some(user.user_id)
          } else {
            None
          }
        })
        .collect();
      let members = users
        .iter()
        .filter_map(|user| {
          if user.journal_id == journal.id && user.field == journal_user::Field::Member {
            Some(user.user_id)
          } else {
            None
          }
        })
        .collect();

      results.push(Self {
        id: journal.id,
        name: journal.name,
        description: journal.description,
        unit: journal.unit,
        admins,
        members,
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
    let mut on_conflict = OnConflict::column(Self::primary_column());
    on_conflict.update_columns([Column::Name, Column::Description, Column::Unit]);
    Entity::insert_many(journals).on_conflict(on_conflict).exec(db).await?;

    let journal_ids = utils::get_ids(&roots);

    journal_user::Entity::delete_many()
      .filter(journal_user::Column::JournalId.is_in(journal_ids.clone()))
      .exec(db)
      .await?;
    let journal_users = roots
      .iter()
      .flat_map(|root| HashSet::<journal_user::Model>::from(root.clone()))
      .unique()
      .map(|model| model.into_active_model())
      .collect::<Vec<_>>();
    journal_user::Entity::insert_many(journal_users).exec(db).await?;

    Ok(())
  }

  async fn do_delete(db: &impl ConnectionTrait, roots: Vec<Self>) -> Result<()> {
    let ids = utils::get_ids(&roots);

    let _ = journal_user::Entity::delete_many()
      .filter(Expr::col(journal_user::Column::JournalId).is_in(ids.clone()))
      .exec(db)
      .await?;

    let _ = Self::Entity::delete_many()
      .filter(Expr::col(Self::primary_column()).is_in(ids.clone()))
      .exec(db)
      .await?;

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
