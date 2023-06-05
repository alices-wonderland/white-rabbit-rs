use crate::{get_user, RunnerArgs};
use anyhow::anyhow;
use backend_core::user::{Role, User, MAX_NAME};
use backend_core::{user, AggregateRoot, Error, Result, FIELD_ID, FIELD_NAME, FIELD_NAME_LENGTH};
use itertools::Itertools;
use std::collections::HashSet;
use std::future::Future;
use uuid::Uuid;

pub async fn test_create_user<F, Fut>(runner: F) -> anyhow::Result<()>
where
  F: Fn(RunnerArgs<User>) -> Fut,
  Fut: Future<Output = Result<Vec<User>>>,
{
  let db = crate::init().await?;
  crate::populate_data(&db).await?;

  let operators =
    vec![None, get_user(&db, user::Query { role: Some(Role::User), ..Default::default() }).await?];

  for operator in operators {
    let command = user::CommandCreate {
      id: None,
      name: format!("New User {}", Uuid::new_v4()),
      role: Role::User,
    };

    let result = runner(RunnerArgs {
      db: db.clone(),
      operator,
      command: user::Command::Create(command.clone()),
    })
    .await?;

    assert_eq!(result[0].name, command.name);
    assert_eq!(result[0].role, command.role);
  }

  Ok(())
}

pub async fn test_create_admin<F, Fut>(runner: F) -> anyhow::Result<()>
where
  F: Fn(RunnerArgs<User>) -> Fut,
  Fut: Future<Output = Result<Vec<User>>>,
{
  let db = crate::init().await?;
  crate::populate_data(&db).await?;

  let operator =
    get_user(&db, user::Query { role: Some(Role::Admin), ..Default::default() }).await?;

  let command = user::CommandCreate { id: None, name: "New User".to_string(), role: Role::Admin };
  let result = runner(RunnerArgs {
    db: db.clone(),
    operator,
    command: user::Command::Create(command.clone()),
  })
  .await?;

  assert_eq!(result[0].name, command.name);
  assert_eq!(result[0].role, command.role);

  Ok(())
}

pub async fn test_error_non_admin_cannot_create_admin<F, Fut>(runner: F) -> anyhow::Result<()>
where
  F: Fn(RunnerArgs<User>) -> Fut,
  Fut: Future<Output = Result<Vec<User>>>,
{
  let db = crate::init().await?;
  crate::populate_data(&db).await?;

  let operators =
    vec![None, get_user(&db, user::Query { role: Some(Role::User), ..Default::default() }).await?];

  let command = user::CommandCreate { id: None, name: "New User".to_string(), role: Role::Admin };

  for operator in operators {
    if let Err(Error::NoWritePermission { operator_id, typ, field_values }) = runner(RunnerArgs {
      db: db.clone(),
      operator: operator.clone(),
      command: user::Command::Create(command.clone()),
    })
    .await
    {
      assert_eq!(operator_id, operator.map(|user| user.id()));
      assert_eq!(typ, User::typ());
      assert_eq!(field_values, vec![("role".to_string(), Role::Admin.to_string())]);
    } else {
      return Err(anyhow!("Non-admin should not create users with Role::Admin"));
    }
  }

  Ok(())
}

pub async fn test_error_invalid_names<F, Fut>(runner: F) -> anyhow::Result<()>
where
  F: Fn(RunnerArgs<User>) -> Fut,
  Fut: Future<Output = Result<Vec<User>>>,
{
  let db = crate::init().await?;
  crate::populate_data(&db).await?;

  let commands = vec!["   ", "", "A", &(0..MAX_NAME + 1).map(|_| "A").join(",")]
    .into_iter()
    .map(|name| {
      user::Command::Create(user::CommandCreate {
        id: None,
        name: name.to_string(),
        role: Role::User,
      })
    })
    .collect::<Vec<_>>();

  for command in commands {
    let result = runner(RunnerArgs { db: db.clone(), operator: None, command }).await;
    if let Err(Error::NotInRange { field, .. }) = result {
      assert_eq!(field, FIELD_NAME_LENGTH);
    } else {
      return Err(anyhow!("user.name should be valid"));
    }
  }

  Ok(())
}

pub async fn test_error_duplicate_name<F, Fut>(runner: F) -> anyhow::Result<()>
where
  F: Fn(RunnerArgs<User>) -> Fut,
  Fut: Future<Output = Result<Vec<User>>>,
{
  let db = crate::init().await?;
  crate::populate_data(&db).await?;

  let operator =
    get_user(&db, user::Query { role: Some(Role::Admin), ..Default::default() }).await?;
  let name = operator.as_ref().map(|user| user.name.clone()).unwrap();

  let result = runner(RunnerArgs {
    db: db.clone(),
    operator,
    command: user::Command::Create(user::CommandCreate {
      id: None,
      name: name.clone(),
      role: Role::User,
    }),
  })
  .await;
  if let Err(Error::AlreadyExist { typ, field_values }) = result {
    assert_eq!(typ, User::typ());
    assert_eq!(field_values, vec![(FIELD_NAME.to_string(), name.clone())]);
  } else {
    return Err(anyhow!("user.name should be valid"));
  }

  Ok(())
}

