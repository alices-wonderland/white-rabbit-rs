use std::collections::HashSet;

use futures::{stream, StreamExt};
use sea_orm::{
  sea_query::{Condition, IntoCondition, JoinType},
  ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, EntityTrait, ModelTrait, PaginatorTrait, QueryFilter,
  QuerySelect, RelationTrait, Select, Set,
};
use serde::{Deserialize, Serialize};

use crate::{
  errors::Error,
  models::{
    account::{self, Strategy, Type},
    account_tag, journal, user, Account, AccountTag, Journal,
  },
};

use super::{
  read_service::{AbstractReadService, ExternalQuery, FullTextQuery, IdQuery, TextQuery},
  write_service::{AbstractCommand, AbstractWriteService},
  AuthUser, JournalService, Permission, FIELD_DESCRIPTION, FIELD_ID, FIELD_NAME, FIELD_TAG, FIELD_UNIT,
  MAX_DESCRIPTION, MAX_NAME, MAX_UNIT, MIN_NAME,
};

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct AccountQuery {
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(default)]
  #[serde(rename = "__fullText")]
  pub full_text: Option<FullTextQuery>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(default)]
  pub id: Option<IdQuery>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(default)]
  pub journal: Option<uuid::Uuid>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(default)]
  pub name: Option<TextQuery>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(default)]
  pub description: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(default)]
  pub typ: Option<account::Type>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(default)]
  pub strategy: Option<account::Strategy>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(default)]
  pub unit: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(default)]
  pub tag: Option<TextQuery>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(default)]
  pub include_archived: Option<bool>,
}

impl IntoCondition for AccountQuery {
  fn into_condition(self) -> Condition {
    let mut cond = Condition::all();

    if let Some(id) = self.id {
      cond = match id {
        IdQuery::Single(id) => cond.add(account::Column::Id.eq(id)),
        IdQuery::Multiple(ids) => cond.add(account::Column::Id.is_in(ids)),
      }
    }

    if let Some(journal) = self.journal {
      cond = cond.add(account::Column::JournalId.eq(journal));
    }

    if let Some(TextQuery::Value(name)) = self.name {
      cond = cond.add(account::Column::Name.eq(name));
    }

    if let Some(typ) = self.typ {
      cond = cond.add(account::Column::Typ.eq(typ));
    }

    if let Some(strategy) = self.strategy {
      cond = cond.add(account::Column::Strategy.eq(strategy));
    }

    if let Some(unit) = self.unit {
      cond = cond.add(account::Column::Unit.eq(unit));
    }

    if let Some(TextQuery::Value(tag)) = self.tag {
      cond = cond.add(account_tag::Column::Tag.eq(tag));
    }

    if !self.include_archived.unwrap_or(false) {
      cond = cond.add(account::Column::IsArchived.eq(false));
    }

    cond
  }
}

impl From<AccountQuery> for Vec<ExternalQuery> {
  fn from(value: AccountQuery) -> Self {
    let mut result = Vec::new();

    if let Some(value) = value.full_text {
      result.push(ExternalQuery::FullText(value));
    }

    if let Some(TextQuery::FullText(value)) = value.name {
      result.push(ExternalQuery::FullText(FullTextQuery {
        fields: Some(vec![FIELD_NAME.to_owned()]),
        value,
      }));
    }

    if let Some(value) = value.description {
      result.push(ExternalQuery::FullText(FullTextQuery {
        fields: Some(vec![FIELD_DESCRIPTION.to_owned()]),
        value,
      }));
    }

    if let Some(TextQuery::FullText(value)) = value.tag {
      result.push(ExternalQuery::FullText(FullTextQuery {
        fields: Some(vec![FIELD_TAG.to_owned()]),
        value,
      }));
    }

    result
  }
}

pub struct AccountService {}

#[async_trait::async_trait]
impl AbstractReadService for AccountService {
  type Model = account::Model;
  type Entity = Account;
  type PrimaryKey = account::PrimaryKey;
  type Query = AccountQuery;

  async fn is_readable(conn: &impl ConnectionTrait, operator: &AuthUser, model: &Self::Model) -> bool {
    if let AuthUser::User(user) = operator {
      (user.role > user::Role::User)
        || if let Ok(Some(journal)) = Journal::find_by_id(model.journal_id).one(conn).await {
          JournalService::is_readable(conn, operator, &journal).await
        } else {
          false
        }
    } else {
      false
    }
  }

  async fn filter_by_external_query(
    conn: &impl ConnectionTrait,
    items: Vec<Self::Model>,
    external_query: &ExternalQuery,
  ) -> Vec<Self::Model> {
    stream::iter(items)
      .filter_map(|item| async {
        match external_query {
          ExternalQuery::FullText(FullTextQuery { value, fields }) => {
            let fields = fields.clone().unwrap_or_else(|| {
              vec![
                FIELD_NAME.to_owned(),
                FIELD_DESCRIPTION.to_owned(),
                FIELD_TAG.to_owned(),
              ]
            });

            for field in fields {
              if match field.as_str() {
                FIELD_NAME => item.name.contains(value),
                FIELD_DESCRIPTION => item.description.contains(value),
                FIELD_TAG => item
                  .find_related(AccountTag)
                  .all(conn)
                  .await
                  .unwrap_or_default()
                  .into_iter()
                  .any(|tag| tag.tag.contains(value)),
                _ => true,
              } {
                return Some(item);
              }
            }
            None
          }
          _ => None,
        }
      })
      .collect()
      .await
  }

