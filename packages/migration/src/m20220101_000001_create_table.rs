use crate::sea_orm::Schema;

use backend_core::journal::{self, journal_users, Journal};
use backend_core::user::{self, User};
use backend_core::Repository;

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
    manager.create_table(schema.create_table_from_entity(journal_users::Entity)).await?;

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
          &users[0..3],
          &users[3..7],
        )
      })
      .collect::<Vec<_>>();
    let _ = Repository::<Journal>::save(db, journals).await.unwrap();

    Ok(())
  }

  async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
    Ok(())
  }
}
