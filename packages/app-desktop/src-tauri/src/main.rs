#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

use backend_shared::run;
use std::env;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
  env::set_var("WHITE_RABBIT_DATABASE_URL", "sqlite::memory:");
  env_logger::init();
  log::info!("Run DB in endpoint_desktop: {}", env::var("WHITE_RABBIT_DATABASE_URL")?);
  let _db = run().await?;
  tauri::Builder::default()
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
  Ok(())
}
