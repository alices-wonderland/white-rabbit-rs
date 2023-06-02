use crate::user::{
  ActiveModel, Column, Command, CommandCreate, Entity, Model, Presentation, PrimaryKey, Query,
  Role, User,
};
use crate::{AggregateRoot, Error, FindAllArgs, Permission, Repository};
use futures::TryStreamExt;
use sea_orm::entity::prelude::*;
use sea_orm::StreamTrait;
use std::cmp::Ordering;
use std::collections::HashMap;

use uuid::Uuid;

#[async_trait::async_trait]
impl AggregateRoot for User {
  type Model = Model;
  type ActiveModel = ActiveModel;
  type Entity = Entity;
  type Presentation = Presentation;
  type PrimaryKey = PrimaryKey;
  type Query = Query;
  type Column = Column;
  type Command = Command;

  fn typ() -> &'static str {
    "User"
  }

  fn id(&self) -> Uuid {
    self.id
  }

  fn primary_column() -> Column {
    Column::Id
  }

  fn compare_by_field(&self, other: &Self, field: impl ToString) -> Option<Ordering> {
    match field.to_string().as_str() {
      "id" => Some(self.id.cmp(&other.id)),
      "name" => Some(self.name.cmp(&other.name)),
      "role" => Some(self.role.cmp(&other.role)),
      _ => None,
    }
  }

  async fn from_models(
    _db: &impl ConnectionTrait,
    models: Vec<Self::Model>,
  ) -> crate::Result<Vec<Self>> {
    Ok(models)
  }

  async fn handle(
    db: &(impl ConnectionTrait + StreamTrait),
    operator: Option<&Model>,
    command: Self::Command,
  ) -> crate::Result<Vec<Self>> {
    Ok(match command {
      Command::Create(command) => vec![Model::create(db, operator, command).await?],
    })
  }

  async fn get_permission(
    _db: &impl ConnectionTrait,
    operator: Option<&Self>,
    models: &[Self],
  ) -> crate::Result<HashMap<Uuid, Permission>> {
    Ok(
      models
        .iter()
        .map(|model| {
          (
            model.id(),
            match operator {
              Some(operator) if operator.role == Role::Admin || operator.id == model.id() => {
                Permission::ReadWrite
              }
              _ => Permission::ReadOnly,
            },
          )
        })
        .collect::<HashMap<_, _>>(),
    )
  }
}

impl Model {
  pub fn new(name: impl ToString, role: Role) -> Model {
    Model { id: Uuid::new_v4(), name: name.to_string(), role }
  }

  async fn create(
    db: &(impl ConnectionTrait + StreamTrait),
    operator: Option<&Model>,
    command: CommandCreate,
  ) -> crate::Result<Model> {
    if Repository::<Model>::do_find_all(
      db,
      FindAllArgs {
        query: Query { name: (command.name.clone(), false), ..Default::default() },
        ..Default::default()
      },
    )
    .await?
    .try_next()
    .await?
    .is_some()
    {
      return Err(Error::already_exists::<Self>(vec![("name", command.name)]));
    }

    let name = command.name.trim();
    if name.len() < 4 || name.len() > 128 {
      return Err(Error::NotInRange { field: "name.length".to_string(), begin: 4, end: 128 });
    }

    let user = Model::new(name, command.role);
    let users = vec![user.clone()];

    match Self::get_permission(db, operator, &users).await?.get(&user.id()) {
      None | Some(Permission::ReadOnly) => return Err(Error::no_write_permission(operator, &user)),
      _ => {}
    }

    Ok(Repository::<Model>::save(db, users).await?.into_iter().last().unwrap())
  }
}
