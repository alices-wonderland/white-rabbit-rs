use sea_orm::sea_query::{Condition, IntoCondition, JoinType};
use sea_orm::{ColumnTrait, EntityTrait, QuerySelect, RelationTrait, Select};

use super::read_service::{AbstractReadService, ExternalQuery, FullTextQuery, IdQuery, TextQuery};
use super::write_service::AbstractWriteService;
use crate::models::user::Role;
use crate::models::{auth_id, user, User};
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
  pub role: Option<Role>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(default)]
  pub auth_id_providers: Option<Vec<String>>,
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
        fields: Some(vec!["name".to_owned()]),
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
  type PrimaryKey = user::PrimaryKey;
  type Query = UserQuery;

  async fn filter_by_external_query(items: Vec<user::Model>, external_query: &ExternalQuery) -> Vec<user::Model> {
    items
      .into_iter()
      .filter(|item| match external_query {
        ExternalQuery::ContainUser(id) => item.id == *id,
        ExternalQuery::FullText(FullTextQuery { value, .. }) => item.name.contains(value),
      })
      .collect()
  }

  fn find_all_select() -> Select<User> {
    User::find()
      .join(JoinType::LeftJoin, user::Relation::AuthId.def())
      .group_by(user::Column::Id)
  }

  fn sortable_field(field: &str) -> Option<user::Column> {
    match field {
      "name" => Some(user::Column::Name),
      _ => None,
    }
  }

  fn primary_field() -> user::Column {
    user::Column::Id
  }

  fn primary_value(model: &Self::Model) -> i32 {
    model.id
  }
}

#[async_trait::async_trait]
impl AbstractWriteService for UserService {}

#[cfg(test)]
mod tests {
  use std::env;

  use migration::IntoCondition;
  use sea_orm::{DbBackend, EntityTrait, QueryFilter, QueryTrait};

  use crate::{
    models,
    models::user::Role,
    services::read_service::{IdQuery, TextQuery},
  };

  use super::UserQuery;

  #[test]
  fn test_condition() {
    env::set_var("RUST_LOG", "info");
    let _ = env_logger::try_init();

    let query = UserQuery {
      id: Some(IdQuery::Multiple(vec![1, 2, 3])),
      name: Some(TextQuery::FullText("FullTextValue".to_owned())),
      role: Some(Role::Admin),
      auth_id_providers: Some(vec![
        "Provider 1".to_string(),
        "Provider 2".to_string(),
        "Provider 3".to_string(),
      ]),
    };

    let statement = models::User::find()
      .find_also_related(models::AuthId)
      .filter(query.clone().into_condition())
      .build(DbBackend::Sqlite)
      .to_string();
    log::info!("sql: {}", statement);

    let statement = models::User::find()
      .find_with_related(models::AuthId)
      .filter(query.into_condition())
      .build(DbBackend::Sqlite)
      .to_string();
    log::info!("sql: {}", statement);

    let query = UserQuery::default();

    let statement = models::User::find()
      .filter(query.into_condition())
      .build(DbBackend::Sqlite)
      .to_string();

    log::info!("sql: {}", statement);
  }
}
