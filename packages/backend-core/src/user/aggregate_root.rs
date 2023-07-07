use crate::user::{
  ActiveModel, Column, Command, CommandCreate, CommandDelete, CommandUpdate, Entity, Model,
  Presentation, PrimaryKey, Query, Role, User,
};
use crate::{
  AggregateRoot, Error, FindAllArgs, Permission, Repository, Result, FIELD_ID, FIELD_NAME,
  FIELD_NAME_LENGTH,
};
use itertools::Itertools;
use sea_orm::entity::prelude::*;
use sea_orm::sea_query::OnConflict;
use sea_orm::{IntoActiveModel, StreamTrait};
use std::cmp::Ordering;
use std::collections::HashMap;
use uuid::Uuid;

pub const FIELD_ROLE: &str = "role";
pub const MIN_NAME: usize = 6;
pub const MAX_NAME: usize = 128;

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

  fn sortable_column(field: impl ToString) -> Option<Self::Column> {
    match field.to_string().as_str() {
      "name" => Some(Column::Name),
      "role" => Some(Column::Role),
      _ => None,
    }
  }

  fn compare_by_field(&self, other: &Self, field: impl ToString) -> Option<Ordering> {
    match field.to_string().as_str() {
      FIELD_ID => Some(self.id.cmp(&other.id)),
      FIELD_NAME => Some(self.name.cmp(&other.name)),
      FIELD_ROLE => Some(self.role.cmp(&other.role)),
      _ => None,
    }
  }

  async fn from_models(_db: &impl ConnectionTrait, models: Vec<Self::Model>) -> Result<Vec<Self>> {
    Ok(models)
  }

  async fn do_save(db: &impl ConnectionTrait, roots: Vec<Self>) -> Result<()> {
    let users = roots
      .iter()
      .unique_by(|root| root.id)
      .map(|root| root.clone().into_active_model())
      .collect::<Vec<_>>();
    let mut on_conflict = OnConflict::column(Self::primary_column());
    on_conflict.update_columns([Column::Name, Column::Role]);
    Entity::insert_many(users).on_conflict(on_conflict).exec(db).await?;
    Ok(())
  }

  async fn handle(
    db: &(impl ConnectionTrait + StreamTrait),
    operator: Option<&Model>,
    command: Self::Command,
  ) -> Result<Vec<Self>> {
    Ok(match command {
      Command::Create(command) => vec![Model::create(db, operator, command).await?],
      Command::Update(command) => vec![Model::update(db, operator, command).await?],
      Command::Delete(command) => {
        let _ = Model::delete(db, operator, command).await?;
        vec![]
      }
    })
  }

  async fn get_permission(
    _db: &impl ConnectionTrait,
    operator: Option<&Self>,
    models: &[Self],
  ) -> Result<HashMap<Uuid, Permission>> {
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

  async fn validate_name(
    db: &(impl ConnectionTrait + StreamTrait),
    name: impl ToString,
  ) -> Result<String> {
    let name = name.to_string().trim().to_string();
    if name.len() < MIN_NAME || name.len() > MAX_NAME {
      Err(Error::NotInRange {
        field: FIELD_NAME_LENGTH.to_string(),
        begin: MIN_NAME,
        end: MAX_NAME,
      })
    } else if !Repository::<Model>::do_find_all(
      db,
      FindAllArgs {
        size: Some(1),
        query: Query { name: (name.clone(), false), ..Default::default() },
        ..Default::default()
      },
      None,
    )
    .await?
    .is_empty()
    {
      Err(Error::already_exists::<Self>(vec![(FIELD_NAME, name)]))
    } else {
      Ok(name)
    }
  }

  fn validate_role(operator: Option<&Model>, role: Role) -> Result<Role> {
    if operator.map(|operator| operator.role >= role).unwrap_or_else(|| role == Role::User) {
      Ok(role)
    } else {
      Err(Error::NoWritePermission {
        operator_id: operator.map(User::id),
        typ: User::typ().to_string(),
        field_values: vec![(FIELD_ROLE.to_string(), role.to_string())],
      })
    }
  }

  async fn create(
    db: &(impl ConnectionTrait + StreamTrait),
    operator: Option<&Model>,
    command: CommandCreate,
  ) -> Result<Model> {
    let name = Self::validate_name(db, command.name).await?;
    let role = Self::validate_role(operator, command.role)?;
    let user = Model::new(name, role);

    Ok(Repository::<Model>::save(db, vec![user]).await?.into_iter().last().unwrap())
  }

  async fn update(
    db: &(impl ConnectionTrait + StreamTrait),
    operator: Option<&Model>,
    command: CommandUpdate,
  ) -> Result<Model> {
    let id: Uuid = command.id.clone().parse()?;
    let mut model = Repository::<User>::find_by_id(db, id)
      .await?
      .ok_or_else(|| Error::not_found::<User>(vec![(FIELD_ID, id)]))?;

    if command.is_empty() {
      return Ok(model);
    }

    Self::check_writeable(db, operator, &[model.clone()]).await?;

    if let Some(name) = command.name {
      model.name = Self::validate_name(db, name).await?;
    }

    if let Some(role) = command.role {
      model.role = Self::validate_role(operator, role)?;
    }

    Ok(model)
  }

  async fn delete(
    db: &(impl ConnectionTrait + StreamTrait),
    operator: Option<&Model>,
    command: CommandDelete,
  ) -> Result<()> {
    if command.id.is_empty() {
      return Ok(());
    }

    let models = Repository::<User>::find_by_ids(db, command.id).await?;
    Self::check_writeable(db, operator, &models).await?;
    Repository::delete(db, models).await
  }
}
