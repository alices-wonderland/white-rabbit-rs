use std::collections::HashSet;
use std::sync::Arc;

use lazy_static::lazy_static;
use sea_orm_migration::sea_orm::{ColumnTrait, EntityTrait, ModelTrait, QueryFilter};

use crate::task::{Input, ServiceTask, Task};
use backend_shared::models::{group, group_user, user, Group, GroupUser, User};
use backend_shared::services::{
  AbstractReadService, AuthUser, FindAllInput, FindPageInput, GroupCommand, GroupCommandCreate, GroupCommandUpdate,
  GroupService, Order, Sort, FIELD_ADMINS,
};
use itertools::Itertools;

lazy_static! {
  pub static ref TASKS: Vec<ServiceTask<GroupService>> = vec![
    Task::FindById(Input {
      name: "Find Group By Id".to_owned(),
      auth_user: Arc::new(Box::new(|conn| Box::pin(async move {
        let user_id = GroupUser::find()
          .filter(group_user::Column::IsAdmin.eq(false))
          .one(&*conn)
          .await?
          .unwrap()
          .user_id;
        Ok(AuthUser::User(User::find_by_id(user_id).one(&*conn).await?.unwrap()))
      }))),
      input: Arc::new(Box::new(|(conn, auth_user)| Box::pin(async move {
        Ok(GroupService::find_all(&*conn, &*auth_user, FindAllInput::default()).await?[0].id)
      }))),
      checker: Arc::new(Box::new(|(conn, _, input, output)| Box::pin(async move {
        let group = Group::find().filter(group::Column::Id.eq(input)).one(&*conn).await?;
        assert_eq!(group, output?);
        Ok(())
      }))),
    }),
    Task::FindPage(Input {
      name: "Find Group Page".to_owned(),
      auth_user: Arc::new(Box::new(|conn| Box::pin(async move {
        let user = User::find()
          .filter(user::Column::Role.eq(user::Role::User))
          .one(&*conn)
          .await?
          .unwrap();
        Ok(AuthUser::User(user))
      }))),
      input: Arc::new(Box::new(|_| Box::pin(async move {
        Ok(FindPageInput {
          query: None,
          pagination: None,
          sort: Sort {
            field: "name".to_owned(),
            order: Order::Asc,
          },
        })
      }))),
      checker: Arc::new(Box::new(|(conn, auth_user, _, output)| Box::pin(async move {
        for (a, b) in output?.items.iter().tuple_windows() {
          let group = group::Model::from(a.item.clone());
          assert!(GroupService::contain_user(&*conn, &group, auth_user.id().parse::<uuid::Uuid>()?, None).await?);
          assert!(a.item.name < b.item.name);
        }
        Ok(())
      }))),
    }),
    Task::Handle(Input {
      name: "Create Group".to_owned(),
      auth_user: Arc::new(Box::new(|conn| Box::pin(async move {
        let user = User::find()
          .filter(user::Column::Role.eq(user::Role::User))
          .one(&*conn)
          .await?
          .unwrap();
        Ok(AuthUser::User(user))
      }))),
      input: Arc::new(Box::new(|_| Box::pin(async move {
        Ok(GroupCommand::Create(GroupCommandCreate {
          target_id: None,
          name: "new group name".to_owned(),
          description: "new group description".to_owned(),
          admins: HashSet::default(),
          members: HashSet::default(),
        }))
      }))),
      checker: Arc::new(Box::new(|(conn, auth_user, input, output)| Box::pin(async move {
        match input {
          GroupCommand::Create(input) => {
            let group: group::Model = output?.unwrap();
            assert_eq!(input.name, group.name);
            assert!(
              GroupService::contain_user(
                &*conn,
                &group,
                auth_user.id().parse::<uuid::Uuid>()?,
                Some(vec![FIELD_ADMINS])
              )
              .await?
            );
            Ok(())
          }
          _ => unreachable!(),
        }
      }))),
    }),
    Task::HandleAll(Input {
      name: "Create, Update and Delete".to_owned(),
      auth_user: Arc::new(Box::new(|conn| Box::pin(async move {
        Ok(AuthUser::User(
          User::find()
            .filter(user::Column::Role.eq(user::Role::User))
            .one(&*conn)
            .await?
            .unwrap(),
        ))
      }))),
      input: Arc::new(Box::new(|_| Box::pin(async move {
        let lid = uuid::Uuid::new_v4();
        Ok(vec![
          GroupCommand::Create(GroupCommandCreate {
            target_id: Some(lid),
            name: "new group name".to_owned(),
            description: "new description".to_owned(),
            admins: HashSet::new(),
            members: HashSet::new(),
          }),
          GroupCommand::Update(GroupCommandUpdate {
            target_id: lid,
            name: Some("new new group name".to_owned()),
            description: Some("new description".to_owned()),
            admins: None,
            members: None,
          }),
          GroupCommand::Delete(lid),
        ])
      }))),
      checker: Arc::new(Box::new(|(conn, auth_user, input, output)| Box::pin(async move {
        match (&*auth_user, &*input, &*output?) {
          (
            AuthUser::User(auth_user),
            &[GroupCommand::Create(ref create), GroupCommand::Update(ref update), GroupCommand::Delete(_)],
            &[Some(ref create_result), Some(ref update_result), None],
          ) => {
            assert_eq!(create.name, create_result.name);
            assert_eq!(create.description, create_result.description);
            assert_eq!(HashSet::from_iter(vec![auth_user.id]), create_result.admins);

            assert_eq!(update.name.as_ref(), Some(&update_result.name));
            assert_eq!(create.description, update_result.description);
            assert_eq!(HashSet::from_iter(vec![auth_user.id]), update_result.admins);

            let users: Vec<_> = group::Model::from(update_result.clone())
              .find_related(User)
              .all(&*conn)
              .await?;
            assert!(users.is_empty());

            Ok(())
          }
          _ => unreachable!(),
        }
      }))),
    })
  ];
}

#[cfg(test)]
mod tests {
  use super::TASKS;
  use backend_shared::services::GroupService;

  #[tokio::test]
  async fn test_tasks() -> backend_shared::Result<()> {
    crate::tests::run_test::<GroupService>(&TASKS).await
  }
}
