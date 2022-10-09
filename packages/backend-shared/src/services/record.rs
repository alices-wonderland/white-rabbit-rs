use std::collections::{HashMap, HashSet};

use chrono::NaiveDate;
use futures::{stream, StreamExt};
use rust_decimal::Decimal;
use sea_orm::{
  sea_query::IntoCondition, ActiveModelTrait, ActiveValue, ColumnTrait, Condition, ConnectionTrait, EntityTrait,
  JoinType, ModelTrait, PaginatorTrait, QueryFilter, QuerySelect, RelationTrait, Select, Set,
};
use serde::{Deserialize, Serialize};

use crate::{
  models::{
    account, journal,
    record::{self, RecordStateItem, Type},
    record_item, record_tag, user, Account, Journal, Record, RecordItem, RecordTag,
  },
  Error,
};

use super::{
  read_service::{AbstractReadService, ComparableQuery, ExternalQuery, FullTextQuery, IdQuery, TextQuery},
  write_service::{AbstractCommand, AbstractWriteService},
  AuthUser, JournalService, Permission, FIELD_DESCRIPTION, FIELD_ID, FIELD_NAME, FIELD_RECORD_ITEMS, FIELD_TAG,
  MAX_DESCRIPTION, MAX_NAME, MAX_RECORD_ITEMS, MIN_NAME, MIN_RECORD_ITEMS,
};

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct RecordQuery {
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
  #[serde(rename = "type")]
  pub typ: Option<record::Type>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(default)]
  pub date: Option<ComparableQuery<NaiveDate>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(default)]
  pub tag: Option<TextQuery>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(default)]
  pub account: Option<uuid::Uuid>,
}

impl IntoCondition for RecordQuery {
  fn into_condition(self) -> Condition {
    let mut cond = Condition::all();

    if let Some(id) = self.id {
      cond = match id {
        IdQuery::Single(id) => cond.add(record::Column::Id.eq(id)),
        IdQuery::Multiple(ids) => cond.add(record::Column::Id.is_in(ids)),
      }
    }

    if let Some(journal) = self.journal {
      cond = cond.add(record::Column::JournalId.eq(journal));
    }

    match self.name {
      Some(TextQuery { value, full_text }) if !full_text => {
        cond = cond.add(record::Column::Name.eq(value));
      }
      _ => (),
    }

    if let Some(typ) = self.typ {
      cond = cond.add(record::Column::Typ.eq(typ));
    }

    if let Some(ComparableQuery { eq, gt, lt, gte, lte }) = self.date {
      let mut sub_query = Condition::all();
      if let Some(date) = eq {
        sub_query = sub_query.add(record::Column::Date.eq(date));
      }

      if let Some(date) = gt {
        sub_query = sub_query.add(record::Column::Date.gt(date));
      }

      if let Some(date) = lt {
        sub_query = sub_query.add(record::Column::Date.lt(date));
      }

      if let Some(date) = gte {
        sub_query = sub_query.add(record::Column::Date.gte(date));
      }

      if let Some(date) = lte {
        sub_query = sub_query.add(record::Column::Date.lte(date));
      }

      cond = cond.add(sub_query);
    }

    match self.tag {
      Some(TextQuery { value, full_text }) if !full_text => {
        cond = cond.add(record_tag::Column::Tag.eq(value));
      }
      _ => (),
    }

    if let Some(account) = self.account {
      cond = cond.add(record_item::Column::AccountId.eq(account));
    }

    cond
  }
}

impl From<RecordQuery> for Vec<ExternalQuery> {
  fn from(value: RecordQuery) -> Self {
    let mut result = Vec::new();

    if let Some(value) = value.full_text {
      result.push(ExternalQuery::FullText(value));
    }

    match value.name {
      Some(TextQuery { value, full_text }) if full_text => {
        result.push(ExternalQuery::FullText(FullTextQuery {
          fields: Some(vec![FIELD_NAME.to_owned()]),
          value,
        }));
      }
      _ => (),
    }

    if let Some(value) = value.description {
      result.push(ExternalQuery::FullText(FullTextQuery {
        fields: Some(vec![FIELD_DESCRIPTION.to_owned()]),
        value,
      }));
    }

    match value.tag {
      Some(TextQuery { value, full_text }) if full_text => {
        result.push(ExternalQuery::FullText(FullTextQuery {
          fields: Some(vec![FIELD_TAG.to_owned()]),
          value,
        }));
      }
      _ => (),
    }

    result
  }
}

