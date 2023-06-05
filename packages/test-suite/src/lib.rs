use backend_core::account::Account;
use backend_core::journal::Journal;
use backend_core::record::{Record, RecordItem};
use backend_core::user::User;
use backend_core::{account, record, user, AggregateRoot, Repository};
use chrono::{Duration, NaiveDate};
use migration::sea_orm::DatabaseConnection;
use migration::{Migrator, MigratorTrait};
use rand::prelude::*;

mod test_runner;
pub mod user_test;

pub use anyhow::Result;
pub use test_runner::TestRunner;

#[macro_export]
macro_rules! generate_tests {
  ($runner: path; $package: ident; $( $func:ident ),*) => {
    $(
    #[tokio::test]
    async fn $func() -> ::test_suite::Result<()> {
      ::test_suite::$package::$func::<$runner>().await
    }
    )*
  };
}

async fn init() -> Result<DatabaseConnection> {
  let db = backend_core::init(".test.env").await?;
  Migrator::up(&db, None).await?;

  Ok(db)
}

async fn populate_data(db: &DatabaseConnection) -> Result<()> {
  let users = (0..10)
    .map(|idx| {
      User::new(
        format!("User {}", idx),
        if idx % 3 == 0 { user::Role::Admin } else { user::Role::User },
      )
    })
    .collect::<Vec<_>>();
  let mut users = Repository::<User>::save(db, users).await.unwrap();

  let journals = (0..3)
    .map(|idx| {
      users.shuffle(&mut thread_rng());
      Journal::new(
        format!("Journal {}", idx),
        format!("Desc {}", idx),
        "CNY",
        &users[0..3],
        &users[3..7],
      )
    })
    .collect::<Vec<_>>();
  let journals = Repository::<Journal>::save(db, journals).await.unwrap();

  let accounts = journals
    .iter()
    .flat_map(|journal| {
      let mut accounts = (0..3)
        .map(|idx| {
          Account::new(
            format!("{} - Account {}", journal.name, idx),
            format!("Desc {}", idx),
            "CNY",
            if idx == 0 { account::Type::Asset } else { account::Type::Expense },
            (0..3).map(|tag| format!("tag {}", tag + idx)),
            journal,
            None,
          )
        })
        .collect::<Vec<_>>();
      accounts[0].parent = Some(accounts[1].id());
      accounts[1].parent = Some(accounts[2].id());
      accounts
    })
    .collect::<Vec<_>>();
  let mut accounts = Repository::<Account>::save(db, accounts).await.unwrap();

  let mut records = Vec::new();
  for journal in &journals {
    for idx in 0..5 {
      accounts.shuffle(&mut thread_rng());
      records.push(Record::new(
        journal,
        format!("Journal {} - {}", &journal.name, idx),
        format!("Desc {}", idx),
        if idx % 3 == 0 { record::Type::Check } else { record::Type::Record },
        NaiveDate::from_ymd_opt(2023, 1, 1).unwrap() + Duration::days(idx * 30),
        (0..3).map(|tag| format!("tag {}", tag + idx)),
        accounts[0..2]
          .iter()
          .map(|account| RecordItem::new(account, (10 * idx).into(), Some("1.5".parse().unwrap()))),
      ));
    }
  }
  let _records = Repository::<Record>::save(db, records).await.unwrap();

  Ok(())
}