pub async fn test_update_user<F, Fut>(runner: F) -> anyhow::Result<()>
where
  F: Fn(RunnerArgs<User>) -> Fut,
  Fut: Future<Output = Result<Vec<User>>>,
{
  let db = crate::init().await?;
  crate::populate_data(&db).await?;

  let operator =
    get_user(&db, user::Query { role: Some(Role::User), ..Default::default() }).await?.unwrap();

  let command = user::CommandUpdate {
    id: operator.id().to_string(),
    name: Some("Updated User Name".to_string()),
    role: None,
  };

  let result = runner(RunnerArgs {
    db: db.clone(),
    operator: Some(operator.clone()),
    command: user::Command::Update(command.clone()),
  })
  .await?;

  assert_eq!(result[0].name, command.name.unwrap());
  assert_eq!(result[0].role, operator.role);

  Ok(())
}

pub async fn test_error_user_modifying_other_user<F, Fut>(runner: F) -> anyhow::Result<()>
where
  F: Fn(RunnerArgs<User>) -> Fut,
  Fut: Future<Output = Result<Vec<User>>>,
{
  let db = crate::init().await?;
  crate::populate_data(&db).await?;

  let operator =
    get_user(&db, user::Query { role: Some(Role::User), ..Default::default() }).await?.unwrap();
  let user_to_modify =
    get_user(&db, user::Query { role: Some(Role::Admin), ..Default::default() }).await?.unwrap(); // 获取待修改的用户（非操作者自身）

  let command = user::CommandUpdate {
    id: user_to_modify.id().to_string(),
    name: Some("Modified User Name".to_string()),
    role: None,
  };

  if let Err(Error::NoWritePermission { operator_id, field_values, .. }) = runner(RunnerArgs {
    db: db.clone(),
    operator: Some(operator.clone()),
    command: user::Command::Update(command.clone()),
  })
  .await
  {
    assert_eq!(operator_id.unwrap(), operator.id());
    assert_eq!(field_values, vec![(FIELD_ID.to_string(), user_to_modify.id().to_string())]);
  } else {
    return Err(anyhow!("User[role = User] should not update other users"));
  }

  Ok(())
}

pub async fn test_delete_user<F, Fut>(runner: F) -> anyhow::Result<()>
where
  F: Fn(RunnerArgs<User>) -> Fut,
  Fut: Future<Output = Result<Vec<User>>>,
{
  let db = crate::init().await?;
  crate::populate_data(&db).await?;

  let operator =
    get_user(&db, user::Query { role: Some(Role::Admin), ..Default::default() }).await?;
  let user_to_delete =
    get_user(&db, user::Query { role: Some(Role::User), ..Default::default() }).await?.unwrap();

  let command = user::CommandDelete { id: HashSet::from_iter(vec![user_to_delete.id()]) };

  let result = runner(RunnerArgs {
    db: db.clone(),
    operator,
    command: user::Command::Delete(command.clone()),
  })
  .await?;

  assert!(result.is_empty());

  Ok(())
}

pub async fn test_error_user_deleting_other_user<F, Fut>(runner: F) -> anyhow::Result<()>
where
  F: Fn(RunnerArgs<User>) -> Fut,
  Fut: Future<Output = Result<Vec<User>>>,
{
  let db = crate::init().await?;
  crate::populate_data(&db).await?;

  let operator =
    get_user(&db, user::Query { role: Some(Role::User), ..Default::default() }).await?.unwrap();
  let user_to_delete =
    get_user(&db, user::Query { role: Some(Role::Admin), ..Default::default() }).await?.unwrap(); // 获取待修改的用户（非操作者自身）

  let command = user::CommandDelete { id: HashSet::from_iter(vec![user_to_delete.id()]) };

  if let Err(Error::NoWritePermission { operator_id, field_values, .. }) = runner(RunnerArgs {
    db: db.clone(),
    operator: Some(operator.clone()),
    command: user::Command::Delete(command.clone()),
  })
  .await
  {
    assert_eq!(operator_id.unwrap(), operator.id());
    assert_eq!(field_values, vec![(FIELD_ID.to_string(), user_to_delete.id().to_string())]);
  } else {
    return Err(anyhow!("User[role = User] should not delete other users"));
  }

  Ok(())
}

#[macro_export]
macro_rules! generate_user_tests {
    ($runner: ident) => {
      ::test_suite::generate_tests!(
        $runner;
        user_test;
        test_create_user,
        test_create_admin,
        test_error_non_admin_cannot_create_admin,
        test_error_invalid_names,
        test_error_duplicate_name,
        test_update_user,
        test_error_user_modifying_other_user,
        test_delete_user,
        test_error_user_deleting_other_user
      );
    };
}