pub struct RecordService {}

#[async_trait::async_trait]
impl AbstractReadService for RecordService {
  type Model = record::Model;
  type Entity = Record;
  type Presentation = record::Presentation;
  type PrimaryKey = record::PrimaryKey;
  type Query = RecordQuery;

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
                  .find_related(RecordTag)
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
    Record::find()
      .join(JoinType::LeftJoin, record_tag::Relation::Record.def().rev())
      .join(JoinType::LeftJoin, record_item::Relation::Record.def().rev())
      .group_by(record::Column::Id)
  }

  fn primary_field() -> record::Column {
    record::Column::Id
  }

  fn primary_value(model: &Self::Model) -> uuid::Uuid {
    model.id
  }

  fn sortable_field(field: &str) -> Option<record::Column> {
    match field {
      "journal" => Some(record::Column::JournalId),
      "name" => Some(record::Column::Name),
      "typ" => Some(record::Column::Typ),
      "date" => Some(record::Column::Date),
      _ => None,
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecordCommand {
  Create(RecordCommandCreate),
  Update(RecordCommandUpdate),
  Delete(#[serde(rename = "targetId")] uuid::Uuid),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordCommandCreate {
  #[serde(rename = "targetId")]
  pub target_id: Option<uuid::Uuid>,
  #[serde(rename = "journalId")]
  pub journal_id: uuid::Uuid,
  pub name: String,
  pub description: String,
  #[serde(rename = "type")]
  pub typ: Type,
  pub date: NaiveDate,
  pub tags: HashSet<String>,
  pub items: HashSet<record_item::Presentation>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordCommandUpdate {
  #[serde(rename = "targetId")]
  pub target_id: uuid::Uuid,
  pub name: Option<String>,
  pub description: Option<String>,
  #[serde(rename = "type")]
  pub typ: Option<Type>,
  pub date: Option<NaiveDate>,
  pub tags: Option<HashSet<String>>,
  pub items: Option<HashSet<record_item::Presentation>>,
}

impl RecordCommandUpdate {
  pub fn is_empty(&self) -> bool {
    self.description.is_none()
      && self.typ.is_none()
      && self.date.is_none()
      && self.tags.is_none()
      && self.items.is_none()
  }
}

impl AbstractCommand for RecordCommand {
  fn target_id(&self) -> Option<uuid::Uuid> {
    match self {
      RecordCommand::Create(RecordCommandCreate { target_id, .. }) => target_id.to_owned(),
      RecordCommand::Update(RecordCommandUpdate { target_id, .. }) => Some(*target_id),
      RecordCommand::Delete(id) => Some(*id),
    }
  }

  fn with_target_id(self, id: uuid::Uuid) -> Self {
    match self {
      RecordCommand::Create(command) => RecordCommand::Create(RecordCommandCreate {
        target_id: Some(id),
        ..command
      }),
      RecordCommand::Update(command) => RecordCommand::Update(RecordCommandUpdate {
        target_id: id,
        ..command
      }),
      RecordCommand::Delete(_) => RecordCommand::Delete(id),
    }
  }
}

impl RecordService {
  fn validate(
    model: &record::ActiveModel,
    tags: Option<&HashSet<String>>,
    unit_items_accounts: Option<(
      String,
      &HashSet<record_item::Presentation>,
      HashMap<uuid::Uuid, account::Model>,
    )>,
  ) -> crate::Result<()> {
    let mut errors = Vec::<Error>::new();

    match &model.name {
      ActiveValue::Set(name) if name.len() < MIN_NAME || name.len() > MAX_NAME => errors.push(Error::LengthRange {
        entity: record::TYPE.to_owned(),
        field: FIELD_NAME.to_owned(),
        min: MIN_NAME,
        max: MAX_NAME,
      }),
      _ => (),
    }

    match &model.description {
      ActiveValue::Set(description) if description.len() > MAX_DESCRIPTION => errors.push(Error::MaxLength {
        entity: record::TYPE.to_owned(),
        field: FIELD_DESCRIPTION.to_owned(),
        value: MAX_DESCRIPTION,
      }),
      _ => (),
    }

    if let Some(tags) = tags {
      errors.append(&mut Error::validate_tags(account::TYPE, tags));
    }

    if let Some((unit, items, accounts)) = unit_items_accounts {
      if items.len() < MIN_RECORD_ITEMS || items.len() > MAX_RECORD_ITEMS {
        errors.push(Error::LengthRange {
          entity: record::TYPE.to_owned(),
          field: FIELD_RECORD_ITEMS.to_owned(),
          min: MIN_RECORD_ITEMS,
          max: MAX_RECORD_ITEMS,
        })
      }
      let mut used_accounts = HashSet::<uuid::Uuid>::new();
      for item in items {
        if let Some(account) = accounts.get(&item.account_id) {
          if account.is_archived {
            errors.push(Error::ArchivedAccount { id: account.id })
          }

          if used_accounts.contains(&account.id) {
            errors.push(Error::DuplicateAccountsInRecord {
              id: model.id.clone().unwrap(),
              account: account.id,
            })
          } else {
            used_accounts.insert(account.id);
          }

          // 1. If type == Record,
          //   1. If item.account.unit == journal.unit,
          //     1. item.amount is REQUIRED, item.price is FORBIDDEN
          //   2. If NOT,
          //     1. item.amount is REQUIRED, item.price is REQUIRED
          // 2. If type == Check,
          //   .1 item.amount is REQUIRED, item.price is FORBIDDEN
          if model.typ.clone().unwrap() == record::Type::Record && account.unit.as_str() != unit {
            if item.price.is_none() {
              errors.push(Error::RecordItemMustContainPrice {
                id: model.id.clone().unwrap(),
                account: account.id,
              });
            }
          } else if item.price.is_some() {
            errors.push(Error::RecordItemForbidPrice {
              id: model.id.clone().unwrap(),
              account: account.id,
            });
          }
        } else {
          errors.push(Error::NotFound {
            entity: account::TYPE.to_owned(),
            field: FIELD_ID.to_owned(),
            value: item.account_id.to_string(),
          })
        }
      }
    }

    match errors.first() {
      Some(error) if errors.len() == 1 => Err(error.clone())?,
      Some(_) => Err(Error::Errors(errors))?,
      None => Ok(()),
    }
  }

  fn record_state(
    journal: journal::Model,
    items: Vec<record_item::Model>,
    accounts: HashMap<uuid::Uuid, account::Model>,
  ) -> crate::Result<record::RecordState> {
    let mut left = Decimal::ZERO;
    let mut right = Decimal::ZERO;
    for item in items {
      if let Some(account) = accounts.get(&item.account_id) {
        let price = if journal.unit == account.unit {
          Decimal::ONE
        } else if let Some(price) = item.price {
          price
        } else {
          return Err(Error::RecordItemMustContainPrice {
            id: journal.id,
            account: account.id,
          })?;
        };
        match account.typ {
          account::Type::Asset | account::Type::Expense => left += item.amount * price,
          _ => right += item.amount * price,
        };
      } else {
        return Err(Error::NotFound {
          entity: account::TYPE.to_owned(),
          field: FIELD_ID.to_owned(),
          value: item.account_id.to_string(),
        })?;
      }
    }
    Ok(record::RecordState::Record(if left != right {
      RecordStateItem::Invalid(left, right)
    } else {
      RecordStateItem::Valid(left)
    }))
  }

  async fn check_state(
    conn: &impl ConnectionTrait,
    model: &record::Model,
    items: Vec<record_item::Model>,
    accounts: HashSet<uuid::Uuid>,
  ) -> crate::Result<record::RecordState> {
    let record_items: HashMap<uuid::Uuid, Decimal> = RecordItem::find()
      .left_join(Record)
      .left_join(Account)
      .filter(
        record::Column::Typ
          .eq(record::Type::Record)
          .and(record::Column::Date.lte(model.date))
          .and(account::Column::Id.is_in(accounts)),
      )
      .all(conn)
      .await?
      .into_iter()
      .fold(HashMap::new(), |mut acc, item| {
        let key = item.account_id;
        let value = acc.get(&key).cloned().unwrap_or_default() + item.amount;
        acc.insert(key, value);
        acc
      });

    Ok(record::RecordState::Check(
      items
        .into_iter()
        .map(|item| {
          let account_id = item.account_id;
          let expected = item.amount;
          let actual = record_items.get(&account_id).cloned().unwrap_or_default();
          (
            account_id,
            if expected == actual {
              RecordStateItem::Valid(expected)
            } else {
              RecordStateItem::Invalid(expected, actual)
            },
          )
        })
        .collect(),
    ))
  }

  pub async fn state(conn: &impl ConnectionTrait, model: &record::Model) -> crate::Result<record::RecordState> {
    let journal = model
      .find_related(Journal)
      .one(conn)
      .await?
      .ok_or_else(|| Error::NotFound {
        entity: journal::TYPE.to_owned(),
        field: FIELD_ID.to_owned(),
        value: model.journal_id.to_string(),
      })?;
    let items = model.find_related(RecordItem).all(conn).await?;
    let accounts: HashSet<_> = items.iter().map(|item| item.account_id).collect();

    if model.typ == record::Type::Record {
      let accounts: HashMap<uuid::Uuid, account::Model> = Account::find()
        .filter(account::Column::Id.is_in(accounts))
        .all(conn)
        .await?
        .into_iter()
        .map(|account| (account.id, account))
        .collect();

      Self::record_state(journal, items, accounts)
    } else {
      Self::check_state(conn, model, items, accounts).await
    }
  }

  pub async fn create(
    conn: &impl ConnectionTrait,
    operator: &AuthUser,
    command: RecordCommandCreate,
  ) -> crate::Result<record::Model> {
    let journal = if let Some(journal) = JournalService::find_by_id(conn, operator, command.journal_id).await? {
      journal
    } else {
      return Err(Error::NotFound {
        entity: journal::TYPE.to_owned(),
        field: FIELD_ID.to_owned(),
        value: command.journal_id.to_string(),
      })?;
    };

    if journal
      .find_related(Record)
      .filter(record::Column::Name.eq(command.name.clone()))
      .count(conn)
      .await?
      > 0
    {
      return Err(Error::AlreadyExists {
        entity: record::TYPE.to_owned(),
        field: FIELD_NAME.to_owned(),
        value: command.name,
      })?;
    }

    let record = record::ActiveModel {
      id: Set(uuid::Uuid::new_v4()),
      journal_id: Set(journal.id),
      name: Set(command.name.clone()),
      description: Set(command.description.clone()),
      typ: Set(command.typ.clone()),
      date: Set(command.date),
    };

    let accounts: HashSet<_> = command.items.iter().map(|item| item.account_id).collect();
    let accounts: HashMap<_, _> = journal
      .find_related(Account)
      .filter(account::Column::Id.is_in(accounts))
      .all(conn)
      .await?
      .into_iter()
      .map(|account| (account.id, account))
      .collect();

    Self::validate(
      &record,
      Some(&command.tags),
      Some((journal.unit, &command.items, accounts)),
    )?;

    let record = record.insert(conn).await?;

    let tags: Vec<_> = command
      .tags
      .into_iter()
      .map(|tag| record_tag::ActiveModel {
        record_id: Set(record.id),
        tag: Set(tag),
      })
      .collect();
    if !tags.is_empty() {
      let _ = RecordTag::insert_many(tags).exec(conn).await?;
    }

    let items: Vec<_> = command
      .items
      .into_iter()
      .map(|item| record_item::ActiveModel {
        record_id: Set(record.id),
        account_id: Set(item.account_id),
        amount: Set(item.amount),
        price: Set(item.price),
      })
      .collect();
    if !items.is_empty() {
      let _ = RecordItem::insert_many(items).exec(conn).await?;
    }
    Ok(record)
  }

  pub async fn update(
    conn: &impl ConnectionTrait,
    operator: &user::Model,
    command: RecordCommandUpdate,
  ) -> crate::Result<record::Model> {
    let record = Record::find_by_id(command.target_id)
      .one(conn)
      .await?
      .ok_or_else(|| Error::NotFound {
        entity: record::TYPE.to_owned(),
        field: FIELD_ID.to_owned(),
        value: command.target_id.to_string(),
      })?;
    let journal = record
      .find_related(Journal)
      .one(conn)
      .await?
      .ok_or_else(|| Error::NotFound {
        entity: journal::TYPE.to_owned(),
        field: FIELD_ID.to_owned(),
        value: record.journal_id.to_string(),
      })?;

    Self::check_writeable(conn, operator, &record).await?;

    if command.is_empty() {
      return Ok(record);
    }

    let mut model = record::ActiveModel {
      id: Set(command.target_id),
      typ: Set(record.typ),
      ..Default::default()
    };

    if let Some(name) = command.name {
      if journal
        .find_related(Record)
        .filter(record::Column::Name.eq(name.clone()))
        .count(conn)
        .await?
        > 0
      {
        return Err(Error::AlreadyExists {
          entity: record::TYPE.to_owned(),
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

    if let Some(date) = command.date {
      model.date = Set(date);
    }

    let unit_items_accounts = if let Some(ref items) = command.items {
      let accounts: HashSet<_> = items.iter().map(|item| item.account_id).collect();
      let accounts: HashMap<_, _> = journal
        .find_related(Account)
        .filter(account::Column::Id.is_in(accounts))
        .all(conn)
        .await?
        .into_iter()
        .map(|account| (account.id, account))
        .collect();
      Some((journal.unit, items, accounts))
    } else {
      None
    };

    Self::validate(&model, command.tags.as_ref(), unit_items_accounts)?;

    if let Some(tags) = command.tags {
      let tags: Vec<_> = tags
        .into_iter()
        .map(|tag| record_tag::ActiveModel {
          record_id: Set(command.target_id),
          tag: Set(tag),
        })
        .collect();
      let _ = RecordTag::delete_many()
        .filter(record_tag::Column::RecordId.eq(command.target_id))
        .exec(conn)
        .await?;
      if !tags.is_empty() {
        let _ = RecordTag::insert_many(tags).exec(conn).await?;
      }
    }

    if let Some(items) = command.items {
      let items: Vec<_> = items
        .into_iter()
        .map(|item| record_item::ActiveModel {
          record_id: Set(record.id),
          account_id: Set(item.account_id),
          amount: Set(item.amount),
          price: Set(item.price),
        })
        .collect();
      let _ = RecordItem::delete_many()
        .filter(record_item::Column::RecordId.eq(record.id))
        .exec(conn)
        .await?;
      if !items.is_empty() {
        let _ = RecordItem::insert_many(items).exec(conn).await?;
      }
    }

    Ok(model.update(conn).await?)
  }

  pub async fn delete(conn: &impl ConnectionTrait, operator: &user::Model, id: uuid::Uuid) -> crate::Result<()> {
    let account = Record::find_by_id(id).one(conn).await?.ok_or_else(|| Error::NotFound {
      entity: record::TYPE.to_owned(),
      field: FIELD_ID.to_owned(),
      value: id.to_string(),
    })?;

    Self::check_writeable(conn, operator, &account).await?;

    let model = record::ActiveModel {
      id: Set(id),
      ..Default::default()
    };
    model.delete(conn).await?;
    Ok(())
  }
}

#[async_trait::async_trait]
impl AbstractWriteService for RecordService {
  type Command = RecordCommand;

  async fn check_writeable(conn: &impl ConnectionTrait, user: &user::Model, model: &Self::Model) -> crate::Result<()> {
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
    operator: &AuthUser,
    command: Self::Command,
  ) -> crate::Result<Option<Self::Model>> {
    if let AuthUser::User(user) = operator {
      match command {
        RecordCommand::Create(command) => {
          let result = Self::create(conn, operator, command).await?;
          Ok(Some(result))
        }
        RecordCommand::Update(command) => {
          let result = Self::update(conn, user, command).await?;
          Ok(Some(result))
        }
        RecordCommand::Delete(id) => {
          Self::delete(conn, user, id).await?;
          Ok(None)
        }
      }
    } else {
      Err(Error::InvalidPermission {
        user: operator.id(),
        entity: record::TYPE.to_owned(),
        id: command.target_id(),
        permission: Permission::Write,
      })?
    }
  }
}
