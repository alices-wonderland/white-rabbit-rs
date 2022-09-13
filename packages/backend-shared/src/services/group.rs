use std::collections::HashSet;

use sea_orm::{
  sea_query::{Condition, IntoCondition, JoinType},
  ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, ModelTrait, PaginatorTrait, QueryFilter, QuerySelect,
  RelationTrait, Select, Set,
};
use serde::{Deserialize, Serialize};

use crate::models::{group, group_user, user, Group, GroupUser, User};

use super::{
  read_service::{AbstractReadService, ContainingUserQuery, ExternalQuery, FullTextQuery, IdQuery, TextQuery},
  write_service::{AbstractCommand, AbstractWriteService},
  AuthUser, FIELD_ADMINS, FIELD_DESCRIPTION, FIELD_MEMBERS, FIELD_NAME,
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

pub enum GroupCommand {
  Create(GroupCommandCreate),
  Update(GroupCommandUpdate),
  Delete(uuid::Uuid),
}

pub struct GroupCommandCreate {
  pub target_id: Option<uuid::Uuid>,
  pub name: String,
  pub description: String,
  pub admins: Vec<uuid::Uuid>,
  pub members: Vec<uuid::Uuid>,
}

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
  pub async fn create(
    conn: &impl ConnectionTrait,
    operator: user::Model,
    command: GroupCommandCreate,
  ) -> anyhow::Result<group::Model> {
    if Group::find()
      .filter(group::Column::Name.eq(command.name.clone()))
      .count(conn)
      .await?
      > 0
    {
      return Err(anyhow::Error::msg("Group name exists"));
    }

    let group = group::ActiveModel {
      id: Set(uuid::Uuid::new_v4()),
      name: Set(command.name),
      description: Set(command.description),
    };
    let group = group.insert(conn).await?;

    let users: HashSet<_> = vec![command.admins.clone(), command.members.clone(), vec![operator.id]]
      .into_iter()
      .flatten()
      .collect();
    let users: Vec<_> = User::find()
      .filter(user::Column::Id.is_in(users))
      .all(conn)
      .await?
      .into_iter()
      .map(|user| group_user::ActiveModel {
        group_id: Set(group.id),
        user_id: Set(user.id),
        is_admin: Set(
          operator.id == user.id || (command.admins.contains(&user.id) && !command.members.contains(&user.id)),
        ),
      })
      .collect();
    let _ = GroupUser::insert_many(users).exec(conn).await?;

    Ok(group)
  }

  pub async fn update(
    conn: &impl ConnectionTrait,
    operator: user::Model,
    command: GroupCommandUpdate,
  ) -> anyhow::Result<group::Model> {
    let group = Group::find_by_id(command.target_id)
      .one(conn)
      .await?
      .ok_or_else(|| anyhow::Error::msg("Not found"))?;
    if !group
      .find_linked(group::GroupAdmin)
      .all(conn)
      .await?
      .contains(&operator)
    {
      return Err(anyhow::Error::msg("No permission"));
    }

    if command.is_empty() {
      return Ok(group);
    }

    let mut model = group::ActiveModel {
      id: Set(command.target_id),
      ..Default::default()
    };

    if let Some(name) = command.name {
      model.name = Set(name);
    }

    if let Some(description) = command.description {
      model.description = Set(description);
    }

    let admins = if let Some(admins) = command.admins {
      User::find()
        .filter(user::Column::Id.is_in(admins))
        .all(conn)
        .await?
        .into_iter()
        .map(|user| group_user::ActiveModel {
          group_id: Set(group.id),
          user_id: Set(user.id),
          is_admin: Set(true),
        })
        .collect()
    } else {
      Vec::new()
    };

    let members = if let Some(members) = command.members {
      User::find()
        .filter(user::Column::Id.is_in(members))
        .all(conn)
        .await?
        .into_iter()
        .map(|user| group_user::ActiveModel {
          group_id: Set(group.id),
          user_id: Set(user.id),
          is_admin: Set(false),
        })
        .collect()
    } else {
      Vec::new()
    };

    let group_users: Vec<_> = vec![admins, members].into_iter().flatten().collect();

    GroupUser::delete_many()
      .filter(group_user::Column::GroupId.eq(group.id))
      .exec(conn)
      .await?;
    GroupUser::insert_many(group_users).exec(conn).await?;

    Ok(model.update(conn).await?)
  }

  pub async fn delete(conn: &impl ConnectionTrait, operator: user::Model, id: uuid::Uuid) -> anyhow::Result<()> {
    let group = Group::find_by_id(id)
      .one(conn)
      .await?
      .ok_or_else(|| anyhow::Error::msg("Not found"))?;
    if !group
      .find_linked(group::GroupAdmin)
      .all(conn)
      .await?
      .contains(&operator)
    {
      return Err(anyhow::Error::msg("No permission"));
    }

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

  async fn handle(
    conn: &impl ConnectionTrait,
    operator: AuthUser,
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
      Err(anyhow::Error::msg("Please login"))
    }
  }
}
