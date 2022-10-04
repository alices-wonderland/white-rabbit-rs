use std::collections::HashSet;

use crate::{
  errors::Error,
  models::{
    group, journal, journal_group, journal_tag, journal_user, user, AccessItem, AccessItemType, Group, Journal,
    JournalGroup, JournalTag, JournalUser, User,
  },
};
use futures::{stream, StreamExt};
use sea_orm::{
  sea_query::{Condition, IntoCondition, JoinType},
  ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, EntityTrait, ModelTrait, PaginatorTrait, QueryFilter,
  QuerySelect, RelationTrait, Select, Set,
};
use serde::{Deserialize, Serialize};

use super::{
  group::GroupService,
  read_service::{AbstractReadService, ContainingUserQuery, ExternalQuery, FullTextQuery, IdQuery, TextQuery},
  user::UserService,
  write_service::{AbstractCommand, AbstractWriteService},
  AuthUser, Permission, FIELD_ADMINS, FIELD_DESCRIPTION, FIELD_ID, FIELD_MEMBERS, FIELD_NAME, FIELD_TAG,
  MAX_ACCESS_ITEM, MAX_DESCRIPTION, MAX_NAME, MIN_ADMIN, MIN_NAME,
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
  pub admins: Option<Vec<AccessItem>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(default)]
  pub members: Option<Vec<AccessItem>>,
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

    if let Some(value) = value.containing_user {
      result.push(ExternalQuery::ContainingUser(value));
    }

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
  type Presentation = journal::Presentation;
  type PrimaryKey = journal::PrimaryKey;
  type Query = JournalQuery;

  async fn is_readable(conn: &impl ConnectionTrait, operator: &AuthUser, model: &Self::Model) -> bool {
    if let AuthUser::User(operator) = operator {
      (operator.role > user::Role::User)
        || Self::contain_user(conn, model, operator.id, None)
          .await
          .unwrap_or(false)
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
          ExternalQuery::ContainingUser(query) => Self::contain_user(
            conn,
            &item,
            query.id(),
            match query {
              ContainingUserQuery::Object { fields, .. } => fields.clone(),
              _ => None,
            },
          )
          .await
          .ok()
          .and_then(|containing| if containing { Some(item) } else { None }),
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum JournalCommand {
  Create(JournalCommandCreate),
  Update(JournalCommandUpdate),
  Delete(uuid::Uuid),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct JournalCommandCreate {
  pub target_id: Option<uuid::Uuid>,
  pub name: String,
  pub description: String,
  pub unit: String,
  pub tags: HashSet<String>,
  pub admins: HashSet<AccessItem>,
  pub members: HashSet<AccessItem>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct JournalCommandUpdate {
  pub target_id: uuid::Uuid,
  pub name: Option<String>,
  pub description: Option<String>,
  pub unit: Option<String>,
  pub is_archived: Option<bool>,
  pub tags: Option<HashSet<String>>,
  pub admins: Option<HashSet<AccessItem>>,
  pub members: Option<HashSet<AccessItem>>,
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
  fn validate(
    model: &journal::ActiveModel,
    admin_users: &HashSet<user::Model>,
    member_users: &HashSet<user::Model>,
    admin_groups: &HashSet<group::Model>,
    member_groups: &HashSet<group::Model>,
    tags: Option<&HashSet<String>>,
  ) -> crate::Result<()> {
    let mut errors = Vec::<Error>::new();
    match &model.name {
      ActiveValue::Set(name) if name.len() < MIN_NAME || name.len() > MAX_NAME => errors.push(Error::LengthRange {
        entity: journal::TYPE.to_owned(),
        field: FIELD_NAME.to_owned(),
        min: MIN_NAME,
        max: MAX_NAME,
      }),
      _ => (),
    }

    match &model.description {
      ActiveValue::Set(description) if description.len() > MAX_DESCRIPTION => errors.push(Error::MaxLength {
        entity: journal::TYPE.to_owned(),
        field: FIELD_DESCRIPTION.to_owned(),
        value: MAX_DESCRIPTION,
      }),
      _ => (),
    }

    let admin_count = admin_users.len() + admin_groups.len();
    if !(MIN_ADMIN..=MAX_ACCESS_ITEM).contains(&admin_count) {
      errors.push(Error::LengthRange {
        entity: journal::TYPE.to_owned(),
        field: FIELD_ADMINS.to_owned(),
        min: MIN_ADMIN,
        max: MAX_ACCESS_ITEM,
      });
    }

    if member_users.len() + member_groups.len() > MAX_ACCESS_ITEM {
      errors.push(Error::MaxLength {
        entity: journal::TYPE.to_owned(),
        field: FIELD_MEMBERS.to_owned(),
        value: MAX_ACCESS_ITEM,
      });
    }

    for item in admin_users.intersection(member_users) {
      errors.push(Error::DuplicatedAccessItem {
        entity: journal::TYPE.to_owned(),
        id: model.id.clone().unwrap(),
        access_item_type: AccessItemType::User,
        access_item_id: item.id,
      });
    }

    for item in admin_groups.intersection(member_groups) {
      errors.push(Error::DuplicatedAccessItem {
        entity: journal::TYPE.to_owned(),
        id: model.id.clone().unwrap(),
        access_item_type: AccessItemType::Group,
        access_item_id: item.id,
      });
    }

    if let Some(tags) = tags {
      errors.append(&mut Error::validate_tags(journal::TYPE, tags));
    }

    match errors.first() {
      Some(error) if errors.len() == 1 => Err(error.clone())?,
      Some(_) => Err(Error::Errors(errors))?,
      None => Ok(()),
    }
  }

  pub async fn contain_user(
    conn: &impl ConnectionTrait,
    model: &journal::Model,
    user: uuid::Uuid,
    fields: Option<Vec<String>>,
  ) -> crate::Result<bool> {
    for field in fields.unwrap_or_else(|| vec![FIELD_ADMINS.to_owned(), FIELD_MEMBERS.to_owned()]) {
      if let Some((user_query, group_query)) = match field.as_str() {
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
        let users = user_query.all(conn).await?;
        if users.iter().any(|item| item.id == user) {
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

  async fn load_access_items(
    conn: &impl ConnectionTrait,
    operator: &user::Model,
    items: impl IntoIterator<Item = AccessItem>,
  ) -> crate::Result<(HashSet<user::Model>, HashSet<group::Model>)> {
    let (users, groups): (Vec<_>, Vec<_>) = items.into_iter().partition(|item| item.typ == AccessItemType::User);

    let users: Vec<_> = users.iter().map(|item| item.id).collect();
    let users: HashSet<_> = stream::iter(User::find().filter(user::Column::Id.is_in(users)).all(conn).await?)
      .filter_map(|user| async move {
        if UserService::is_readable(conn, &AuthUser::User(operator.clone()), &user).await {
          Some(user)
        } else {
          None
        }
      })
      .collect()
      .await;

    let groups: Vec<_> = groups.iter().map(|item| item.id).collect();
    let groups: HashSet<_> = stream::iter(Group::find().filter(group::Column::Id.is_in(groups)).all(conn).await?)
      .filter_map(|group| async move {
        if GroupService::is_readable(conn, &AuthUser::User(operator.clone()), &group).await {
          Some(group)
        } else {
          None
        }
      })
      .collect()
      .await;

    Ok((users, groups))
  }

  fn create_access_items(
    journal: uuid::Uuid,
    admin_users: HashSet<user::Model>,
    member_users: HashSet<user::Model>,
    admin_groups: HashSet<group::Model>,
    member_groups: HashSet<group::Model>,
  ) -> (Vec<journal_user::ActiveModel>, Vec<journal_group::ActiveModel>) {
    let users: Vec<_> = vec![(admin_users, true), (member_users, false)]
      .into_iter()
      .flat_map(|(items, is_admin)| {
        items.into_iter().map(move |item| journal_user::ActiveModel {
          journal_id: Set(journal),
          user_id: Set(item.id),
          is_admin: Set(is_admin),
        })
      })
      .collect();

    let groups: Vec<_> = vec![(admin_groups, true), (member_groups, false)]
      .into_iter()
      .flat_map(|(items, is_admin)| {
        items.into_iter().map(move |item| journal_group::ActiveModel {
          journal_id: Set(journal),
          group_id: Set(item.id),
          is_admin: Set(is_admin),
        })
      })
      .collect();

    (users, groups)
  }

  pub async fn create(
    conn: &impl ConnectionTrait,
    operator: &user::Model,
    command: JournalCommandCreate,
  ) -> crate::Result<journal::Model> {
    if Journal::find()
      .filter(journal::Column::Name.eq(command.name.clone()))
      .count(conn)
      .await?
      > 0
    {
      return Err(Error::AlreadyExists {
        entity: journal::TYPE.to_owned(),
        field: FIELD_NAME.to_owned(),
        value: command.name,
      })?;
    }

    let id = uuid::Uuid::new_v4();
    let journal = journal::ActiveModel {
      id: Set(id),
      name: Set(command.name.clone()),
      description: Set(command.description.clone()),
      unit: Set(command.unit.clone()),
      is_archived: Set(false),
    };

    let admins: HashSet<_> = if command.admins.is_empty() {
      HashSet::from_iter(vec![AccessItem {
        typ: AccessItemType::User,
        id: operator.id,
      }])
    } else {
      command.admins
    };
    let (admin_users, admin_groups) = Self::load_access_items(conn, operator, admins).await?;
    let (member_users, member_groups) = Self::load_access_items(conn, operator, command.members).await?;

    Self::validate(
      &journal,
      &admin_users,
      &member_users,
      &admin_groups,
      &member_groups,
      Some(&command.tags),
    )?;

    let tags: Vec<_> = command
      .tags
      .into_iter()
      .map(|tag| journal_tag::ActiveModel {
        journal_id: Set(id),
        tag: Set(tag),
      })
      .collect();

    let (users, groups) = Self::create_access_items(id, admin_users, member_users, admin_groups, member_groups);

    let journal = journal.insert(conn).await?;
    if !tags.is_empty() {
      let _ = JournalTag::insert_many(tags).exec(conn).await?;
    }
    if !users.is_empty() {
      let _ = JournalUser::insert_many(users).exec(conn).await?;
    }
    if !groups.is_empty() {
      let _ = JournalGroup::insert_many(groups).exec(conn).await?;
    }

    Ok(journal)
  }

  pub async fn update(
    conn: &impl ConnectionTrait,
    operator: &user::Model,
    command: JournalCommandUpdate,
  ) -> crate::Result<journal::Model> {
    let journal = Journal::find_by_id(command.target_id)
      .one(conn)
      .await?
      .ok_or_else(|| Error::NotFound {
        entity: journal::TYPE.to_owned(),
        field: FIELD_ID.to_owned(),
        value: command.target_id.to_string(),
      })?;

    Self::check_writeable(conn, operator, &journal).await?;

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
        return Err(Error::AlreadyExists {
          entity: journal::TYPE.to_owned(),
          field: FIELD_NAME.to_owned(),
          value: name,
        })?;
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

    let (admin_users, admin_groups) = if let Some(admins) = command.admins {
      Self::load_access_items(conn, operator, admins).await?
    } else {
      (
        journal
          .find_linked(journal::JournalUserAdmin)
          .all(conn)
          .await?
          .into_iter()
          .collect(),
        journal
          .find_linked(journal::JournalGroupAdmin)
          .all(conn)
          .await?
          .into_iter()
          .collect(),
      )
    };

    let (member_users, member_groups) = if let Some(members) = command.members {
      Self::load_access_items(conn, operator, members).await?
    } else {
      (
        journal
          .find_linked(journal::JournalUserMember)
          .all(conn)
          .await?
          .into_iter()
          .collect(),
        journal
          .find_linked(journal::JournalGroupMember)
          .all(conn)
          .await?
          .into_iter()
          .collect(),
      )
    };

    Self::validate(
      &model,
      &admin_users,
      &member_users,
      &admin_groups,
      &member_groups,
      command.tags.as_ref(),
    )?;

    if let Some(tags) = command.tags {
      let tags: Vec<_> = tags
        .into_iter()
        .map(|tag| journal_tag::ActiveModel {
          journal_id: Set(command.target_id),
          tag: Set(tag),
        })
        .collect();
      let _ = JournalTag::delete_many()
        .filter(journal_tag::Column::JournalId.eq(command.target_id))
        .exec(conn)
        .await?;
      if !tags.is_empty() {
        let _ = JournalTag::insert_many(tags).exec(conn).await?;
      }
    }

    let (users, groups) = Self::create_access_items(
      command.target_id,
      admin_users,
      member_users,
      admin_groups,
      member_groups,
    );
    if !users.is_empty() {
      let _ = JournalUser::delete_many()
        .filter(journal_user::Column::JournalId.eq(command.target_id))
        .exec(conn)
        .await?;
      let _ = JournalUser::insert_many(users).exec(conn).await?;
    }
    if !groups.is_empty() {
      let _ = JournalGroup::delete_many()
        .filter(journal_group::Column::JournalId.eq(command.target_id))
        .exec(conn)
        .await?;
      let _ = JournalGroup::insert_many(groups).exec(conn).await?;
    }
    Ok(model.update(conn).await?)
  }

  pub async fn delete(conn: &impl ConnectionTrait, operator: &user::Model, id: uuid::Uuid) -> crate::Result<()> {
    let journal = Journal::find_by_id(id)
      .one(conn)
      .await?
      .ok_or_else(|| Error::NotFound {
        entity: journal::TYPE.to_owned(),
        field: FIELD_ID.to_owned(),
        value: id.to_string(),
      })?;

    Self::check_writeable(conn, operator, &journal).await?;

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

  async fn check_writeable(conn: &impl ConnectionTrait, user: &user::Model, model: &Self::Model) -> crate::Result<()> {
    if user.role > user::Role::Admin {
      return Ok(());
    }

    if !Self::contain_user(conn, model, user.id, Some(vec![FIELD_ADMINS.to_owned()])).await? {
      return Err(Error::InvalidPermission {
        user: user.id.to_string(),
        entity: journal::TYPE.to_owned(),
        id: Some(model.id),
        permission: Permission::Write,
      })?;
    }

    Ok(())
  }

  async fn handle(
    conn: &impl ConnectionTrait,
    operator: &AuthUser,
    command: Self::Command,
  ) -> crate::Result<Option<Self::Model>> {
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
      return Err(Error::InvalidPermission {
        user: operator.id(),
        entity: journal::TYPE.to_owned(),
        id: command.target_id(),
        permission: Permission::Write,
      })?;
    }
  }
}
