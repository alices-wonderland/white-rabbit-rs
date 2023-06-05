#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

use backend_core::user::User;
use backend_core::{user, AggregateRoot, Error, FindAllArgs, Order, Presentation, Repository};
use futures::{TryFutureExt, TryStreamExt};
use sea_orm::{ConnectOptions, Database, DatabaseTransaction, DbConn, TransactionTrait};
use std::collections::HashSet;
use std::default::Default;
use std::env;
use tauri::State;
use uuid::Uuid;

async fn test_get_operator(db: &DatabaseTransaction) -> backend_core::Result<Option<User>> {
  Repository::<User>::do_find_all(
    db,
    FindAllArgs {
      query: user::Query { role: Some(user::Role::Admin), ..Default::default() },
      ..Default::default()
    },
  )
  .await?
  .try_next()
  .await
}

#[tauri::command]
async fn user_create(
  db: State<'_, DbConn>,
  command: user::CommandCreate,
) -> Result<Option<user::Presentation>, String> {
  Ok(
    db.inner()
      .transaction(|tx| {
        Box::pin(async move {
          let operator = test_get_operator(tx).await?;
          let result = User::handle(tx, operator.as_ref(), user::Command::Create(command)).await?;
          Ok(Presentation::from(tx, operator.as_ref(), result).await?.into_iter().last())
        })
      })
      .map_ok(|models| models.into_iter().last())
      .map_err(Error::from)
      .await?,
  )
}

#[tauri::command]
async fn user_find_all(
  db: State<'_, DbConn>,
  id: HashSet<Uuid>,
  name: String,
  role: Option<user::Role>,
  sort: Vec<(String, Order)>,
) -> Result<Vec<user::Presentation>, String> {
  Ok(
    db.inner()
      .transaction(|tx| {
        Box::pin(async move {
          let operator = test_get_operator(tx).await?;
          let result = Repository::<User>::find_all(
            tx,
            operator.as_ref(),
            FindAllArgs { query: user::Query { id, name: (name, true), role }, sort },
          )
          .await?
          .try_collect::<Vec<_>>()
          .await?;

          Presentation::from(tx, operator.as_ref(), result).await
        })
      })
      .map_err(Error::from)
      .await?,
  )
}

async fn init() -> anyhow::Result<DbConn> {
  let _ = dotenv::from_filename(".desktop.test.env")?;
  let _ = env_logger::try_init();
  let mut opt: ConnectOptions = env::var("WHITE_RABBIT_DATABASE_URL")?.into();
  opt.max_connections(10).min_connections(5);
  Ok(Database::connect(opt).await?)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let db = init().await?;

  log::info!("Tauri starts");

  tauri::Builder::default()
    .manage(db)
    .invoke_handler(tauri::generate_handler![user_create, user_find_all])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
  Ok(())
}
