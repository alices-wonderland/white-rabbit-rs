#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

use backend_shared::{
  models::{user, IntoPresentation},
  services::{AbstractReadService, AuthUser, FindAllInput, UserService},
};
use futures::{stream, StreamExt, TryStreamExt};
use sea_orm::{Database, DatabaseConnection, TransactionTrait};

#[tauri::command]
async fn get_users(
  state: tauri::State<'_, DatabaseConnection>,
) -> Result<Vec<user::Presentation>, backend_shared::Error> {
  let txn = state.inner().begin().await?;
  let result = UserService::find_all(
    &txn,
    &AuthUser::Id(("Provider".to_owned(), "Value".to_owned())),
    FindAllInput::default(),
  )
  .await?;
  let result = stream::iter(result)
    .then(|item| item.into_presentation(&txn))
    .try_collect()
    .await?;
  txn.commit().await?;
  Ok(result)
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
  dotenv::from_filename(".desktop.test.env")?;
  let _ = env_logger::try_init();

  tauri::Builder::default()
    .manage(Database::connect(std::env::var("WHITE_RABBIT_DATABASE_URL")?).await?)
    .invoke_handler(tauri::generate_handler![get_users])
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
