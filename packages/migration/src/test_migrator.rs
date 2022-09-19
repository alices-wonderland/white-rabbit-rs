use crate::{m20220101_000001_create_table, m20220916_095218_seed_data};
use sea_orm_migration::{async_trait, MigrationTrait, MigratorTrait};

pub struct TestMigrator;

#[async_trait::async_trait]
impl MigratorTrait for TestMigrator {
  fn migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![
      Box::new(m20220101_000001_create_table::Migration),
      Box::new(m20220916_095218_seed_data::Migration),
    ]
  }
}
