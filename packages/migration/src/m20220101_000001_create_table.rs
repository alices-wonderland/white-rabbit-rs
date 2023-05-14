use crate::sea_orm::Schema;
use backend_core::UserEntity;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    let builder = manager.get_database_backend();
    let schema = Schema::new(builder);
    manager.create_table(schema.create_table_from_entity(UserEntity)).await?;
    Ok(())
  }

  async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
    Ok(())
  }
}
