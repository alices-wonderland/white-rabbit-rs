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

    let idx = Index::create()
      .name("idx-account-journal_id-name")
      .table(account::Entity)
      .col(account::Column::JournalId)
      .col(account::Column::Name)
      .unique()
      .to_owned();
    manager.create_index(idx).await?;

    let idx = Index::create()
      .name("idx-entry-journal_id-name")
      .table(entry::Entity)
      .col(entry::Column::JournalId)
      .col(entry::Column::Name)
      .unique()
      .to_owned();
    manager.create_index(idx).await?;

    Ok(())
  }
}
