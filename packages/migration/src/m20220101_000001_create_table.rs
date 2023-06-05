use crate::sea_orm::Schema;
use backend_core::account::{self, account_tag};
use backend_core::journal::{self, journal_user};
use backend_core::record;
use backend_core::record::{record_item, record_tag};
use backend_core::user::{self};
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

    Ok(())
  }

  async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
    Ok(())
  }
}
