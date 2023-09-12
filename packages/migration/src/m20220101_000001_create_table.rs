use crate::sea_orm::Schema;
use backend_core::entity::{
  account, account_tag, entry, entry_item, entry_tag, journal, journal_tag,
};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    let builder = manager.get_database_backend();
    let schema = Schema::new(builder);

    manager.create_table(schema.create_table_from_entity(journal::Entity)).await?;
    manager.create_table(schema.create_table_from_entity(journal_tag::Entity)).await?;
    manager.create_table(schema.create_table_from_entity(account::Entity)).await?;
    manager.create_table(schema.create_table_from_entity(account_tag::Entity)).await?;
    manager.create_table(schema.create_table_from_entity(entry::Entity)).await?;
    manager.create_table(schema.create_table_from_entity(entry_item::Entity)).await?;
    manager.create_table(schema.create_table_from_entity(entry_tag::Entity)).await?;

    Ok(())
  }

  async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
    Ok(())
  }
}
