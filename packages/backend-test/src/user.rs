use std::collections::HashSet;
use std::sync::Arc;

use lazy_static::lazy_static;
use sea_orm_migration::sea_orm::{ColumnTrait, EntityTrait, ModelTrait, QueryFilter};

use crate::task::{Input, ServiceTask, Task};
use backend_shared::models::{auth_id, user, AuthId, User};
use backend_shared::services::{
  AuthUser, FindPageInput, Order, Sort, UserCommand, UserCommandCreate, UserCommandUpdate, UserService,
};
use itertools::Itertools;

lazy_static! {
  pub static ref TASKS: Vec<ServiceTask<UserService>> = vec![
    Task::FindById(Input {
      name: "Find User By Id".to_owned(),
      auth_user: Arc::new(Box::new(|_| Box::pin(async move {
        Ok(AuthUser::Id(("Provider".to_owned(), "Value".to_owned())))
      }))),
      input: Arc::new(Box::new(|(conn, _)| Box::pin(async move {
        Ok(
          User::find()
            .filter(user::Column::Role.eq(user::Role::Admin))
            .one(&*conn)
            .await?
            .unwrap()
            .id,
        )
      }))),
      checker: Arc::new(Box::new(|(conn, _, input, output)| Box::pin(async move {
        let user = User::find().filter(user::Column::Id.eq(input)).one(&*conn).await?;
        assert_eq!(user, output?);
        Ok(())
      }))),
    }),
    Task::FindPage(Input {
      name: "Find User Page".to_owned(),
      auth_user: Arc::new(Box::new(|_| Box::pin(async move {
        Ok(AuthUser::Id(("Provider".to_owned(), "Value".to_owned())))
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
      checker: Arc::new(Box::new(|(_, _, _, output)| Box::pin(async move {
        for (a, b) in output?.items.iter().tuple_windows() {
          assert!(a.item.name < b.item.name);
        }
        Ok(())
      }))),
    }),
    Task::Handle(Input {
      name: "Create User".to_owned(),
      auth_user: Arc::new(Box::new(|_| Box::pin(async move {
        Ok(AuthUser::Id(("Provider".to_owned(), "Value".to_owned())))
      }))),
      input: Arc::new(Box::new(|_| Box::pin(async move {
        Ok(UserCommand::Create(UserCommandCreate {
          target_id: None,
          name: "new user name".to_owned(),
          role: user::Role::User,
          auth_ids: HashSet::default(),
        }))
      }))),
      checker: Arc::new(Box::new(|(conn, auth_user, input, output)| Box::pin(async move {
        match (&*auth_user, input) {
          (AuthUser::Id((provider, value)), UserCommand::Create(input)) => {
            let user: user::Model = output?.unwrap();
            let auth_ids = user.find_related(AuthId).all(&*conn).await?;
            assert_eq!(input.name, user.name);
            assert_eq!(input.role, user.role);
            assert_eq!(
              vec![auth_id::Model {
                user_id: user.id,
                provider: provider.clone(),
                value: value.clone()
              }],
              auth_ids
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
            .filter(user::Column::Role.eq(user::Role::Admin))
            .one(&*conn)
            .await?
            .unwrap(),
        ))
      }))),
      input: Arc::new(Box::new(|_| Box::pin(async move {
        let lid = uuid::Uuid::new_v4();
        Ok(vec![
          UserCommand::Create(UserCommandCreate {
            target_id: Some(lid),
            name: "new user name".to_owned(),
            role: user::Role::User,
            auth_ids: HashSet::from_iter(vec![
              ("Provider 1".to_owned(), "Value 1".to_owned()),
              ("Provider 2".to_owned(), "Value 2".to_owned()),
            ]),
          }),
          UserCommand::Update(UserCommandUpdate {
            target_id: lid,
            name: Some("new new user name".to_owned()),
            role: None,
            auth_ids: None,
          }),
          UserCommand::Delete(lid),
        ])
      }))),
      checker: Arc::new(Box::new(|(conn, _, input, output)| Box::pin(async move {
        match (&*input, &*output?) {
          (
            &[UserCommand::Create(ref create), UserCommand::Update(ref update), UserCommand::Delete(_)],
            &[Some(ref create_result), Some(ref update_result), None],
          ) => {
            assert_eq!(create.name, create_result.name);
            assert_eq!(create.role, create_result.role);
            assert_eq!(create.auth_ids, create_result.auth_ids);

            assert_eq!(update.name.as_ref(), Some(&update_result.name));
            assert_eq!(create.role, update_result.role);
            assert_eq!(create.auth_ids, update_result.auth_ids);

            let tags: Vec<_> = user::Model::from(update_result.clone())
              .find_related(AuthId)
              .all(&*conn)
              .await?;
            assert!(tags.is_empty());

            Ok(())
          }
          _ => unreachable!(),
        }
      }))),
    }),
  ];
}

#[cfg(test)]
mod tests {
  use super::TASKS;
  use backend_shared::services::UserService;

  #[tokio::test]
  async fn test_tasks() -> backend_shared::Result<()> {
    crate::tests::run_test::<UserService>(&TASKS).await
  }
}
