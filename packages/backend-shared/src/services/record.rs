use chrono::NaiveDate;
use futures::{stream, StreamExt};
use rust_decimal::Decimal;
use sea_orm::{
  sea_query::IntoCondition, ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, EntityTrait, JoinType,
  ModelTrait, PaginatorTrait, QueryFilter, QuerySelect, RelationTrait, Select, Set,
};
use serde::{Deserialize, Serialize};

use crate::models::{
  record::{self, Type},
  record_item, record_tag, user, Journal, Record, RecordItem, RecordTag,
};

use super::{
  read_service::{AbstractReadService, ComparableQuery, ExternalQuery, FullTextQuery, IdQuery, TextQuery},
  write_service::{AbstractCommand, AbstractWriteService},
  AuthUser, JournalService, FIELD_DESCRIPTION, FIELD_NAME, FIELD_TAG,
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

    if let Some(TextQuery::Value(name)) = self.name {
      cond = cond.add(record::Column::Name.eq(name));
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

    if let Some(TextQuery::Value(tag)) = self.tag {
      cond = cond.add(record_tag::Column::Tag.eq(tag));
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

pub struct RecordService {}

#[async_trait::async_trait]
impl AbstractReadService for RecordService {
  type Model = record::Model;
  type Entity = Record;
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
      "name" => Some(record::Column::Name),
      "journal" => Some(record::Column::JournalId),
      "typ" => Some(record::Column::Typ),
      "date" => Some(record::Column::Date),
      _ => None,
    }
  }
}

pub enum RecordCommand {
  Create(RecordCommandCreate),
  Update(RecordCommandUpdate),
  Delete(uuid::Uuid),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordItemValue {
  pub account_id: uuid::Uuid,
  pub amount: Option<Decimal>,
  pub price: Option<Decimal>,
}

pub struct RecordCommandCreate {
  pub target_id: Option<uuid::Uuid>,
  pub journal_id: uuid::Uuid,
  pub name: String,
  pub description: String,
  pub typ: Type,
  pub date: NaiveDate,
  pub tags: Vec<String>,
  pub items: Vec<RecordItemValue>,
}

pub struct RecordCommandUpdate {
  pub target_id: uuid::Uuid,
  pub name: Option<String>,
  pub description: Option<String>,
  pub typ: Option<Type>,
  pub date: Option<NaiveDate>,
  pub tags: Option<Vec<String>>,
  pub items: Option<Vec<RecordItemValue>>,
}

impl RecordCommandUpdate {
  pub fn is_empty(&self) -> bool {
    self.name.is_none()
      && self.description.is_none()
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
  pub async fn create(
    conn: &impl ConnectionTrait,
    operator: user::Model,
    command: RecordCommandCreate,
  ) -> anyhow::Result<record::Model> {
    if Record::find()
      .filter(record::Column::Name.eq(command.name.clone()))
      .count(conn)
      .await?
      > 0
    {
      return Err(anyhow::Error::msg("Record name exists"));
    }

    let journal =
      if let Some(journal) = JournalService::find_by_id(conn, AuthUser::User(operator), command.journal_id).await? {
        journal
      } else {
        return Err(anyhow::Error::msg("Journal not exists"));
      };

    let record = record::ActiveModel {
      id: Set(uuid::Uuid::new_v4()),
      journal_id: Set(journal.id),
      name: Set(command.name.clone()),
      description: Set(command.description.clone()),
      typ: Set(command.typ.clone()),
      date: Set(command.date),
    };
    let record = record.insert(conn).await?;

    let tags: Vec<_> = command
      .tags
      .into_iter()
      .map(|tag| record_tag::ActiveModel {
        record_id: Set(record.id),
        tag: Set(tag),
      })
      .collect();
    let _ = RecordTag::insert_many(tags).exec(conn).await?;

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
    let _ = RecordItem::insert_many(items).exec(conn).await?;

    Ok(record)
  }

  pub async fn update(
    conn: &impl ConnectionTrait,
    operator: user::Model,
    command: RecordCommandUpdate,
  ) -> anyhow::Result<record::Model> {
    let record = Record::find_by_id(command.target_id)
      .one(conn)
      .await?
      .ok_or_else(|| anyhow::Error::msg("Not found"))?;

    Self::check_writeable(conn, &operator, &record).await?;

    if command.is_empty() {
      return Ok(record);
    }

    let mut model = record::ActiveModel {
      id: Set(command.target_id),
      ..Default::default()
    };

    if let Some(name) = command.name {
      if Record::find()
        .filter(record::Column::Name.eq(name.clone()))
        .count(conn)
        .await?
        > 0
      {
        return Err(anyhow::Error::msg("Account name exists"));
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

    let tags: Vec<_> = command
      .tags
      .unwrap_or_default()
      .into_iter()
      .map(|tag| record_tag::ActiveModel {
        record_id: Set(record.id),
        tag: Set(tag),
      })
      .collect();
    if !tags.is_empty() {
      let _ = RecordTag::delete_many()
        .filter(record_tag::Column::RecordId.eq(record.id))
        .exec(conn)
        .await?;
      let _ = RecordTag::insert_many(tags).exec(conn).await?;
    }

    let items: Vec<_> = command
      .items
      .unwrap_or_default()
      .into_iter()
      .map(|item| record_item::ActiveModel {
        record_id: Set(record.id),
        account_id: Set(item.account_id),
        amount: Set(item.amount),
        price: Set(item.price),
      })
      .collect();
    if !items.is_empty() {
      let _ = RecordItem::delete_many()
        .filter(record_item::Column::RecordId.eq(record.id))
        .exec(conn)
        .await?;
      let _ = RecordItem::insert_many(items).exec(conn).await?;
    }

    Ok(model.update(conn).await?)
  }

  pub async fn delete(conn: &impl ConnectionTrait, operator: user::Model, id: uuid::Uuid) -> anyhow::Result<()> {
    let account = Record::find_by_id(id)
      .one(conn)
      .await?
      .ok_or_else(|| anyhow::Error::msg("Not found"))?;

    Self::check_writeable(conn, &operator, &account).await?;

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

  async fn check_writeable(conn: &impl ConnectionTrait, user: &user::Model, model: &Self::Model) -> anyhow::Result<()> {
    if user.role > user::Role::Admin {
      Ok(())
    } else if let Some(journal) = Journal::find_by_id(model.journal_id).one(conn).await? {
      JournalService::check_writeable(conn, user, &journal).await
    } else {
      Err(anyhow::Error::msg("Journal not found"))
    }
  }

  async fn handle(
    conn: &impl ConnectionTrait,
    operator: AuthUser,
    command: Self::Command,
  ) -> anyhow::Result<Option<Self::Model>> {
    if let AuthUser::User(operator) = operator {
      match command {
        RecordCommand::Create(command) => {
          let result = Self::create(conn, operator, command).await?;
          Ok(Some(result))
        }
        RecordCommand::Update(command) => {
          let result = Self::update(conn, operator, command).await?;
          Ok(Some(result))
        }
        RecordCommand::Delete(id) => {
          Self::delete(conn, operator, id).await?;
          Ok(None)
        }
      }
    } else {
      Err(anyhow::Error::msg("Please login"))
    }
  }
}
