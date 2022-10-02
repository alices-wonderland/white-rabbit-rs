use std::collections::HashSet;

use sea_orm::sea_query::{Condition, IntoCondition, JoinType};
use sea_orm::{
  ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, EntityTrait, PaginatorTrait, QueryFilter, QuerySelect,
  RelationTrait, Select, Set,
};

use super::read_service::{AbstractReadService, ExternalQuery, FullTextQuery, IdQuery, TextQuery};
use super::write_service::{AbstractCommand, AbstractWriteService};
use super::{
  AuthUser, Permission, FIELD_AUTH_IDS, FIELD_ID, FIELD_NAME, MAX_AUTH_IDS, MAX_NAME, MIN_AUTH_IDS, MIN_NAME,
};
use crate::errors::Error;
use crate::models::{auth_id, user, AuthId, User};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct UserQuery {
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(default)]
  pub id: Option<IdQuery>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(default)]
  pub name: Option<TextQuery>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(default)]
  pub role: Option<user::Role>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(default)]
  pub auth_id_providers: Option<HashSet<String>>,
}

impl IntoCondition for UserQuery {
  fn into_condition(self) -> Condition {
    let mut cond = Condition::all();

    if let Some(id) = self.id {
      cond = match id {
        IdQuery::Single(id) => cond.add(user::Column::Id.eq(id)),
        IdQuery::Multiple(ids) => cond.add(user::Column::Id.is_in(ids)),
      }
    }

    if let Some(TextQuery::Value(name)) = self.name {
      cond = cond.add(user::Column::Name.eq(name));
    }

    if let Some(role) = self.role {
      cond = cond.add(user::Column::Role.eq(role));
    }

    if let Some(values) = self.auth_id_providers {
      cond = cond.add(auth_id::Column::Provider.is_in(values));
    }

    cond
  }
}

impl From<UserQuery> for Vec<ExternalQuery> {
  fn from(value: UserQuery) -> Self {
    let mut result = Vec::new();

    if let Some(TextQuery::FullText(value)) = value.name {
      result.push(ExternalQuery::FullText(FullTextQuery {
        fields: Some(vec![FIELD_NAME.to_owned()]),
        value,
      }));
    }

    result
  }
}

pub struct UserService {}

#[async_trait::async_trait]
impl AbstractReadService for UserService {
  type Model = user::Model;
  type Entity = User;
  type Presentation = user::Presentation;
  type PrimaryKey = user::PrimaryKey;
  type Query = UserQuery;

  async fn filter_by_external_query(
    _: &impl ConnectionTrait,
    items: Vec<user::Model>,
    external_query: &ExternalQuery,
  ) -> Vec<user::Model> {
    items
      .into_iter()
      .filter(|item| match external_query {
        ExternalQuery::FullText(FullTextQuery { value, .. }) => item.name.contains(value),
        _ => true,
      })
      .collect()
  }

  fn find_all_select() -> Select<User> {
    User::find()
      .join(JoinType::LeftJoin, user::Relation::AuthId.def())
      .group_by(user::Column::Id)
  }

  fn primary_field() -> user::Column {
    user::Column::Id
  }

  fn primary_value(model: &Self::Model) -> uuid::Uuid {
    model.id
  }

