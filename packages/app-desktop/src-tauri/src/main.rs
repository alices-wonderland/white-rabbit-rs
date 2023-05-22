#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

use backend_core::user::User;
use backend_core::{user, AggregateRoot, Error, FindAllArgs, Order, Repository};

use futures::{TryFutureExt, TryStreamExt};
use sea_orm::{ConnectOptions, Database, DbConn, TransactionTrait};
use std::collections::HashSet;
use std::default::Default;
use std::env;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
async fn user_create(
  db: State<'_, DbConn>,
  command: user::CommandCreate,
) -> Result<Option<User>, String> {
  Ok(
    db.inner()
      .transaction(|tx| {
        Box::pin(async move {
          let operator = Repository::<User>::find_all(
            tx,
            FindAllArgs {
              query: user::Query { role: Some(user::Role::Admin), ..Default::default() },
              ..Default::default()
            },
          )
          .await?
          .try_next()
          .await?;
          User::handle(tx, operator.as_ref(), user::Command::Create(command)).await
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
) -> Result<Vec<User>, String> {
  Ok(
    db.inner()
      .transaction(|tx| {
        Box::pin(async move {
          let operator = Repository::<User>::find_all(
            tx,
            FindAllArgs {
              query: user::Query { role: Some(user::Role::Admin), ..Default::default() },
              ..Default::default()
            },
          )
          .await?
          .try_next()
          .await?;

          let result = Repository::<User>::find_all(
            tx,
            FindAllArgs {
              operator: operator.as_ref(),
              query: user::Query { id, name: (name, true), role },
              sort,
            },
          )
          .await?
          .try_collect::<Vec<_>>()
          .await?;

          Ok(result)
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

#[cfg(test)]
mod tests {
  use backend_core::account::Account;
  use backend_core::journal::Journal;
  use backend_core::user::User;
  use backend_core::{account, journal, user, AggregateRoot, FindAllArgs, Repository};

  use futures::TryStreamExt;
  use migration::{Migrator, MigratorTrait};
  use std::collections::HashSet;

  #[tokio::test]
  async fn populate_data() -> anyhow::Result<()> {
    let db = crate::init().await?;

    Migrator::up(&db, None).await?;

    let admins = Repository::<User>::find_all(
      &db,
      FindAllArgs {
        query: user::Query { name: ("User 1".to_string(), true), ..Default::default() },
        ..Default::default()
      },
    )
    .await?
    .try_collect::<Vec<_>>()
    .await?;
    log::info!("Admin len: {}", admins.len());

    let members = Repository::<User>::find_all(
      &db,
      FindAllArgs {
        query: user::Query { name: ("User 2".to_string(), true), ..Default::default() },
        ..Default::default()
      },
    )
    .await?
    .try_collect::<Vec<_>>()
    .await?;
    log::info!("Member len: {}", members.len());

    let journals = Repository::<Journal>::find_all(
      &db,
      FindAllArgs {
        query: journal::Query {
          name: ("Journal".to_string(), true),
          description: "Desc".to_string(),
          admin: admins.iter().map(|user| user.id()).collect(),
          member: members.iter().map(|user| user.id()).collect(),
          ..Default::default()
        },
        ..Default::default()
      },
    )
    .await?
    .try_collect::<Vec<_>>()
    .await?;
    log::info!("Journal len: {}", journals.len());
    for result in &journals {
      log::info!("Journal: {:#?}", result);
      let admins = Repository::<User>::find_by_ids(&db, result.admins.clone()).await?;
      for user in admins {
        log::info!("  Admin: {:?}", user);
      }
      let members = Repository::<User>::find_by_ids(&db, result.members.clone()).await?;
      for user in members {
        log::info!("  Member: {:?}", user);
      }
    }

    let query = account::Query {
      name: ("Account".to_string(), true),
      description: "Desc".to_string(),
      tag: "tag".to_string(),
      journal: HashSet::from_iter(vec![journals[0].id()]),
      parent: HashSet::from_iter(vec![None]),
      ..Default::default()
    };

    let parents = Repository::<Account>::find_all(
      &db,
      FindAllArgs { query: query.clone(), ..Default::default() },
    )
    .await?
    .try_collect::<Vec<_>>()
    .await?;
    log::info!("Parent len: {}", parents.len());
    for result in &parents {
      log::info!("  Account: {:#?}", result);
    }

    let children = Repository::<Account>::find_all(
      &db,
      FindAllArgs {
        query: account::Query {
          parent: HashSet::from_iter(vec![Some(parents[0].id), None]),
          ..query
        },
        ..Default::default()
      },
    )
    .await?
    .try_collect::<Vec<_>>()
    .await?;
    log::info!("Children len: {}", children.len());
    for result in &children {
      log::info!("  Account: {:#?}", result);
    }
    Ok(())
  }
}
