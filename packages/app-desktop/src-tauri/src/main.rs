#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

use backend_core::entity::{account, entry, journal};
use backend_core::init;
use futures::TryFutureExt;
use sea_orm::{DbConn, TransactionTrait};
use std::collections::HashSet;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
async fn journal_find_by_id(
  db: State<'_, DbConn>,
  id: Uuid,
) -> Result<Option<journal::Root>, String> {
  db.inner()
    .transaction(|tx| {
      Box::pin(async move {
        journal::Root::find_one(
          tx,
          Some(journal::Query { id: HashSet::from_iter([id]), ..Default::default() }),
        )
        .await
      })
    })
    .map_err(|err| err.to_string())
    .await
}

#[tauri::command]
async fn journal_find_all(
  db: State<'_, DbConn>,
  query: journal::Query,
) -> Result<Vec<journal::Root>, String> {
  log::info!("Journal Query: {:#?}", query);
  db.inner()
    .transaction(|tx| Box::pin(async move { journal::Root::find_all(tx, Some(query), None).await }))
    .map_err(|err| err.to_string())
    .await
}

#[tauri::command]
async fn account_find_by_id(
  db: State<'_, DbConn>,
  id: Uuid,
) -> Result<Option<account::Root>, String> {
  db.inner()
    .transaction(|tx| {
      Box::pin(async move {
        account::Root::find_one(
          tx,
          Some(account::Query { id: HashSet::from_iter([id]), ..Default::default() }),
        )
        .await
      })
    })
    .map_err(|err| err.to_string())
    .await
}

#[tauri::command]
async fn account_find_all(
  db: State<'_, DbConn>,
  query: account::Query,
) -> Result<Vec<account::Root>, String> {
  log::info!("Account Query: {:#?}", query);
  db.inner()
    .transaction(|tx| Box::pin(async move { account::Root::find_all(tx, Some(query), None).await }))
    .map_err(|err| err.to_string())
    .await
}

#[tauri::command]
async fn entry_find_by_id(db: State<'_, DbConn>, id: Uuid) -> Result<Option<entry::Root>, String> {
  db.inner()
    .transaction(|tx| {
      Box::pin(async move {
        entry::Root::find_one(
          tx,
          Some(entry::Query { id: HashSet::from_iter([id]), ..Default::default() }),
        )
        .await
      })
    })
    .map_err(|err| err.to_string())
    .await
}

#[tauri::command]
async fn entry_find_all(
  db: State<'_, DbConn>,
  query: entry::Query,
) -> Result<Vec<entry::Root>, String> {
  log::info!("Entry Query: {:#?}", query);
  db.inner()
    .transaction(|tx| Box::pin(async move { entry::Root::find_all(tx, Some(query), None).await }))
    .map_err(|err| err.to_string())
    .await
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let db = init(".desktop.test.env").await?;

  tauri::Builder::default()
    .manage(db)
    .invoke_handler(tauri::generate_handler![
      journal_find_by_id,
      journal_find_all,
      account_find_by_id,
      account_find_all,
      entry_find_by_id,
      entry_find_all
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
  Ok(())
}
