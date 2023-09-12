#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

use backend_core::init;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let db = init(".desktop.test.env").await?;

  tauri::Builder::default()
    .manage(db)
    .invoke_handler(tauri::generate_handler![])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
  Ok(())
}
