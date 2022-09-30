use std::collections::HashSet;

use sea_orm::{
  sea_query::{Condition, IntoCondition, JoinType},
  ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, EntityTrait, ModelTrait, PaginatorTrait, QueryFilter,
  QuerySelect, RelationTrait, Select, Set,
};
use serde::{Deserialize, Serialize};

use crate::{
  errors::Error,
  models::{group, group_user, user, AccessItemType, Group, GroupUser, User},
};

use super::{
  read_service::{AbstractReadService, ContainingUserQuery, ExternalQuery, FullTextQuery, IdQuery, TextQuery},
  write_service::{AbstractCommand, AbstractWriteService},
  AuthUser, Permission, FIELD_ADMINS, FIELD_DESCRIPTION, FIELD_ID, FIELD_MEMBERS, FIELD_NAME, MAX_ACCESS_ITEM,
  MAX_DESCRIPTION, MAX_NAME, MIN_NAME,
};

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct GroupQuery {
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
  pub admins: Option<Vec<uuid::Uuid>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(default)]
  pub members: Option<Vec<uuid::Uuid>>,
}

impl IntoCondition for GroupQuery {
  fn into_condition(self) -> Condition {
    let mut cond = Condition::all();

    if let Some(id) = self.id {
      cond = match id {
        IdQuery::Single(id) => cond.add(group::Column::Id.eq(id)),
        IdQuery::Multiple(ids) => cond.add(group::Column::Id.is_in(ids)),
      }
    }

    if let Some(TextQuery::Value(name)) = self.name {
      cond = cond.add(group::Column::Name.eq(name));
    }

    if let Some(containing_user) = self.containing_user {
      let mut sub_cond = Condition::all().add(group_user::Column::UserId.eq(containing_user.id()));

      if let ContainingUserQuery::Object {
        fields: Some(fields), ..
      } = containing_user
      {
        for field in fields {
          sub_cond = sub_cond.add_option(match field.as_str() {
            FIELD_ADMINS => Some(group_user::Column::IsAdmin.eq(true)),
            FIELD_MEMBERS => Some(group_user::Column::IsAdmin.eq(false)),
            _ => None,
          });
        }
      };

      cond = cond.add(sub_cond);
    }

    if let Some(admins) = self.admins {
      cond = cond.add(
        group_user::Column::UserId
          .is_in(admins)
          .and(group_user::Column::IsAdmin.eq(true)),
      );
    }

    if let Some(members) = self.members {
      cond = cond.add(
        group_user::Column::UserId
          .is_in(members)
          .and(group_user::Column::IsAdmin.eq(false)),
      );
    }

    cond
  }
}

impl From<GroupQuery> for Vec<ExternalQuery> {
  fn from(value: GroupQuery) -> Self {
    let mut result = Vec::new();

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

    result
  }
}

pub struct GroupService {}

#[async_trait::async_trait]
impl AbstractReadService for GroupService {
  type Model = group::Model;
  type Entity = Group;
  type PrimaryKey = group::PrimaryKey;
  type Query = GroupQuery;

  async fn is_readable(conn: &impl ConnectionTrait, operator: &AuthUser, model: &Self::Model) -> bool {
    if let AuthUser::User(operator) = operator {
      (operator.role > user::Role::User) || Self::contain_user(conn, model, operator, None).await.unwrap_or(false)
    } else {
      false
    }
  }

  async fn filter_by_external_query(
    _: &impl ConnectionTrait,
    items: Vec<Self::Model>,
    external_query: &ExternalQuery,
  ) -> Vec<Self::Model> {
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
    Group::find()
      .join(JoinType::LeftJoin, group::Relation::User.def())
      .group_by(group::Column::Id)
  }

  fn primary_field() -> group::Column {
    group::Column::Id
  }

  fn primary_value(model: &Self::Model) -> uuid::Uuid {
    model.id
  }

