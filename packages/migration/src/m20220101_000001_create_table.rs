use crate::sea_orm::Schema;
use backend_core::account::{self, account_tag, Account};
use backend_core::journal::{self, journal_user, Journal};
use backend_core::record::{record_item, record_tag, Record, RecordItem, Type};
use backend_core::user::{self, User};
use backend_core::{record, AggregateRoot, Repository};
use chrono::{Duration, NaiveDate};
use rand::prelude::*;

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    let builder = manager.get_database_backend();
    let schema = Schema::new(builder);

    manager.create_table(schema.create_table_from_entity(user::Entity)).await?;
    manager.create_table(schema.create_table_from_entity(journal::Entity)).await?;
    manager.create_table(schema.create_table_from_entity(journal_user::Entity)).await?;
    manager.create_table(schema.create_table_from_entity(account::Entity)).await?;
    manager.create_table(schema.create_table_from_entity(account_tag::Entity)).await?;
    manager.create_table(schema.create_table_from_entity(record::Entity)).await?;
    manager.create_table(schema.create_table_from_entity(record_item::Entity)).await?;
    manager.create_table(schema.create_table_from_entity(record_tag::Entity)).await?;

    let db = manager.get_connection();

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
          if idx % 3 == 0 { Type::Check } else { Type::Record },
          NaiveDate::from_ymd_opt(2023, 1, 1).unwrap() + Duration::days(idx * 30),
          (0..3).map(|tag| format!("tag {}", tag + idx)),
          accounts[0..2].iter().map(|account| {
            RecordItem::new(account, (10 * idx).into(), Some("1.5".parse().unwrap()))
          }),
        ));
      }
    }
    let _records = Repository::<Record>::save(db, records).await.unwrap();

    Ok(())
  }

  async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
    Ok(())
  }
}
