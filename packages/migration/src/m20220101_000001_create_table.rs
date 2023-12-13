use crate::sea_orm::Schema;
use backend_core::entity::{
  account, account_tag, entry, entry_item, entry_tag, journal, journal_tag, MAX_DESCRIPTION_LENGTH,
  MAX_NAME_LENGTH, MAX_SHORT_TEXT_LENGTH,
};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    let builder = manager.get_database_backend();
    let schema = Schema::new(builder);

    let table = Table::create()
      .table(journal::Entity)
      .col(ColumnDef::new(journal::Column::Id).uuid().primary_key())
      .col(
        ColumnDef::new(journal::Column::Name)
          .string_len(MAX_NAME_LENGTH as u32)
          .not_null()
          .unique_key(),
      )
      .col(
        ColumnDef::new(journal::Column::Description)
          .string_len(MAX_DESCRIPTION_LENGTH as u32)
          .not_null(),
      )
      .col(
        ColumnDef::new(journal::Column::Unit).string_len(MAX_SHORT_TEXT_LENGTH as u32).not_null(),
      )
      .to_owned();
    manager.create_table(table).await?;

    let idx = Index::create()
      .name("idx-journals-unit")
      .table(journal::Entity)
      .col(journal::Column::Id)
      .col(journal::Column::Unit)
      .unique()
      .to_owned();
    manager.create_index(idx).await?;

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
