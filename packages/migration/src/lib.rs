mod m20220101_000001_create_table;
mod m20220916_095218_seed_data;
mod migrator;
mod test_migrator;

pub use migrator::Migrator;
pub use sea_orm_migration::MigratorTrait;
pub use test_migrator::TestMigrator;
