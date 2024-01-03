#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

use backend_core::entity::{entry, hierarchy_report, Presentation};
use backend_core::{init, Error};
use futures::TryFutureExt;
use sea_orm::{DbConn, TransactionTrait};
use std::collections::HashSet;
use uuid::Uuid;

macro_rules! generate_handlers {
  ($entity: ident) => {
    paste::paste! {
      #[tauri::command]
      async fn [<$entity _find_by_id>](
        db: ::tauri::State<'_, DbConn>,
        id: Uuid,
      ) -> Result<Option<::backend_core::entity::$entity::Root>, String> {
        db.inner()
          .transaction(|tx| {
            Box::pin(async move {
              ::backend_core::entity::$entity::Root::find_one(
                tx,
                Some(::backend_core::entity::$entity::Query { id: ::std::collections::HashSet::from_iter([id]), ..Default::default() }),
              )
              .await
            })
          })
          .map_err(|err| err.to_string())
          .await
      }

      #[tauri::command]
      async fn [<$entity _find_all>](
        db: ::tauri::State<'_, DbConn>,
        query: Option<::backend_core::entity::$entity::Query>,
        size: Option<u64>,
        sort: Option<::backend_core::entity::$entity::Sort>,
      ) -> Result<Vec<::backend_core::entity::$entity::Root>, String> {
        db.inner()
          .transaction(|tx| {
            Box::pin(async move { ::backend_core::entity::$entity::Root::find_all(tx, query, size, sort).await })
          })
          .map_err(|err| err.to_string())
          .await
      }

      #[tauri::command]
      async fn [<$entity _handle_command>](
        db: ::tauri::State<'_, DbConn>,
        command: ::backend_core::entity::$entity::Command,
      ) -> Result<Vec<::backend_core::entity::$entity::Root>, String> {
        db.inner()
          .transaction(|tx| Box::pin(async move { ::backend_core::entity::$entity::Root::handle(tx, command).await }))
          .map_err(|err| err.to_string())
          .await
      }
    }
  };
}

generate_handlers!(journal);
generate_handlers!(account);

#[tauri::command]
async fn entry_find_by_id(
  db: tauri::State<'_, DbConn>,
  id: Uuid,
) -> Result<Option<entry::Presentation>, String> {
  db.inner()
    .transaction::<_, _, Error>(|tx| {
      Box::pin(async move {
        let root: Vec<_> = entry::Root::find_one(
          tx,
          Some(entry::Query {
            id: ::std::collections::HashSet::from_iter([id]),
            ..Default::default()
          }),
        )
        .await?
        .into_iter()
        .collect();
        Ok(entry::Presentation::from_roots(tx, root).await?.into_iter().last())
      })
    })
    .map_err(|err| err.to_string())
    .await
}

#[tauri::command]
async fn entry_find_all(
  db: tauri::State<'_, DbConn>,
  query: Option<entry::Query>,
  size: Option<u64>,
  sort: Option<entry::Sort>,
) -> Result<Vec<entry::Presentation>, String> {
  db.inner()
    .transaction(|tx| {
      Box::pin(async move {
        let roots = entry::Root::find_all(tx, query, size, sort).await?;
        entry::Presentation::from_roots(tx, roots).await
      })
    })
    .map_err(|err| err.to_string())
    .await
}

#[tauri::command]
async fn entry_handle_command(
  db: tauri::State<'_, DbConn>,
  command: entry::Command,
) -> Result<Vec<entry::Presentation>, String> {
  db.inner()
    .transaction(|tx| {
      Box::pin(async move {
        let roots = entry::Root::handle(tx, command).await?;
        entry::Presentation::from_roots(tx, roots).await
      })
    })
    .map_err(|err| err.to_string())
    .await
}

#[tauri::command]
async fn hierarchy_report_find_by_id(
  db: tauri::State<'_, DbConn>,
  id: String,
) -> Result<Option<hierarchy_report::Root>, String> {
  db.inner()
    .transaction(|tx| {
      Box::pin(hierarchy_report::Root::find_one(
        tx,
        Some(hierarchy_report::Query { id: HashSet::from_iter([id]), ..Default::default() }),
      ))
    })
    .map_err(|err| err.to_string())
    .await
}

#[tauri::command]
async fn hierarchy_report_find_all(
  db: tauri::State<'_, DbConn>,
  query: Option<hierarchy_report::Query>,
) -> Result<Vec<hierarchy_report::Root>, String> {
  db.inner()
    .transaction(|tx| Box::pin(hierarchy_report::Root::find_all(tx, query)))
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
      journal_handle_command,
      account_find_by_id,
      account_find_all,
      account_handle_command,
      entry_find_by_id,
      entry_find_all,
      entry_handle_command,
      hierarchy_report_find_by_id,
      hierarchy_report_find_all,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
  Ok(())
}
