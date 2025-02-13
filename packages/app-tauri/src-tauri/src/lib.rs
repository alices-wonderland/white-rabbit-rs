#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

use backend_core::entity::{entry, hierarchy_report, Presentation, ReadRoot};
use backend_core::{init, Error};
use futures::TryFutureExt;
use sea_orm::{DbConn, TransactionError, TransactionTrait};
use std::collections::HashSet;
use tauri::{Emitter, Manager};
use uuid::Uuid;

macro_rules! generate_handlers {
  ($entity: ident) => {
    paste::paste! {
      #[tauri::command]
      async fn [<$entity _find_by_id>](
        db: ::tauri::State<'_, DbConn>,
        id: Uuid,
      ) -> ::backend_core::Result<Option<::backend_core::entity::$entity::Root>> {
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
          .map_err(|err| match err {
            TransactionError::Connection(err) => err.into(),
            TransactionError::Transaction(err) => err,
          })
          .await
      }

      #[tauri::command]
      async fn [<$entity _find_all>](
        db: ::tauri::State<'_, DbConn>,
        query: Option<::backend_core::entity::$entity::Query>,
        size: Option<u64>,
        sort: Option<::backend_core::entity::$entity::Sort>,
      ) -> ::backend_core::Result<Vec<::backend_core::entity::$entity::Root>> {
        db.inner()
          .transaction(|tx| {
            Box::pin(async move { ::backend_core::entity::$entity::Root::find_all(tx, query, size, sort).await })
          })
          .map_err(|err| match err {
            TransactionError::Connection(err) => err.into(),
            TransactionError::Transaction(err) => err,
          })
          .await
      }

      #[tauri::command]
      async fn [<$entity _handle_command>](
        db: ::tauri::State<'_, DbConn>,
        command: ::backend_core::entity::$entity::Command,
      ) -> ::backend_core::Result<Vec<::backend_core::entity::$entity::Root>> {
        db.inner()
          .transaction(|tx| Box::pin(async move { ::backend_core::entity::$entity::Root::handle(tx, command).await }))
          .map_err(|err| match err {
            TransactionError::Connection(err) => err.into(),
            TransactionError::Transaction(err) => err,
          })
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
) -> backend_core::Result<Option<entry::Presentation>> {
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
    .map_err(|err| match err {
      TransactionError::Connection(err) => err.into(),
      TransactionError::Transaction(err) => err,
    })
    .await
}

#[tauri::command]
async fn entry_find_all(
  db: tauri::State<'_, DbConn>,
  query: Option<entry::Query>,
  size: Option<u64>,
  sort: Option<entry::Sort>,
) -> backend_core::Result<Vec<entry::Presentation>> {
  db.inner()
    .transaction(|tx| {
      Box::pin(async move {
        let roots = entry::Root::find_all(tx, query, size, sort).await?;
        entry::Presentation::from_roots(tx, roots).await
      })
    })
    .map_err(|err| match err {
      TransactionError::Connection(err) => err.into(),
      TransactionError::Transaction(err) => err,
    })
    .await
}

#[tauri::command]
async fn entry_handle_command(
  db: tauri::State<'_, DbConn>,
  command: entry::Command,
) -> backend_core::Result<Vec<entry::Presentation>> {
  db.inner()
    .transaction(|tx| {
      Box::pin(async move {
        let roots = entry::Root::handle(tx, command).await?;
        entry::Presentation::from_roots(tx, roots).await
      })
    })
    .map_err(|err| match err {
      TransactionError::Connection(err) => err.into(),
      TransactionError::Transaction(err) => err,
    })
    .await
}

#[tauri::command]
async fn hierarchy_report_find_by_id(
  db: tauri::State<'_, DbConn>,
  id: String,
) -> backend_core::Result<Option<hierarchy_report::Root>> {
  db.inner()
    .transaction(|tx| {
      Box::pin(hierarchy_report::Root::find_one(
        tx,
        Some(hierarchy_report::Query { id: HashSet::from_iter([id]), ..Default::default() }),
      ))
    })
    .map_err(|err| match err {
      TransactionError::Connection(err) => err.into(),
      TransactionError::Transaction(err) => err,
    })
    .await
}

#[tauri::command]
async fn hierarchy_report_find_all(
  db: tauri::State<'_, DbConn>,
  query: Option<hierarchy_report::Query>,
) -> backend_core::Result<Vec<hierarchy_report::Root>> {
  db.inner()
    .transaction(|tx| Box::pin(hierarchy_report::Root::find_all(tx, query, None, None)))
    .map_err(|err| match err {
      TransactionError::Connection(err) => err.into(),
      TransactionError::Transaction(err) => err,
    })
    .await
}

fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
  let handle = app.handle().clone();
  tauri::async_runtime::spawn(async move {
    if let Ok(db) = init(".desktop.test.env").await {
      handle.manage(db);
    } else {
      let _ = handle.emit("local-server-down", ());
      println!("Local Server is not running");
    }
  });
  Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .setup(setup)
    .plugin(tauri_plugin_clipboard_manager::init())
    .plugin(tauri_plugin_global_shortcut::Builder::new().build())
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_process::init())
    .plugin(tauri_plugin_http::init())
    .plugin(tauri_plugin_shell::init())
    .plugin(tauri_plugin_fs::init())
    .plugin(tauri_plugin_notification::init())
    .plugin(tauri_plugin_os::init())
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
}
