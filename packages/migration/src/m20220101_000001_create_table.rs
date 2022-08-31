use backend_shared::models::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    let schema = sea_orm::Schema::new(manager.get_database_backend());

    let tables = vec![
      schema.create_table_from_entity(User),
      schema.create_table_from_entity(AuthId),
      schema.create_table_from_entity(Group),
      schema.create_table_from_entity(GroupUser),
      schema.create_table_from_entity(Journal),
      schema.create_table_from_entity(JournalTag),
      schema.create_table_from_entity(JournalUser),
      schema.create_table_from_entity(JournalGroup),
      schema.create_table_from_entity(Account),
      schema.create_table_from_entity(AccountTag),
      schema.create_table_from_entity(Record),
      schema.create_table_from_entity(RecordTag),
      schema.create_table_from_entity(RecordItem),
    ];

    for table in tables {
      let _ = manager.create_table(table).await?;
    }

    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager.drop_table(Table::drop().table(RecordItem).to_owned()).await?;
    manager.drop_table(Table::drop().table(RecordTag).to_owned()).await?;
    manager.drop_table(Table::drop().table(AccountTag).to_owned()).await?;
    manager.drop_table(Table::drop().table(JournalGroup).to_owned()).await?;
    manager.drop_table(Table::drop().table(JournalUser).to_owned()).await?;
    manager.drop_table(Table::drop().table(JournalTag).to_owned()).await?;
    manager.drop_table(Table::drop().table(GroupUser).to_owned()).await?;
    manager.drop_table(Table::drop().table(AuthId).to_owned()).await?;
    manager.drop_table(Table::drop().table(Record).to_owned()).await?;
    manager.drop_table(Table::drop().table(Account).to_owned()).await?;
    manager.drop_table(Table::drop().table(Journal).to_owned()).await?;
    manager.drop_table(Table::drop().table(Group).to_owned()).await?;
    manager.drop_table(Table::drop().table(User).to_owned()).await?;
    Ok(())
  }
}
