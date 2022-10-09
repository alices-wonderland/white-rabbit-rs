#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

mod services;

use sea_orm::Database;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
  dotenv::from_filename(".desktop.test.env")?;
  let _ = env_logger::try_init();

  tauri::Builder::default()
    .manage(Database::connect(std::env::var("WHITE_RABBIT_DATABASE_URL")?).await?)
    .invoke_handler(services::HANDLERS)
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
  Ok(())
}

#[cfg(test)]
mod tests {
  use backend_test::{Migrator, MigratorTrait};
  use sea_orm::Database;

  #[tokio::test]
  async fn populate_data() -> anyhow::Result<()> {
    dotenv::from_filename(".desktop.test.env")?;
    let _ = env_logger::try_init();

    let db = Database::connect(std::env::var("WHITE_RABBIT_DATABASE_URL")?).await?;
    Migrator::up(&db, None).await?;

    Ok(())
  }
}
