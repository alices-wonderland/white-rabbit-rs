#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

use backend_core::user::User;
use backend_core::{
  user, AggregateRoot, Error, FindAllArgs, FindPageArgs, Page, Presentation, Repository, Sort,
};
use futures::TryFutureExt;
use sea_orm::{ConnectOptions, Database, DatabaseTransaction, DbConn, TransactionTrait};
use std::default::Default;
use std::env;
use tauri::State;
use uuid::Uuid;

async fn test_get_operator(db: &DatabaseTransaction) -> backend_core::Result<Option<User>> {
  Ok(
    Repository::<User>::do_find_all(
      db,
      FindAllArgs {
        size: Some(1),
        query: user::Query { role: Some(user::Role::Admin), ..Default::default() },
        ..Default::default()
      },
      None,
    )
    .await?
    .into_iter()
    .last(),
  )
}

macro_rules! generate_tauri_command {
    ($model: ident) => {
      paste::paste! {
        #[tauri::command]
        async fn [<$model _handle_command>](
          db: State<'_, DbConn>,
          command: ::backend_core::$model::Command,
        ) -> Result<Option<::backend_core::$model::Presentation>, String> {
          Ok(
            db.inner()
              .transaction(|tx| {
                Box::pin(async move {
                  let operator = test_get_operator(tx).await?;
                  let result = ::backend_core::$model::[< $model:camel >]::handle(tx, operator.as_ref(), command).await?;
                  Ok(::backend_core::$model::Presentation::from_aggregate_roots(tx, operator.as_ref(), result).await?.into_iter().last())
                })
              })
              .map_ok(|models| models.into_iter().last())
              .map_err(Error::from)
              .await?,
          )
        }

        #[tauri::command]
        async fn [<$model _find_page>](
          db: State<'_, DbConn>,
          query: ::backend_core::$model::Query,
          sort: Sort,
          after: Option<Uuid>,
          before: Option<Uuid>,
          size: usize,
        ) -> Result<Page<::backend_core::$model::[< $model:camel >]>, String> {
          Ok(
            db.inner()
              .transaction(|tx| {
                Box::pin(async move {
                  let operator = test_get_operator(tx).await?;
                  Repository::<::backend_core::$model::[< $model:camel >]>::find_page(
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
        async fn [<$model _find_all>](
          db: State<'_, DbConn>,
          query: ::backend_core::$model::Query,
          sort: Option<Sort>,
          size: Option<usize>,
        ) -> Result<Vec<::backend_core::$model::Presentation>, String> {
          Ok(
            db.inner()
              .transaction(|tx| {
                Box::pin(async move {
                  let operator = test_get_operator(tx).await?;
                  let result =
                    Repository::<::backend_core::$model::[< $model:camel >]>::find_all(tx, operator.as_ref(), FindAllArgs { query, sort, size }).await?;
                    log::info!("Find All: count: {},  {:?}", result.len(), result);
                  ::backend_core::$model::Presentation::from_aggregate_roots(tx, operator.as_ref(), result).await
                })
              })
              .map_err(Error::from)
              .await?,
          )
        }

        #[tauri::command]
        async fn [<$model _find_by_id>](
          db: State<'_, DbConn>,
          id: Uuid,
        ) -> Result<Option<::backend_core::$model::Presentation>, String> {
          Ok(
            db.inner()
              .transaction(|tx| {
                Box::pin(async move {
                  let operator = test_get_operator(tx).await?;
                  let result = Repository::<::backend_core::$model::[< $model:camel >]>::find_by_id(tx, id).await?;

                  if let Some(result) = result {
                    Ok(::backend_core::$model::Presentation::from_aggregate_roots(tx, operator.as_ref(), vec![result]).await?.into_iter().last())
                  } else {
                    Ok(None)
                  }
                })
              })
              .map_err(Error::from)
              .await?,
          )
        }
      }
    };
}

generate_tauri_command!(user);
generate_tauri_command!(account);
generate_tauri_command!(journal);
generate_tauri_command!(record);

async fn init() -> anyhow::Result<DbConn> {
  let _ = dotenv::from_filename(".desktop.test.env")?;
  let _ = env_logger::try_init();
  let mut opt: ConnectOptions = env::var("WHITE_RABBIT_DATABASE_URL")?.into();
  opt.max_connections(100).min_connections(5);
  Ok(Database::connect(opt).await?)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let db = init().await?;

  log::info!("Tauri starts");

  tauri::Builder::default()
    .manage(db)
    .invoke_handler(tauri::generate_handler![
      user_handle_command,
      user_find_page,
      user_find_all,
      user_find_by_id,
      account_handle_command,
      account_find_page,
      account_find_all,
      account_find_by_id,
      journal_handle_command,
      journal_find_page,
      journal_find_all,
      journal_find_by_id,
      record_handle_command,
      record_find_page,
      record_find_all,
      record_find_by_id
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
  Ok(())
}
