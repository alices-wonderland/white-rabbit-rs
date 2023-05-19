#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

use backend_core::{
  AggregateRoot, Error, FindAllArgs, Order, Repository, Role, User, UserCommand, UserCommandCreate, UserQuery,
};

use futures::TryStreamExt;
use sea_orm::{Database, DbConn, TransactionTrait};
use std::collections::HashSet;
use std::default::Default;
use std::env;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
async fn user_create(db: State<'_, DbConn>, command: UserCommandCreate) -> Result<Option<User>, String> {
  let tx = db.inner().begin().await.map_err(Error::from)?;
  let result = User::handle(&tx, None, UserCommand::Create(command)).await?;
  tx.commit().await.map_err(Error::from)?;
  Ok(result.into_iter().last())
}

#[tauri::command]
async fn user_find_all(
  db: State<'_, DbConn>,
  id: HashSet<Uuid>,
  name: String,
  role: Option<Role>,
  sort: Vec<(String, Order)>,
) -> Result<Vec<User>, String> {
  let tx = db.inner().begin().await.map_err(Error::from)?;
  let result =
    Repository::<User>::find_all(&tx, FindAllArgs { query: UserQuery { id, name: (name, true), role }, sort })
      .await?
      .try_collect::<Vec<_>>()
      .await?;
  tx.commit().await.map_err(Error::from)?;
  Ok(result)
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
