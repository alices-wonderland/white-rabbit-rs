pub use anyhow::Result;
use backend_core::entity::{
  account, entry, journal, MAX_SHORT_TEXT_LENGTH, MAX_TAGS_LENGTH, MIN_SHORT_TEXT_LENGTH,
};
use fake::faker::chrono::en::Date;
use fake::faker::company::en::CompanyName;
use fake::faker::currency::en::CurrencyCode;
use fake::faker::finance::en::Bic;
use fake::faker::lorem::en::{Paragraph, Words};
use fake::Fake;
use itertools::Itertools;
use migration::sea_orm::{DbConn, Iterable};
use migration::{Migrator, MigratorTrait};
use rand::seq::{IteratorRandom, SliceRandom};
use rand::Rng;
use rust_decimal::Decimal;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

fn gen_tags() -> HashSet<String> {
  Words(0..16)
    .fake::<Vec<_>>()
    .into_iter()
    .filter(|tag| tag.len() >= MIN_SHORT_TEXT_LENGTH && tag.len() <= MAX_SHORT_TEXT_LENGTH)
    .take(MAX_TAGS_LENGTH)
    .collect()
}

fn gen_entry_items(accounts: &[account::Root]) -> HashMap<Uuid, (Decimal, Option<Decimal>)> {
  let mut rng = rand::thread_rng();
  accounts
    .choose_multiple(&mut rng, 3)
    .map(|account| {
      let price: Option<Decimal> =
        if rng.gen_bool(0.5) { None } else { Some(rng.gen_range(1..100).into()) };
      (account.id, (rng.gen_range(10..100).into(), price))
    })
    .collect()
}

pub async fn init() -> backend_core::Result<DbConn> {
  let mut rng = rand::thread_rng();
  let db = backend_core::init(".test.env").await?;
  Migrator::up(&db, None).await?;

  let commands = (0..3)
    .map(|_| journal::CommandCreate {
      name: CompanyName().fake(),
      description: Paragraph(0..rng.gen_range(5..8)).fake(),
      unit: CurrencyCode().fake(),
      tags: gen_tags(),
    })
    .collect();
  let journals = journal::Root::create(&db, commands).await?;

  let commands = journals
    .iter()
    .flat_map(|journal| {
      let mut results = account::Type::iter()
        .map(|typ| account::CommandCreate {
          journal_id: journal.id,
          name: format!("{} - {}", journal.name.clone(), typ),
          description: Paragraph(0..10).fake(),
          unit: journal.unit.clone(),
          typ,
          tags: gen_tags(),
        })
        .collect::<Vec<_>>();

      for _ in 0..3 {
        results.push(account::CommandCreate {
          journal_id: journal.id,
          name: CompanyName().fake(),
          description: Paragraph(0..10).fake(),
          unit: CurrencyCode().fake(),
          typ: account::Type::iter().choose(&mut rng).unwrap(),
          tags: gen_tags(),
        })
      }

      results
    })
    .collect();
  let accounts = account::Root::create(&db, commands)
    .await?
    .into_iter()
    .into_group_map_by(|account| account.journal_id);

  let commands = journals
    .iter()
    .flat_map(|journal| {
      if let Some(accounts) = accounts.get(&journal.id) {
        let mut results: Vec<_> = entry::Type::iter()
          .map(|typ| entry::CommandCreate {
            journal_id: journal.id,
            name: format!("{} - {}", journal.name, typ),
            description: Paragraph(0..10).fake(),
            typ,
            date: Date().fake(),
            tags: gen_tags(),
            items: gen_entry_items(accounts),
          })
          .collect();

        for _ in 0..10 {
          results.push(entry::CommandCreate {
            journal_id: journal.id,
            name: Bic().fake(),
            description: Paragraph(0..10).fake(),
            typ: entry::Type::iter().choose(&mut rng).unwrap(),
            date: Date().fake(),
            tags: gen_tags(),
            items: gen_entry_items(accounts),
          })
        }

        results
      } else {
        vec![]
      }
    })
    .collect();
  let _ = entry::Root::create(&db, commands).await?;

  Ok(db)
}

#[macro_export]
macro_rules! generate_tests {
  ($runner: ident; $package: ident; $( $func: ident ),*) => {
    $(
    #[tokio::test]
    async fn $func() -> ::test_suite::Result<()> {
      ::test_suite::$package::$func($runner).await
    }
    )*
  };
}
