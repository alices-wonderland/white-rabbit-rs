pub mod m20220101_000001_create_table;
mod migrator;

pub use migrator::Migrator;
pub use sea_orm_migration::MigratorTrait;
