use crate::entity::entry::Type;
use crate::entity::journal::Root;
use crate::entity::{
  account, account_tag, entry, entry_item, entry_tag, journal, journal_tag, FIELD_NAME,
};
use chrono::{NaiveDate, Utc};
use itertools::Itertools;
use rust_decimal_macros::dec;
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection, EntityTrait, Schema};
use std::collections::HashSet;
use std::env;
use strum::IntoEnumIterator;

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
  let _ = db.execute(builder.build(&schema.create_table_from_entity(entry_item::Entity))).await;
  let _ = db.execute(builder.build(&schema.create_table_from_entity(entry_tag::Entity))).await;

  let now = Utc::now();
  let journals: Vec<journal::Root> = vec![
    journal::Root::new(
      None,
      format!("Name 1 - {}", now),
      "desc 1",
      "unit 1",
      vec!["tag1", "  tag2"],
    ),
    journal::Root::new(
      None,
      format!("  Name 2 - {}  ", now),
      "  desc 2",
      "unit 3  ",
      vec!["tag1", "   ", "  tag2"],
    ),
  ]
  .into_iter()
  .try_collect()?;

  let journal_id = journals.get(0).map(|model| model.id).unwrap();
  Root::save(&db, journals).await?;

  let _ = journal::Entity::delete_by_id(journal_id).exec(&db).await;

  let journal = journal::Root::find_one(
    &db,
    Some(journal::Query {
      full_text: ("name".to_string(), HashSet::from_iter([FIELD_NAME.to_string()])),
      ..Default::default()
    }),
  )
  .await?
  .unwrap();

  println!("Journal: {:#?}", journal);

  let journals = journal::Root::find_all(&db, None, None).await?;
  let accounts: Vec<account::Root> = journals
    .iter()
    .flat_map(|journal| {
      account::Type::iter().map(|typ| {
        account::Root::new(
          journal.id,
          format!("{} - {}", journal.name, typ),
          "Desc",
          "Unit ",
          typ,
          vec!["Tag 1", "", "Tag 2"],
        )
      })
    })
    .try_collect()?;
  account::Root::save(&db, accounts).await?;
  let account = account::Root::find_one(
    &db,
    Some(account::Query { full_text: ("a".to_string(), HashSet::default()), ..Default::default() }),
  )
  .await?
  .unwrap();
  println!("Account: {:#?}", account);

  let mut entries = vec![];
  for ref journal in journals {
    let accounts = account::Root::find_all(
      &db,
      Some(account::Query { journal_id: HashSet::from_iter([journal.id]), ..Default::default() }),
      None,
    )
    .await?;

    for (cnt, account_window) in accounts.windows(2).enumerate() {
      entries.push(entry::Root::new(
        journal.id,
        format!("{} - {}", journal.name, cnt),
        "Desc",
        Type::Record,
        NaiveDate::from_ymd_opt(2023, 12, 31).unwrap(),
        vec!["", "  ", "Tag 1"],
        vec![
          (account_window[0].id, (dec!(0.1), Some(dec!(0.2)))),
          (account_window[1].id, (dec!(0.2), Some(dec!(0.4)))),
        ],
      )?);
    }
  }

  entry::Root::save(&db, entries).await?;
  let entry = entry::Root::find_one(
    &db,
    Some(entry::Query {
      full_text: ("de".to_string(), Default::default()),
      account_id: HashSet::from_iter([account.id]),
      ..Default::default()
    }),
  )
  .await?
  .unwrap();
  println!("Entry: {:#?}", entry);

  Ok(db)
}
