use backend_core::user::{Role, User};

use crate::TestRunner;
use backend_core::{user, FindAllArgs, Repository};
use futures::stream::TryStreamExt;
use migration::sea_orm::prelude::Uuid;

pub async fn test_create_user<T>() -> crate::Result<()>
where
  T: TestRunner<AggregateRoot = User>,
{
  let db = crate::init().await?;
  crate::populate_data(&db).await?;

  let operators = vec![
    None,
    Repository::<User>::do_find_all(
      &db,
      FindAllArgs {
        query: user::Query { role: Some(Role::User), ..Default::default() },
        ..Default::default()
      },
    )
    .await?
    .try_next()
    .await?,
  ];

  for operator in operators {
    let command = user::CommandCreate {
      id: None,
      name: format!("New User {}", Uuid::new_v4()),
      role: Role::User,
    };

    let result: Vec<User> =
      T::run_test(&db, operator.as_ref(), user::Command::Create(command.clone())).await?;

    assert_eq!(result[0].name, command.name);
    assert_eq!(result[0].role, command.role);
  }

  Ok(())
}

pub async fn test_create_admin<T>() -> crate::Result<()>
where
  T: TestRunner<AggregateRoot = User>,
{
  let db = crate::init().await?;
  crate::populate_data(&db).await?;

  let operator = Repository::<User>::do_find_all(
    &db,
    FindAllArgs {
      query: user::Query { role: Some(Role::Admin), ..Default::default() },
      ..Default::default()
    },
  )
  .await?
  .try_next()
  .await?;

  let command = user::CommandCreate { id: None, name: "New User".to_string(), role: Role::Admin };
  let result: Vec<User> =
    T::run_test(&db, operator.as_ref(), user::Command::Create(command.clone())).await?;

  assert_eq!(result[0].name, command.name);
  assert_eq!(result[0].role, command.role);

  Ok(())
}

#[macro_export]
macro_rules! generate_user_tests {
    ($runner: path) => {
      ::test_suite::generate_tests!(
        $runner;
        user_test;
        test_create_user,
        test_create_admin
      );
    };
}