  fn find_all_select() -> Select<Self::Entity> {
    Account::find()
      .join(JoinType::LeftJoin, account_tag::Relation::Account.def().rev())
      .group_by(account::Column::Id)
  }

  fn primary_field() -> account::Column {
    account::Column::Id
  }

  fn primary_value(model: &Self::Model) -> uuid::Uuid {
    model.id
  }

  fn sortable_field(field: &str) -> Option<account::Column> {
    match field {
      "name" => Some(account::Column::Name),
      "journal" => Some(account::Column::JournalId),
      "typ" => Some(account::Column::Typ),
      "strategy" => Some(account::Column::Strategy),
      "unit" => Some(account::Column::Unit),
      "is_archived" => Some(account::Column::IsArchived),
      _ => None,
    }
  }
}

pub enum AccountCommand {
  Create(AccountCommandCreate),
  Update(AccountCommandUpdate),
  Delete(uuid::Uuid),
}

pub struct AccountCommandCreate {
  pub target_id: Option<uuid::Uuid>,
  pub journal_id: uuid::Uuid,
  pub name: String,
  pub description: String,
  pub typ: Type,
  pub strategy: Strategy,
  pub unit: String,
  pub tags: HashSet<String>,
}

pub struct AccountCommandUpdate {
  pub target_id: uuid::Uuid,
  pub name: Option<String>,
  pub description: Option<String>,
  pub typ: Option<Type>,
  pub strategy: Option<Strategy>,
  pub unit: Option<String>,
  pub tags: Option<HashSet<String>>,
  pub is_archived: Option<bool>,
}

impl AccountCommandUpdate {
  pub fn is_empty(&self) -> bool {
    self.name.is_none()
      && self.description.is_none()
      && self.typ.is_none()
      && self.strategy.is_none()
      && self.unit.is_none()
  }
}

impl AbstractCommand for AccountCommand {
  fn target_id(&self) -> Option<uuid::Uuid> {
    match self {
      AccountCommand::Create(AccountCommandCreate { target_id, .. }) => target_id.to_owned(),
      AccountCommand::Update(AccountCommandUpdate { target_id, .. }) => Some(*target_id),
      AccountCommand::Delete(id) => Some(*id),
    }
  }

  fn with_target_id(self, id: uuid::Uuid) -> Self {
    match self {
      AccountCommand::Create(command) => AccountCommand::Create(AccountCommandCreate {
        target_id: Some(id),
        ..command
      }),
      AccountCommand::Update(command) => AccountCommand::Update(AccountCommandUpdate {
        target_id: id,
        ..command
      }),
      AccountCommand::Delete(_) => AccountCommand::Delete(id),
    }
  }
}

impl AccountService {
  fn validate(model: &account::ActiveModel, tags: Option<&HashSet<String>>) -> anyhow::Result<()> {
    let mut errors = Vec::<Error>::new();
    match &model.name {
      ActiveValue::Set(name) if name.len() < MIN_NAME || name.len() > MAX_NAME => errors.push(Error::LengthRange {
        entity: account::TYPE.to_owned(),
        field: FIELD_NAME.to_owned(),
        min: MIN_NAME,
        max: MAX_NAME,
      }),
      _ => (),
    }

    match &model.description {
      ActiveValue::Set(description) if description.len() > MAX_DESCRIPTION => errors.push(Error::MaxLength {
        entity: account::TYPE.to_owned(),
        field: FIELD_DESCRIPTION.to_owned(),
        value: MAX_DESCRIPTION,
      }),
      _ => (),
    }

    match &model.unit {
      ActiveValue::Set(unit) if unit.len() > MAX_UNIT => errors.push(Error::MaxLength {
        entity: account::TYPE.to_owned(),
        field: FIELD_UNIT.to_owned(),
        value: MAX_UNIT,
      }),
      _ => (),
    }

    if let Some(tags) = tags {
      errors.append(&mut Error::validate_tags(account::TYPE, tags));
    }

    match errors.first() {
      Some(error) if errors.len() == 1 => Err(error.clone())?,
      Some(_) => Err(Error::Errors(errors))?,
      None => Ok(()),
    }
  }

