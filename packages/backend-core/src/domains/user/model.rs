use crate::{
  AggregateRoot, Error, FindAllArgs, Permission, Presentation, Repository, User, UserCommand, UserCommandCreate,
  UserQuery,
};

use futures::TryStreamExt;
use sea_orm::entity::prelude::*;
use sea_orm::StreamTrait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: Uuid,
  #[sea_orm(unique, indexed)]
  pub name: String,
  #[sea_orm(indexed)]
  pub role: Role,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[async_trait::async_trait]
impl<'a> AggregateRoot<'a> for Model {
  type Model = Model;
  type ActiveModel = ActiveModel;
  type Entity = Entity;
  type Presentation = Model;
  type PrimaryKey = PrimaryKey;
  type Query = UserQuery;
  type Column = Column;
  type Command = UserCommand;

  fn typ() -> &'static str {
    "User"
  }

  fn id(&self) -> Uuid {
    self.id
  }

  fn primary_column() -> Column {
    Column::Id
  }

  async fn from_models(_db: &impl ConnectionTrait, models: Vec<Self::Model>) -> crate::Result<Vec<Self>> {
    Ok(models)
  }

  async fn handle(
    db: &(impl ConnectionTrait + StreamTrait),
    operator: Option<Model>,
    command: Self::Command,
  ) -> crate::Result<Vec<Self>> {
    Ok(match command {
      UserCommand::Create(command) => vec![Model::create(db, operator, command).await?],
    })
  }

  async fn get_permission(
    _db: &impl ConnectionTrait,
    operator: Option<&Self>,
    models: &[Self],
  ) -> crate::Result<HashMap<Uuid, Permission>> {
    Ok(HashMap::from_iter(models.iter().map(|model| {
      (
        model.id(),
        match operator {
          Some(operator) if operator.role == Role::Admin || operator.id == model.id() => Permission::ReadWrite,
          _ => Permission::ReadOnly,
        },
      )
    })))
  }
}

impl Model {
  pub fn new(name: impl ToString, role: Role) -> Model {
    Model { id: Uuid::new_v4(), name: name.to_string(), role }
  }

  async fn create(
    db: &(impl ConnectionTrait + StreamTrait),
    operator: Option<Model>,
    command: UserCommandCreate,
  ) -> crate::Result<Model> {
    if Repository::<User>::find_all(
      db,
      FindAllArgs {
        query: UserQuery { name: (command.name.clone(), false), ..Default::default() },
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

    match Self::get_permission(db, operator.as_ref(), &users).await?.get(&user.id()) {
      None | Some(Permission::ReadOnly) => return Err(Error::no_write_permission(operator.as_ref(), &user)),
      _ => {}
    }

    Ok(Repository::<User>::save(db, users).await?.into_iter().last().unwrap())
  }
}

#[async_trait::async_trait]
impl<'a> Presentation<'a> for Model {
  type AggregateRoot = Model;

  async fn from(_db: &impl ConnectionTrait, models: Vec<Self::AggregateRoot>) -> Vec<Self> {
    models
  }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(Some(1))")]
pub enum Role {
  #[sea_orm(string_value = "U")]
  User,
  #[sea_orm(string_value = "A")]
  Admin,
}