  fn sortable_field(field: &str) -> Option<user::Column> {
    match field {
      FIELD_NAME => Some(user::Column::Name),
      _ => None,
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserCommand {
  Create(UserCommandCreate),
  Update(UserCommandUpdate),
  Delete(uuid::Uuid),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserCommandCreate {
  pub target_id: Option<uuid::Uuid>,
  pub name: String,
  pub role: user::Role,
  pub auth_ids: HashSet<(String, String)>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserCommandUpdate {
  pub target_id: uuid::Uuid,
  pub name: Option<String>,
  pub role: Option<user::Role>,
  pub auth_ids: Option<HashSet<(String, String)>>,
}

impl UserCommandUpdate {
  pub fn is_empty(&self) -> bool {
    self.name.is_none() && self.role.is_none() && self.auth_ids.is_none()
  }
}

impl AbstractCommand for UserCommand {
  fn target_id(&self) -> Option<uuid::Uuid> {
    match self {
      UserCommand::Create(UserCommandCreate { target_id, .. }) => target_id.to_owned(),
      UserCommand::Update(UserCommandUpdate { target_id, .. }) => Some(*target_id),
      UserCommand::Delete(id) => Some(*id),
    }
  }

  fn with_target_id(self, id: uuid::Uuid) -> Self {
    match self {
      UserCommand::Create(command) => UserCommand::Create(UserCommandCreate {
        target_id: Some(id),
        ..command
      }),
      UserCommand::Update(command) => UserCommand::Update(UserCommandUpdate {
        target_id: id,
        ..command
      }),
      UserCommand::Delete(_) => UserCommand::Delete(id),
    }
  }
}

impl UserService {
  fn validate(model: &user::ActiveModel, auth_ids: Option<&HashSet<(String, String)>>) -> anyhow::Result<()> {
    let mut errors = Vec::<Error>::new();

    match &model.name {
      ActiveValue::Set(name) if name.len() < MIN_NAME || name.len() > MAX_NAME => errors.push(Error::LengthRange {
        entity: user::TYPE.to_owned(),
        field: FIELD_NAME.to_owned(),
        min: MIN_NAME,
        max: MAX_NAME,
      }),
      _ => (),
    }

    match auth_ids {
      Some(auth_ids) if auth_ids.len() > MAX_AUTH_IDS || auth_ids.len() < MIN_AUTH_IDS => {
        errors.push(Error::LengthRange {
          entity: user::TYPE.to_owned(),
          field: FIELD_AUTH_IDS.to_owned(),
          min: MIN_AUTH_IDS,
          max: MAX_AUTH_IDS,
        })
      }
      _ => {}
    }

    match errors.first() {
      Some(error) if errors.len() == 1 => Err(error.clone())?,
      Some(_) => Err(Error::Errors(errors))?,
      None => Ok(()),
    }
  }

  pub async fn create(
    conn: &impl ConnectionTrait,
    operator: &AuthUser,
    command: UserCommandCreate,
  ) -> anyhow::Result<user::Model> {
    if User::find()
      .filter(user::Column::Name.eq(command.name.clone()))
      .count(conn)
      .await?
      > 0
    {
      return Err(Error::AlreadyExists {
        entity: user::TYPE.to_owned(),
        field: FIELD_NAME.to_owned(),
        value: command.name,
      })?;
    }

    let auth_ids = match operator {
      AuthUser::User(user) if user.role != user::Role::User && user.role > command.role => command.auth_ids,
      AuthUser::Id(auth_id) if command.role == user::Role::User => HashSet::from_iter(vec![auth_id.clone()]),
      _ => {
        return Err(Error::InvalidPermission {
          user: operator.id(),
          entity: user::TYPE.to_owned(),
          id: None,
          permission: Permission::Write,
        })?
      }
    };

    let model = user::ActiveModel {
      id: Set(uuid::Uuid::new_v4()),
      name: Set(command.name),
      role: Set(command.role),
    };
    Self::validate(&model, Some(&auth_ids))?;
    let model = model.insert(conn).await?;

    let auth_ids = auth_ids
      .into_iter()
      .map(|(provider, value)| auth_id::ActiveModel {
        user_id: Set(model.id),
        provider: Set(provider),
        value: Set(value),
      })
      .collect::<Vec<_>>();
    AuthId::insert_many(auth_ids).exec(conn).await?;

    Ok(model)
  }

  pub async fn update(
    conn: &impl ConnectionTrait,
    operator: &user::Model,
    command: UserCommandUpdate,
  ) -> anyhow::Result<user::Model> {
    let user = User::find_by_id(command.target_id)
      .one(conn)
      .await?
      .ok_or_else(|| Error::NotFound {
        entity: user::TYPE.to_owned(),
        field: FIELD_ID.to_owned(),
        value: command.target_id.to_string(),
      })?;

    if operator.role != user::Role::Owner
      && operator.id != command.target_id
      && (operator.role <= user.role || operator.role <= command.role.clone().unwrap_or_default())
    {
      return Err(Error::InvalidPermission {
        user: operator.id.to_string(),
        entity: user::TYPE.to_owned(),
        id: Some(command.target_id),
        permission: Permission::Write,
      })?;
    }

    if command.is_empty() {
      return Ok(user);
    }

    let mut model = user::ActiveModel {
      id: Set(command.target_id),
      ..Default::default()
    };

    if let Some(name) = command.name {
      model.name = Set(name);
    }

    if let Some(role) = command.role {
      model.role = Set(role);
    }

    Self::validate(&model, command.auth_ids.as_ref())?;

    if let Some(auth_ids) = command.auth_ids {
      AuthId::delete_many()
        .filter(auth_id::Column::UserId.eq(user.id))
        .exec(conn)
        .await?;
      let auth_ids = auth_ids
        .into_iter()
        .map(|(provider, value)| auth_id::ActiveModel {
          user_id: Set(user.id),
          provider: Set(provider),
          value: Set(value),
        })
        .collect::<Vec<_>>();
      AuthId::insert_many(auth_ids).exec(conn).await?;
    }

    Ok(model.update(conn).await?)
  }

  pub async fn delete(conn: &impl ConnectionTrait, operator: &user::Model, id: uuid::Uuid) -> anyhow::Result<()> {
    let user = User::find_by_id(id).one(conn).await?.ok_or_else(|| Error::NotFound {
      entity: user::TYPE.to_owned(),
      field: FIELD_ID.to_owned(),
      value: id.to_string(),
    })?;
    if operator.role != user::Role::Owner && operator.id != id && operator.role <= user.role {
      return Err(Error::InvalidPermission {
        user: operator.id.to_string(),
        entity: user::TYPE.to_owned(),
        id: Some(id),
        permission: Permission::Write,
      })?;
    }
    let model = user::ActiveModel {
      id: Set(id),
      ..Default::default()
    };
    model.delete(conn).await?;
    Ok(())
  }
}

#[async_trait::async_trait]
impl AbstractWriteService for UserService {
  type Command = UserCommand;

  async fn handle(
    conn: &impl ConnectionTrait,
    operator: &AuthUser,
    command: Self::Command,
  ) -> anyhow::Result<Option<Self::Model>> {
    let target_id = command.target_id();
    match (command, operator) {
      (UserCommand::Create(command), _) => {
        let result = Self::create(conn, operator, command).await?;
        Ok(Some(result))
      }
      (UserCommand::Update(command), AuthUser::User(operator)) => {
        let result = Self::update(conn, operator, command).await?;
        Ok(Some(result))
      }
      (UserCommand::Delete(id), AuthUser::User(operator)) => {
        Self::delete(conn, operator, id).await?;
        Ok(None)
      }
      _ => {
        return Err(Error::InvalidPermission {
          user: operator.id(),
          entity: user::TYPE.to_owned(),
          id: target_id,
          permission: Permission::Write,
        })?
      }
    }
  }
}
