use crate::models::{
  group,
  journal::{self, AccessItem, AccessItemType},
  journal_group, journal_tag, journal_user, user, Group, Journal, JournalGroup, JournalTag, JournalUser, User,
};
use futures::{stream, StreamExt};
use sea_orm::{
  sea_query::{Condition, IntoCondition, JoinType},
  ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, ModelTrait, PaginatorTrait, QueryFilter, QuerySelect,
  RelationTrait, Select, Set,
};
use serde::{Deserialize, Serialize};

use super::{
  group::GroupService,
  read_service::{AbstractReadService, ContainingUserQuery, ExternalQuery, FullTextQuery, IdQuery, TextQuery},
  user::UserService,
  write_service::{AbstractCommand, AbstractWriteService},
  AuthUser, FIELD_ADMINS, FIELD_DESCRIPTION, FIELD_MEMBERS, FIELD_NAME, FIELD_TAG,
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
        Some(
          journal_group::Column::GroupId
            .is_in(groups)
            .and(journal_group::Column::IsAdmin.eq(true)),
        )
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
        Some(
          journal_user::Column::UserId
            .is_in(users)
            .and(journal_group::Column::IsAdmin.eq(true)),
        )
      };

      cond = cond.add(Condition::any().add_option(groups).add_option(users));
    }

    if let Some(members) = self.members {
      let groups: Vec<_> = members
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
        Some(
          journal_group::Column::GroupId
            .is_in(groups)
            .and(journal_group::Column::IsAdmin.eq(false)),
        )
      };

      let users: Vec<_> = members
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
        Some(
          journal_user::Column::UserId
            .is_in(users)
            .and(journal_group::Column::IsAdmin.eq(false)),
        )
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

  async fn is_readable(conn: &impl ConnectionTrait, operator: &AuthUser, model: &Self::Model) -> bool {
    if let AuthUser::User(operator) = operator {
      (operator.role > user::Role::User) || Self::contain_user(conn, model, operator, None).await.unwrap_or(false)
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
                  .find_related(JournalTag)
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
  pub is_archived: Option<bool>,
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

impl JournalService {
  pub async fn contain_user(
    conn: &impl ConnectionTrait,
    model: &journal::Model,
    user: &user::Model,
    fields: Option<Vec<&str>>,
  ) -> anyhow::Result<bool> {
    for field in fields.unwrap_or_else(|| vec![FIELD_ADMINS, FIELD_MEMBERS]) {
      if let Some((user_query, group_query)) = match field {
        FIELD_ADMINS => Some((
          model.find_linked(journal::JournalUserAdmin),
          model.find_linked(journal::JournalGroupAdmin),
        )),
        FIELD_MEMBERS => Some((
          model.find_linked(journal::JournalUserMember),
          model.find_linked(journal::JournalGroupMember),
        )),
        _ => None,
      } {
        if user_query.all(conn).await?.contains(user) {
          return Ok(true);
        }

        for group in group_query.all(conn).await? {
          if GroupService::contain_user(conn, &group, user, None).await? {
            return Ok(true);
          }
        }
      }
    }

    Ok(false)
  }

  async fn load_users(
    conn: &impl ConnectionTrait,
    operator: &user::Model,
    journal: &journal::Model,
    admins: &[AccessItem],
    members: &[AccessItem],
  ) -> anyhow::Result<Vec<journal_user::ActiveModel>> {
    let admins: Vec<_> = admins
      .iter()
      .filter_map(|item| match item.typ {
        AccessItemType::User => Some(item.id),
        AccessItemType::Group => None,
      })
      .collect();
    let admins: Vec<_> = stream::iter(User::find().filter(user::Column::Id.is_in(admins)).all(conn).await?)
      .filter_map(|user| async move {
        if UserService::is_readable(conn, &AuthUser::User(operator.clone()), &user).await {
          Some(journal_user::ActiveModel {
            journal_id: Set(journal.id),
            user_id: Set(user.id),
            is_admin: Set(true),
          })
        } else {
          None
        }
      })
      .collect()
      .await;

    let members: Vec<_> = members
      .iter()
      .filter_map(|item| match item.typ {
        AccessItemType::User => Some(item.id),
        AccessItemType::Group => None,
      })
      .collect();
    let members: Vec<_> = stream::iter(User::find().filter(user::Column::Id.is_in(members)).all(conn).await?)
      .filter_map(|user| async move {
        if UserService::is_readable(conn, &AuthUser::User(operator.clone()), &user).await {
          Some(journal_user::ActiveModel {
            journal_id: Set(journal.id),
            user_id: Set(user.id),
            is_admin: Set(false),
          })
        } else {
          None
        }
      })
      .collect()
      .await;

    Ok(vec![admins, members].into_iter().flatten().collect())
  }

  async fn load_groups(
    conn: &impl ConnectionTrait,
    operator: &user::Model,
    journal: &journal::Model,
    admins: &[AccessItem],
    members: &[AccessItem],
  ) -> anyhow::Result<Vec<journal_group::ActiveModel>> {
    let admins: Vec<_> = admins
      .iter()
      .filter_map(|item| match item.typ {
        AccessItemType::Group => Some(item.id),
        AccessItemType::User => None,
      })
      .collect();
    let admins: Vec<_> = stream::iter(Group::find().filter(group::Column::Id.is_in(admins)).all(conn).await?)
      .filter_map(|group| async move {
        if GroupService::is_readable(conn, &AuthUser::User(operator.clone()), &group).await {
          Some(journal_group::ActiveModel {
            journal_id: Set(journal.id),
            group_id: Set(group.id),
            is_admin: Set(true),
          })
        } else {
          None
        }
      })
      .collect()
      .await;

    let members: Vec<_> = members
      .iter()
      .filter_map(|item| match item.typ {
        AccessItemType::Group => Some(item.id),
        AccessItemType::User => None,
      })
      .collect();
    let members: Vec<_> = stream::iter(Group::find().filter(group::Column::Id.is_in(members)).all(conn).await?)
      .filter_map(|group| async move {
        if GroupService::is_readable(conn, &AuthUser::User(operator.clone()), &group).await {
          Some(journal_group::ActiveModel {
            journal_id: Set(journal.id),
            group_id: Set(group.id),
            is_admin: Set(false),
          })
        } else {
          None
        }
      })
      .collect()
      .await;

    Ok(vec![admins, members].into_iter().flatten().collect())
  }

  pub async fn create(
    conn: &impl ConnectionTrait,
    operator: user::Model,
    command: JournalCommandCreate,
  ) -> anyhow::Result<journal::Model> {
    if Journal::find()
      .filter(journal::Column::Name.eq(command.name.clone()))
      .count(conn)
      .await?
      > 0
    {
      return Err(anyhow::Error::msg("Journal name exists"));
    }

    let journal = journal::ActiveModel {
      id: Set(uuid::Uuid::new_v4()),
      name: Set(command.name.clone()),
      description: Set(command.description.clone()),
      unit: Set(command.unit.clone()),
      is_archived: Set(false),
    };
    let journal = journal.insert(conn).await?;

    let tags: Vec<_> = command
      .tags
      .into_iter()
      .map(|tag| journal_tag::ActiveModel {
        journal_id: Set(journal.id),
        tag: Set(tag),
      })
      .collect();
    let _ = JournalTag::insert_many(tags).exec(conn).await?;

    let users = Self::load_users(conn, &operator, &journal, &command.admins, &command.members).await?;
    let _ = JournalUser::insert_many(users).exec(conn).await?;

    let groups = Self::load_groups(conn, &operator, &journal, &command.admins, &command.members).await?;
    let _ = JournalGroup::insert_many(groups).exec(conn).await?;

    Ok(journal)
  }

  pub async fn update(
    conn: &impl ConnectionTrait,
    operator: user::Model,
    command: JournalCommandUpdate,
  ) -> anyhow::Result<journal::Model> {
    let journal = Journal::find_by_id(command.target_id)
      .one(conn)
      .await?
      .ok_or_else(|| anyhow::Error::msg("Not found"))?;

    Self::check_writeable(conn, &operator, &journal).await?;

    if command.is_empty() {
      return Ok(journal);
    }

    let mut model = journal::ActiveModel {
      id: Set(command.target_id),
      ..Default::default()
    };

    if let Some(name) = command.name {
      if Journal::find()
        .filter(journal::Column::Name.eq(name.clone()))
        .count(conn)
        .await?
        > 0
      {
        return Err(anyhow::Error::msg("Group name exists"));
      }

      model.name = Set(name);
    }

    if let Some(description) = command.description {
      model.description = Set(description);
    }

    if let Some(unit) = command.unit {
      model.unit = Set(unit);
    }

    if let Some(is_archived) = command.is_archived {
      model.is_archived = Set(is_archived);
    }

    let tags: Vec<_> = command
      .tags
      .unwrap_or_default()
      .into_iter()
      .map(|tag| journal_tag::ActiveModel {
        journal_id: Set(journal.id),
        tag: Set(tag),
      })
      .collect();
    if !tags.is_empty() {
      let _ = JournalTag::delete_many()
        .filter(journal_tag::Column::JournalId.eq(journal.id))
        .exec(conn)
        .await?;
      let _ = JournalTag::insert_many(tags).exec(conn).await?;
    }

    let admins = command.admins.unwrap_or_default();
    let members = command.members.unwrap_or_default();

    let users = Self::load_users(conn, &operator, &journal, &admins, &members).await?;
    if !users.is_empty() {
      let _ = JournalUser::delete_many()
        .filter(journal_user::Column::JournalId.eq(journal.id))
        .exec(conn)
        .await?;
      let _ = JournalUser::insert_many(users).exec(conn).await?;
    }

    let groups = Self::load_groups(conn, &operator, &journal, &admins, &members).await?;
    if !groups.is_empty() {
      let _ = JournalGroup::delete_many()
        .filter(journal_group::Column::JournalId.eq(journal.id))
        .exec(conn)
        .await?;
      let _ = JournalGroup::insert_many(groups).exec(conn).await?;
    }

    Ok(model.update(conn).await?)
  }

  pub async fn delete(conn: &impl ConnectionTrait, operator: user::Model, id: uuid::Uuid) -> anyhow::Result<()> {
    let journal = Journal::find_by_id(id)
      .one(conn)
      .await?
      .ok_or_else(|| anyhow::Error::msg("Not found"))?;

    Self::check_writeable(conn, &operator, &journal).await?;

    let model = group::ActiveModel {
      id: Set(id),
      ..Default::default()
    };
    model.delete(conn).await?;
    Ok(())
  }
}

#[async_trait::async_trait]
impl AbstractWriteService for JournalService {
  type Command = JournalCommand;

  async fn check_writeable(conn: &impl ConnectionTrait, user: &user::Model, model: &Self::Model) -> anyhow::Result<()> {
    if user.role > user::Role::Admin {
      return Ok(());
    }

    if !Self::contain_user(conn, model, user, Some(vec![FIELD_ADMINS])).await? {
      return Err(anyhow::Error::msg("No permission"));
    }

    Ok(())
  }

  async fn handle(
    conn: &impl ConnectionTrait,
    operator: AuthUser,
    command: Self::Command,
  ) -> anyhow::Result<Option<Self::Model>> {
    if let AuthUser::User(operator) = operator {
      match command {
        JournalCommand::Create(command) => {
          let result = Self::create(conn, operator, command).await?;
          Ok(Some(result))
        }
        JournalCommand::Update(command) => {
          let result = Self::update(conn, operator, command).await?;
          Ok(Some(result))
        }
        JournalCommand::Delete(id) => {
          Self::delete(conn, operator, id).await?;
          Ok(None)
        }
      }
    } else {
      Err(anyhow::Error::msg("Please login"))
    }
  }
}