  pub async fn create(
    conn: &impl ConnectionTrait,
    operator: user::Model,
    command: AccountCommandCreate,
  ) -> anyhow::Result<account::Model> {
    if Account::find()
      .filter(account::Column::Name.eq(command.name.clone()))
      .count(conn)
      .await?
      > 0
    {
      return Err(Error::AlreadyExists {
        entity: account::TYPE.to_owned(),
        field: FIELD_NAME.to_owned(),
        value: command.name,
      })?;
    }

    let journal =
      if let Some(journal) = JournalService::find_by_id(conn, AuthUser::User(operator), command.journal_id).await? {
        journal
      } else {
        return Err(Error::NotFound {
          entity: journal::TYPE.to_owned(),
          field: FIELD_ID.to_owned(),
          value: command.journal_id.to_string(),
        })?;
      };

    let account = account::ActiveModel {
      id: Set(uuid::Uuid::new_v4()),
      journal_id: Set(journal.id),
      name: Set(command.name.clone()),
      description: Set(command.description.clone()),
      typ: Set(command.typ.clone()),
      strategy: Set(command.strategy.clone()),
      unit: Set(command.unit.clone()),
      is_archived: Set(false),
    };
    Self::validate(&account, Some(&command.tags))?;

    let account = account.insert(conn).await?;
    let tags: Vec<_> = command
      .tags
      .into_iter()
      .map(|tag| account_tag::ActiveModel {
        account_id: Set(account.id),
        tag: Set(tag),
      })
      .collect();
    let _ = AccountTag::insert_many(tags).exec(conn).await?;

    Ok(account)
  }

  pub async fn update(
    conn: &impl ConnectionTrait,
    operator: user::Model,
    command: AccountCommandUpdate,
  ) -> anyhow::Result<account::Model> {
    let account = Account::find_by_id(command.target_id)
      .one(conn)
      .await?
      .ok_or_else(|| Error::NotFound {
        entity: account::TYPE.to_owned(),
        field: FIELD_ID.to_owned(),
        value: command.target_id.to_string(),
      })?;

    Self::check_writeable(conn, &operator, &account).await?;

    if command.is_empty() {
      return Ok(account);
    }

    let mut model = account::ActiveModel {
      id: Set(command.target_id),
      ..Default::default()
    };

    if let Some(name) = command.name {
      if Account::find()
        .filter(account::Column::Name.eq(name.clone()))
        .count(conn)
        .await?
        > 0
      {
        return Err(Error::AlreadyExists {
          entity: account::TYPE.to_owned(),
          field: FIELD_NAME.to_owned(),
          value: name,
        })?;
      }

      model.name = Set(name);
    }

    if let Some(description) = command.description {
      model.description = Set(description);
    }

    if let Some(typ) = command.typ {
      model.typ = Set(typ);
    }

    if let Some(strategy) = command.strategy {
      model.strategy = Set(strategy);
    }

    if let Some(unit) = command.unit {
      model.unit = Set(unit);
    }

    if let Some(is_archived) = command.is_archived {
      model.is_archived = Set(is_archived);
    }

    Self::validate(&model, command.tags.as_ref())?;

    if let Some(tags) = command.tags {
      let tags: Vec<_> = tags
        .into_iter()
        .map(|tag| account_tag::ActiveModel {
          account_id: Set(account.id),
          tag: Set(tag),
        })
        .collect();
      let _ = AccountTag::delete_many()
        .filter(account_tag::Column::AccountId.eq(account.id))
        .exec(conn)
        .await?;
      let _ = AccountTag::insert_many(tags).exec(conn).await?;
    }

    Ok(model.update(conn).await?)
  }

  pub async fn delete(conn: &impl ConnectionTrait, operator: user::Model, id: uuid::Uuid) -> anyhow::Result<()> {
    let account = Account::find_by_id(id)
      .one(conn)
      .await?
      .ok_or_else(|| Error::NotFound {
        entity: account::TYPE.to_owned(),
        field: FIELD_ID.to_owned(),
        value: id.to_string(),
      })?;

    Self::check_writeable(conn, &operator, &account).await?;

    let model = account::ActiveModel {
      id: Set(id),
      ..Default::default()
    };
    model.delete(conn).await?;
    Ok(())
  }
}

#[async_trait::async_trait]
impl AbstractWriteService for AccountService {
  type Command = AccountCommand;

  async fn check_writeable(conn: &impl ConnectionTrait, user: &user::Model, model: &Self::Model) -> anyhow::Result<()> {
    if user.role > user::Role::Admin {
      Ok(())
    } else if let Some(journal) = Journal::find_by_id(model.journal_id).one(conn).await? {
      JournalService::check_writeable(conn, user, &journal).await
    } else {
      Err(Error::NotFound {
        entity: journal::TYPE.to_owned(),
        field: FIELD_ID.to_owned(),
        value: model.journal_id.to_string(),
      })?
    }
  }

  async fn handle(
    conn: &impl ConnectionTrait,
    operator: AuthUser,
    command: Self::Command,
  ) -> anyhow::Result<Option<Self::Model>> {
    if let AuthUser::User(operator) = operator {
      match command {
        AccountCommand::Create(command) => {
          let result = Self::create(conn, operator, command).await?;
          Ok(Some(result))
        }
        AccountCommand::Update(command) => {
          let result = Self::update(conn, operator, command).await?;
          Ok(Some(result))
        }
        AccountCommand::Delete(id) => {
          Self::delete(conn, operator, id).await?;
          Ok(None)
        }
      }
    } else {
      return Err(Error::InvalidPermission {
        user: operator.get_id(),
        entity: account::TYPE.to_owned(),
        id: command.target_id(),
        permission: Permission::Write,
      })?;
    }
  }
}
