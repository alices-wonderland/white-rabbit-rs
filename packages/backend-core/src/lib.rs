pub mod account;
mod aggregate_root;
mod errors;
pub mod journal;
pub mod record;
mod repository;
pub mod user;
pub mod utils;

pub use aggregate_root::*;
pub use errors::*;
pub use repository::*;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::env;

pub async fn init(filename: &str) -> Result<DatabaseConnection> {
  let _ = dotenv::from_filename(filename);
  let _ = env_logger::try_init();
  let mut opt: ConnectOptions = env::var("WHITE_RABBIT_DATABASE_URL").unwrap().into();
  opt.max_connections(10).min_connections(5);
  let db = Database::connect(opt).await?;
  Ok(db)
}