  fn sortable_field(field: &str) -> Option<group::Column> {
    match field {
      "name" => Some(group::Column::Name),
      _ => None,
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GroupCommand {
  Create(GroupCommandCreate),
  Update(GroupCommandUpdate),
  Delete(uuid::Uuid),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroupCommandCreate {
  pub target_id: Option<uuid::Uuid>,
  pub name: String,
  pub description: String,
  pub admins: Vec<uuid::Uuid>,
  pub members: Vec<uuid::Uuid>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroupCommandUpdate {
  pub target_id: uuid::Uuid,
  pub name: Option<String>,
  pub description: Option<String>,
  pub admins: Option<Vec<uuid::Uuid>>,
  pub members: Option<Vec<uuid::Uuid>>,
}

impl GroupCommandUpdate {
  pub fn is_empty(&self) -> bool {
    self.name.is_none() && self.description.is_none() && self.admins.is_none() && self.members.is_none()
  }
}

impl AbstractCommand for GroupCommand {
  fn target_id(&self) -> Option<uuid::Uuid> {
    match self {
      GroupCommand::Create(GroupCommandCreate { target_id, .. }) => target_id.to_owned(),
      GroupCommand::Update(GroupCommandUpdate { target_id, .. }) => Some(*target_id),
      GroupCommand::Delete(id) => Some(*id),
    }
  }

  fn with_target_id(self, id: uuid::Uuid) -> Self {
    match self {
      GroupCommand::Create(command) => GroupCommand::Create(GroupCommandCreate {
        target_id: Some(id),
        ..command
      }),
      GroupCommand::Update(command) => GroupCommand::Update(GroupCommandUpdate {
        target_id: id,
        ..command
      }),
      GroupCommand::Delete(_) => GroupCommand::Delete(id),
    }
  }
}

impl GroupService {
  fn validate(
    model: &group::ActiveModel,
    admins: &HashSet<user::Model>,
    members: &HashSet<user::Model>,
  ) -> anyhow::Result<()> {
    let mut errors = Vec::<Error>::new();
    match &model.name {
      ActiveValue::Set(name) if name.len() < MIN_NAME || name.len() > MAX_NAME => errors.push(Error::LengthRange {
        entity: group::TYPE.to_owned(),
        field: FIELD_NAME.to_owned(),
        min: MIN_NAME,
        max: MAX_NAME,
      }),
      _ => (),
    }

    match &model.description {
      ActiveValue::Set(description) if description.len() > MAX_DESCRIPTION => errors.push(Error::MaxLength {
        entity: group::TYPE.to_owned(),
        field: FIELD_DESCRIPTION.to_owned(),
        value: MAX_DESCRIPTION,
      }),
      _ => (),
    }

    if admins.len() > MAX_ACCESS_ITEM {
      errors.push(Error::MaxLength {
        entity: group::TYPE.to_owned(),
        field: FIELD_ADMINS.to_owned(),
        value: MAX_ACCESS_ITEM,
      });
    }

    if members.len() > MAX_ACCESS_ITEM {
      errors.push(Error::MaxLength {
        entity: group::TYPE.to_owned(),
        field: FIELD_MEMBERS.to_owned(),
        value: MAX_ACCESS_ITEM,
      });
    }

    for item in admins.intersection(members) {
      errors.push(Error::DuplicatedAccessItem {
        entity: group::TYPE.to_owned(),
        id: model.id.clone().unwrap(),
        access_item_type: AccessItemType::User,
        access_item_id: item.id,
      });
    }

    match errors.first() {
      Some(error) if errors.len() == 1 => Err(error.clone())?,
      Some(_) => Err(Error::Errors(errors))?,
      None => Ok(()),
    }
  }

  async fn load_access_items(
    conn: &impl ConnectionTrait,
    users: impl IntoIterator<Item = uuid::Uuid>,
  ) -> anyhow::Result<HashSet<user::Model>> {
    Ok(
      User::find()
        .filter(user::Column::Id.is_in(users))
        .all(conn)
        .await?
        .into_iter()
        .collect(),
    )
  }

  pub async fn contain_user(
    conn: &impl ConnectionTrait,
    model: &group::Model,
    user: &user::Model,
    fields: Option<Vec<&str>>,
  ) -> anyhow::Result<bool> {
    for field in fields.unwrap_or_else(|| vec![FIELD_ADMINS, FIELD_MEMBERS]) {
      if let Some(query) = match field {
        FIELD_ADMINS => Some(model.find_linked(group::GroupAdmin)),
        FIELD_MEMBERS => Some(model.find_linked(group::GroupMember)),
        _ => None,
      } {
        if query.all(conn).await?.contains(user) {
          return Ok(true);
        }
      }
    }

    Ok(false)
  }

  pub async fn create(
    conn: &impl ConnectionTrait,
    operator: &user::Model,
    command: GroupCommandCreate,
  ) -> anyhow::Result<group::Model> {
    if Group::find()
      .filter(group::Column::Name.eq(command.name.clone()))
      .count(conn)
      .await?
      > 0
    {
      return Err(Error::AlreadyExists {
        entity: group::TYPE.to_owned(),
        field: FIELD_NAME.to_owned(),
        value: command.name,
      })?;
    }

    let id = uuid::Uuid::new_v4();
    let model = group::ActiveModel {
      id: Set(id),
      name: Set(command.name),
      description: Set(command.description),
    };

    let admins: HashSet<_> = vec![command.admins, vec![operator.id]].into_iter().flatten().collect();
    let admins = Self::load_access_items(conn, admins).await?;
    let members = Self::load_access_items(conn, command.members).await?;
    Self::validate(&model, &admins, &members)?;

    let admins: Vec<_> = admins
      .into_iter()
      .map(|user| group_user::ActiveModel {
        group_id: Set(id),
        user_id: Set(user.id),
        is_admin: Set(true),
      })
      .collect();
    let members: Vec<_> = members
      .into_iter()
      .map(|user| group_user::ActiveModel {
        group_id: Set(id),
        user_id: Set(user.id),
        is_admin: Set(false),
      })
      .collect();

    let model = model.insert(conn).await?;
    let users: Vec<_> = vec![admins, members].into_iter().flatten().collect();
    let _ = GroupUser::insert_many(users).exec(conn).await?;

    Ok(model)
  }

  pub async fn update(
    conn: &impl ConnectionTrait,
    operator: &user::Model,
    command: GroupCommandUpdate,
  ) -> anyhow::Result<group::Model> {
    let group = Group::find_by_id(command.target_id)
      .one(conn)
      .await?
      .ok_or_else(|| Error::NotFound {
        entity: group::TYPE.to_owned(),
        field: FIELD_ID.to_owned(),
        value: command.target_id.to_string(),
      })?;

    Self::check_writeable(conn, operator, &group).await?;

    if command.is_empty() {
      return Ok(group);
    }

    let mut model = group::ActiveModel {
      id: Set(command.target_id),
      ..Default::default()
    };

    if let Some(name) = command.name {
      if Group::find()
        .filter(group::Column::Name.eq(name.clone()))
        .count(conn)
        .await?
        > 0
      {
        return Err(Error::AlreadyExists {
          entity: group::TYPE.to_owned(),
          field: FIELD_NAME.to_owned(),
          value: name,
        })?;
      }

      model.name = Set(name);
    }

    if let Some(description) = command.description {
      model.description = Set(description);
    }

    let admins: HashSet<_> = if let Some(admins) = command.admins {
      Self::load_access_items(conn, admins).await?
    } else {
      group
        .find_linked(group::GroupAdmin)
        .all(conn)
        .await?
        .into_iter()
        .collect()
    };

    let members: HashSet<_> = if let Some(members) = command.members {
      Self::load_access_items(conn, members).await?
    } else {
      group
        .find_linked(group::GroupMember)
        .all(conn)
        .await?
        .into_iter()
        .collect()
    };
    Self::validate(&model, &admins, &members)?;

    let admins: Vec<_> = admins
      .into_iter()
      .map(|user| group_user::ActiveModel {
        group_id: Set(command.target_id),
        user_id: Set(user.id),
        is_admin: Set(true),
      })
      .collect();
    let members: Vec<_> = members
      .into_iter()
      .map(|user| group_user::ActiveModel {
        group_id: Set(command.target_id),
        user_id: Set(user.id),
        is_admin: Set(false),
      })
      .collect();

    GroupUser::delete_many()
      .filter(group_user::Column::GroupId.eq(group.id))
      .exec(conn)
      .await?;
    let users: Vec<_> = vec![admins, members].into_iter().flatten().collect();
    let _ = GroupUser::insert_many(users).exec(conn).await?;

    Ok(model.update(conn).await?)
  }

  pub async fn delete(conn: &impl ConnectionTrait, operator: &user::Model, id: uuid::Uuid) -> anyhow::Result<()> {
    let group = Group::find_by_id(id).one(conn).await?.ok_or_else(|| Error::NotFound {
      entity: group::TYPE.to_owned(),
      field: FIELD_ID.to_owned(),
      value: id.to_string(),
    })?;

    Self::check_writeable(conn, operator, &group).await?;

    let model = group::ActiveModel {
      id: Set(id),
      ..Default::default()
    };
    model.delete(conn).await?;
    Ok(())
  }
}

#[async_trait::async_trait]
impl AbstractWriteService for GroupService {
  type Command = GroupCommand;

  async fn check_writeable(conn: &impl ConnectionTrait, user: &user::Model, model: &Self::Model) -> anyhow::Result<()> {
    if user.role > user::Role::User {
      return Ok(());
    }

    if !Self::contain_user(conn, model, user, Some(vec![FIELD_ADMINS])).await? {
      return Err(Error::InvalidPermission {
        user: user.id.to_string(),
        entity: group::TYPE.to_owned(),
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
  ) -> anyhow::Result<Option<Self::Model>> {
    if let AuthUser::User(operator) = operator {
      match command {
        GroupCommand::Create(command) => {
          let result = Self::create(conn, operator, command).await?;
          Ok(Some(result))
        }
        GroupCommand::Update(command) => {
          let result = Self::update(conn, operator, command).await?;
          Ok(Some(result))
        }
        GroupCommand::Delete(id) => {
          Self::delete(conn, operator, id).await?;
          Ok(None)
        }
      }
    } else {
      Err(Error::InvalidPermission {
        user: operator.get_id(),
        entity: group::TYPE.to_owned(),
        id: command.target_id(),
        permission: Permission::Write,
      })?
    }
  }
}
