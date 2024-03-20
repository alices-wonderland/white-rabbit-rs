#![allow(async_fn_in_trait)]

use sea_orm::{ConnectOptions, Database, DatabaseConnection};

use std::env;

pub mod entity;
mod error;

pub use error::*;

pub async fn init(filename: &str) -> Result<DatabaseConnection> {
  let _ = dotenv::from_filename(filename);
  let _ = env_logger::try_init();
  let mut opt: ConnectOptions = env::var("WHITE_RABBIT_DATABASE_URL").unwrap().into();
  opt.max_connections(10).min_connections(5);
  let db = Database::connect(opt).await?;
  Ok(db)
}
