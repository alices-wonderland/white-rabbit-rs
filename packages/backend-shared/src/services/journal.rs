use crate::models::{
  journal::{self, AccessItem, AccessItemType},
  journal_group, journal_tag, journal_user, Journal,
};
use sea_orm::{
  sea_query::{Condition, IntoCondition, JoinType},
  ColumnTrait, EntityTrait, QuerySelect, RelationTrait, Select,
};
use serde::{Deserialize, Serialize};

use super::{
  read_service::{AbstractReadService, ContainingUserQuery, ExternalQuery, FullTextQuery, IdQuery, TextQuery},
  write_service::AbstractCommand,
  FIELD_DESCRIPTION, FIELD_NAME, FIELD_TAG,
};

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct JournalQuery {
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(default)]
  #[serde(rename = "__fullText")]
  pub full_text: Option<FullTextQuery>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(default)]
  #[serde(rename = "__containingUser")]
  pub containing_user: Option<ContainingUserQuery>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(default)]
  pub id: Option<IdQuery>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(default)]
  pub name: Option<TextQuery>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(default)]
  pub description: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(default)]
  pub unit: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(default)]
  pub tag: Option<TextQuery>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(default)]
  pub admins: Option<Vec<journal::AccessItem>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(default)]
  pub members: Option<Vec<journal::AccessItem>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(default)]
  pub include_archived: Option<bool>,
}

impl IntoCondition for JournalQuery {
  fn into_condition(self) -> Condition {
    let mut cond = Condition::all();

    if let Some(id) = self.id {
      cond = match id {
        IdQuery::Single(id) => cond.add(journal::Column::Id.eq(id)),
        IdQuery::Multiple(ids) => cond.add(journal::Column::Id.is_in(ids)),
      }
    }

    if let Some(TextQuery::Value(name)) = self.name {
      cond = cond.add(journal::Column::Name.eq(name));
    }

    if let Some(unit) = self.unit {
      cond = cond.add(journal::Column::Unit.eq(unit));
    }

    if let Some(TextQuery::Value(tag)) = self.tag {
      cond = cond.add(journal_tag::Column::Tag.eq(tag));
    }

    if let Some(admins) = self.admins {
      let groups: Vec<_> = admins
        .iter()
        .filter_map(|item| {
          if item.typ == AccessItemType::Group {
            Some(item.id)
          } else {
            None
          }
        })
        .collect();
      let groups = if groups.is_empty() {
        None
      } else {
        Some(journal_group::Column::GroupId.is_in(groups))
      };

      let users: Vec<_> = admins
        .iter()
        .filter_map(|item| {
          if item.typ == AccessItemType::User {
            Some(item.id)
          } else {
            None
          }
        })
        .collect();
      let users = if users.is_empty() {
        None
      } else {
        Some(journal_user::Column::UserId.is_in(users))
      };

      cond = cond.add(Condition::any().add_option(groups).add_option(users));
    }

    if !self.include_archived.unwrap_or(false) {
      cond = cond.add(journal::Column::IsArchived.eq(false));
    }

    cond
  }
}

impl From<JournalQuery> for Vec<ExternalQuery> {
  fn from(value: JournalQuery) -> Self {
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

pub struct JournalService {}

#[async_trait::async_trait]
impl AbstractReadService for JournalService {
  type Model = journal::Model;
  type Entity = Journal;
  type PrimaryKey = journal::PrimaryKey;
  type Query = JournalQuery;

  async fn filter_by_external_query(items: Vec<Self::Model>, external_query: &ExternalQuery) -> Vec<Self::Model> {
    items
      .into_iter()
      .filter(|item| match external_query {
        ExternalQuery::FullText(FullTextQuery { value, fields }) => {
          if let Some(fields) = fields {
            fields.iter().all(|field| match field.as_str() {
              FIELD_NAME => item.name.contains(value),
              FIELD_DESCRIPTION => item.description.contains(value),
              _ => true,
            })
          } else {
            item.name.contains(value) || item.description.contains(value)
          }
        }
        _ => true,
      })
      .collect()
  }

  fn find_all_select() -> Select<Self::Entity> {
    Journal::find()
      .join(JoinType::LeftJoin, journal_user::Relation::Journal.def().rev())
      .join(JoinType::LeftJoin, journal_user::Relation::User.def())
      .join(JoinType::LeftJoin, journal_group::Relation::Journal.def().rev())
      .join(JoinType::LeftJoin, journal_group::Relation::Group.def())
      .group_by(journal::Column::Id)
  }

  fn primary_field() -> journal::Column {
    journal::Column::Id
  }

  fn primary_value(model: &Self::Model) -> uuid::Uuid {
    model.id
  }

  fn sortable_field(field: &str) -> Option<journal::Column> {
    match field {
      "name" => Some(journal::Column::Name),
      "unit" => Some(journal::Column::Unit),
      "is_archived" => Some(journal::Column::IsArchived),
      _ => None,
    }
  }
}

pub enum JournalCommand {
  Create(JournalCommandCreate),
  Update(JournalCommandUpdate),
  Delete(uuid::Uuid),
}

pub struct JournalCommandCreate {
  pub target_id: Option<uuid::Uuid>,
  pub name: String,
  pub description: String,
  pub unit: String,
  pub tags: Vec<String>,
  pub admins: Vec<AccessItem>,
  pub members: Vec<AccessItem>,
}

pub struct JournalCommandUpdate {
  pub target_id: uuid::Uuid,
  pub name: Option<String>,
  pub description: Option<String>,
  pub unit: Option<String>,
  pub tags: Option<Vec<String>>,
  pub admins: Option<Vec<AccessItem>>,
  pub members: Option<Vec<AccessItem>>,
}

impl JournalCommandUpdate {
  pub fn is_empty(&self) -> bool {
    self.name.is_none()
      && self.description.is_none()
      && self.unit.is_none()
      && self.tags.is_none()
      && self.admins.is_none()
      && self.members.is_none()
  }
}

impl AbstractCommand for JournalCommand {
  fn target_id(&self) -> Option<uuid::Uuid> {
    match self {
      JournalCommand::Create(JournalCommandCreate { target_id, .. }) => target_id.to_owned(),
      JournalCommand::Update(JournalCommandUpdate { target_id, .. }) => Some(*target_id),
      JournalCommand::Delete(id) => Some(*id),
    }
  }

  fn with_target_id(self, id: uuid::Uuid) -> Self {
    match self {
      JournalCommand::Create(command) => JournalCommand::Create(JournalCommandCreate {
        target_id: Some(id),
        ..command
      }),
      JournalCommand::Update(command) => JournalCommand::Update(JournalCommandUpdate {
        target_id: id,
        ..command
      }),
      JournalCommand::Delete(_) => JournalCommand::Delete(id),
    }
  }
}
