#![feature(result_option_inspect)]
#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

use backend_core::{create, Repository, User, UserCommandCreate, UserRepository};
use futures::{StreamExt, TryStreamExt};
use sea_orm::{Database, DbConn, TransactionTrait};
use std::env;
use tauri::State;

#[tauri::command]
async fn user_create(db: State<'_, DbConn>, command: UserCommandCreate) -> Result<Option<User>, String> {
  let tx = db.inner().begin().await.map_err(|e| e.to_string())?;
  let result = create(&tx, command).await.inspect_err(|e| log::error!("Error: {:?}", e)).map_err(|e| e.to_string())?;
  tx.commit().await.map_err(|e| e.to_string())?;
  Ok(result)
}

#[tauri::command]
async fn user_find_all(db: State<'_, DbConn>) -> Result<Vec<User>, String> {
  let tx = db.inner().begin().await.map_err(|e| e.to_string())?;
  let result = UserRepository::find_all(&tx)
    .await
    .map_err(|e| e.to_string())?
    .try_chunks(10)
    .take(2)
    .try_collect::<Vec<_>>()
    .await
    .map_err(|e| e.to_string())?;
  tx.commit().await.map_err(|e| e.to_string())?;
  Ok(result.into_iter().flatten().collect())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let _ = dotenv::from_filename(".desktop.test.env")?;
  let _ = env_logger::try_init();
  let conn = Database::connect(env::var("WHITE_RABBIT_DATABASE_URL")?).await?;

  log::info!("Tauri starts");

  tauri::Builder::default()
    .manage(conn)
    .invoke_handler(tauri::generate_handler![user_create, user_find_all])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
  Ok(())
}

#[cfg(test)]
mod tests {
  use migration::{Migrator, MigratorTrait};
  use sea_orm::Database;

  #[tokio::test]
  async fn populate_data() -> anyhow::Result<()> {
    let _ = dotenv::from_filename(".desktop.test.env")?;
    let _ = env_logger::try_init();

    let db = Database::connect(std::env::var("WHITE_RABBIT_DATABASE_URL")?).await?;
    Migrator::up(&db, None).await?;

    Ok(())
  }
}
