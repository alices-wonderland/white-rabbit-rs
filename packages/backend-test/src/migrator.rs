use crate::m20220916_095218_seed_data;
use migration::m20220101_000001_create_table;
use sea_orm_migration::{async_trait, MigrationTrait, MigratorTrait};

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
  fn migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![
      Box::new(m20220101_000001_create_table::Migration),
      Box::new(m20220916_095218_seed_data::Migration),
    ]
  }
}
