use crate::entity::{account, account_tag, entry, entry_item, entry_tag, journal, journal_tag};

use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection, Schema};

use std::env;

pub mod entity;
mod error;

pub use error::*;

pub async fn init(filename: &str) -> anyhow::Result<DatabaseConnection> {
  let _ = dotenv::from_filename(filename);
  let _ = env_logger::try_init();
  let mut opt: ConnectOptions = env::var("WHITE_RABBIT_DATABASE_URL").unwrap().into();
  opt.max_connections(10).min_connections(5);
  let db = Database::connect(opt).await?;

  let builder = db.get_database_backend();
  let schema = Schema::new(builder);

  let _ = db.execute(builder.build(&schema.create_table_from_entity(journal::Entity))).await;
  let _ = db.execute(builder.build(&schema.create_table_from_entity(journal_tag::Entity))).await;
  let _ = db.execute(builder.build(&schema.create_table_from_entity(account::Entity))).await;
  let _ = db.execute(builder.build(&schema.create_table_from_entity(account_tag::Entity))).await;
  let _ = db.execute(builder.build(&schema.create_table_from_entity(entry::Entity))).await;
  let _ = db.execute(builder.build(&schema.create_table_from_entity(entry_tag::Entity))).await;
  let _ = db.execute(builder.build(&schema.create_table_from_entity(entry_item::Entity))).await;

  Ok(db)
}
