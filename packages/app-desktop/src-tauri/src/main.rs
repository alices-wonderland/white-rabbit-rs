#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

use backend_core::user::User;
use backend_core::{
  user, AggregateRoot, Error, FindAllArgs, FindPageArgs, Page, Presentation, Repository, Sort,
};
use futures::{TryFutureExt, TryStreamExt};
use sea_orm::{ConnectOptions, Database, DatabaseTransaction, DbConn, TransactionTrait};

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
async fn user_find_page(
  db: State<'_, DbConn>,
  query: user::Query,
  sort: Sort,
  after: Option<Uuid>,
  before: Option<Uuid>,
  size: usize,
) -> Result<Page<User>, String> {
  Ok(
    db.inner()
      .transaction(|tx| {
        Box::pin(async move {
          let operator = test_get_operator(tx).await?;
          Repository::<User>::find_page(
            tx,
            FindPageArgs { operator: operator.as_ref(), query, sort, after, before, size },
          )
          .await
        })
      })
      .map_err(Error::from)
      .await?,
  )
}

#[tauri::command]
async fn user_find_all(
  db: State<'_, DbConn>,
  query: user::Query,
  sort: Sort,
) -> Result<Vec<user::Presentation>, String> {
  Ok(
    db.inner()
      .transaction(|tx| {
        Box::pin(async move {
          let operator = test_get_operator(tx).await?;
          let result =
            Repository::<User>::find_all(tx, operator.as_ref(), FindAllArgs { query, sort })
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

#[tauri::command]
async fn user_find_by_id(
  db: State<'_, DbConn>,
  id: Uuid,
) -> Result<Option<user::Presentation>, String> {
  Ok(
    db.inner()
      .transaction(|tx| {
        Box::pin(async move {
          let operator = test_get_operator(tx).await?;
          let result = Repository::<User>::find_by_id(tx, id).await?;

          if let Some(result) = result {
            Ok(Presentation::from(tx, operator.as_ref(), vec![result]).await?.into_iter().last())
          } else {
            Ok(None)
          }
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
    .invoke_handler(tauri::generate_handler![
      user_create,
      user_find_page,
      user_find_all,
      user_find_by_id
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
  Ok(())
}
