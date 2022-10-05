#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

use backend_shared::{
  models::{account, record, user, IntoPresentation, User},
  services::{
    AbstractReadService, AccountQuery, AccountService, AuthUser, FindAllInput, FindPageInput, Page, RecordQuery,
    RecordService, UserQuery, UserService, FIELD_ID,
  },
};
use futures::{stream, StreamExt, TryStreamExt};
use sea_orm::{Database, DatabaseConnection, EntityTrait, TransactionTrait};

#[tauri::command]
async fn get_users(
  state: tauri::State<'_, DatabaseConnection>,
  operator: Option<uuid::Uuid>,
  input: FindAllInput<UserQuery>,
) -> Result<Vec<user::Presentation>, backend_shared::Error> {
  let txn = state.inner().begin().await?;
  let operator: AuthUser = if let Some(operator) = operator {
    User::find_by_id(operator)
      .one(&txn)
      .await?
      .ok_or_else(|| backend_shared::Error::NotFound {
        entity: user::TYPE.to_owned(),
        field: FIELD_ID.to_owned(),
        value: operator.to_string(),
      })?
      .into()
  } else {
    AuthUser::Id(("Provider".to_owned(), "Value".to_owned()))
  };

  let result = UserService::find_all(&txn, &operator, input).await?;
  let result = stream::iter(result)
    .then(|item| item.into_presentation(&txn))
    .try_collect()
    .await?;
  txn.commit().await?;
  Ok(result)
}

#[tauri::command]
async fn get_user_page(
  state: tauri::State<'_, DatabaseConnection>,
  operator: uuid::Uuid,
  input: FindPageInput<UserQuery>,
) -> Result<Page<user::Presentation>, backend_shared::Error> {
  let txn = state.inner().begin().await?;
  let operator: AuthUser = User::find_by_id(operator)
    .one(&txn)
    .await?
    .ok_or_else(|| backend_shared::Error::NotFound {
      entity: user::TYPE.to_owned(),
      field: FIELD_ID.to_owned(),
      value: operator.to_string(),
    })?
    .into();
  let result = UserService::find_page(&txn, &operator, input).await?;
  txn.commit().await?;
  Ok(result)
}

#[tauri::command]
async fn get_records(
  state: tauri::State<'_, DatabaseConnection>,
  operator: uuid::Uuid,
  input: FindAllInput<RecordQuery>,
) -> Result<Vec<record::Presentation>, backend_shared::Error> {
  let txn = state.inner().begin().await?;
  let operator: AuthUser = User::find_by_id(operator)
    .one(&txn)
    .await?
    .ok_or_else(|| backend_shared::Error::NotFound {
      entity: user::TYPE.to_owned(),
      field: FIELD_ID.to_owned(),
      value: operator.to_string(),
    })?
    .into();
  let result = RecordService::find_all(&txn, &operator, input).await.unwrap();
  log::info!("records: {:#?}", result);

  let result = stream::iter(result)
    .then(|item| item.into_presentation(&txn))
    .try_collect()
    .await
    .unwrap();
  log::info!("record presentations: {:#?}", result);

  txn.commit().await?;
  Ok(result)
}

#[tauri::command]
async fn get_accounts(
  state: tauri::State<'_, DatabaseConnection>,
  operator: uuid::Uuid,
  input: FindAllInput<AccountQuery>,
) -> Result<Vec<account::Presentation>, backend_shared::Error> {
  let txn = state.inner().begin().await?;
  let operator: AuthUser = User::find_by_id(operator)
    .one(&txn)
    .await?
    .ok_or_else(|| backend_shared::Error::NotFound {
      entity: user::TYPE.to_owned(),
      field: FIELD_ID.to_owned(),
      value: operator.to_string(),
    })?
    .into();
  let result = AccountService::find_all(&txn, &operator, input).await?;
  let result = stream::iter(result)
    .then(|item| item.into_presentation(&txn))
    .try_collect()
    .await?;
  txn.commit().await?;
  Ok(result)
}

#[tauri::command]
async fn get_account_by_id(
  state: tauri::State<'_, DatabaseConnection>,
  operator: uuid::Uuid,
  id: uuid::Uuid,
) -> Result<Option<account::Presentation>, backend_shared::Error> {
  let txn = state.inner().begin().await?;
  let operator: AuthUser = User::find_by_id(operator)
    .one(&txn)
    .await?
    .ok_or_else(|| backend_shared::Error::NotFound {
      entity: user::TYPE.to_owned(),
      field: FIELD_ID.to_owned(),
      value: operator.to_string(),
    })?
    .into();
  let result = if let Some(result) = AccountService::find_by_id(&txn, &operator, id).await? {
    Some(result.into_presentation(&txn).await?)
  } else {
    None
  };
  txn.commit().await?;
  Ok(result)
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
  dotenv::from_filename(".desktop.test.env")?;
  let _ = env_logger::try_init();

  tauri::Builder::default()
    .manage(Database::connect(std::env::var("WHITE_RABBIT_DATABASE_URL")?).await?)
    .invoke_handler(tauri::generate_handler![
      get_users,
      get_user_page,
      get_records,
      get_accounts,
      get_account_by_id,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
  Ok(())
}

#[cfg(test)]
mod tests {
  use backend_test::{Migrator, MigratorTrait};
  use sea_orm::Database;

  #[tokio::test]
  async fn populate_data() -> anyhow::Result<()> {
    dotenv::from_filename(".desktop.test.env")?;
    let _ = env_logger::try_init();

    let db = Database::connect(std::env::var("WHITE_RABBIT_DATABASE_URL")?).await?;
    Migrator::up(&db, None).await?;

    Ok(())
  }
}
