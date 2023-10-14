#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

use backend_core::init;
use futures::TryFutureExt;
use sea_orm::TransactionTrait;

macro_rules! generate_handlers {
  ($entity: ident) => {
    paste::paste! {
      #[tauri::command]
      async fn [<$entity _find_by_id>](
        db: ::tauri::State<'_, ::sea_orm::DbConn>,
        id: ::uuid::Uuid,
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
        db: ::tauri::State<'_, ::sea_orm::DbConn>,
        query: ::backend_core::entity::$entity::Query,
      ) -> Result<Vec<::backend_core::entity::$entity::Root>, String> {
        db.inner()
          .transaction(|tx| {
            Box::pin(async move { ::backend_core::entity::$entity::Root::find_all(tx, Some(query), None).await })
          })
          .map_err(|err| err.to_string())
          .await
      }

      #[tauri::command]
      async fn [<$entity _handle_command>](
        db: ::tauri::State<'_, ::sea_orm::DbConn>,
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
generate_handlers!(entry);

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
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
  Ok(())
}
