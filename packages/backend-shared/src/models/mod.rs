pub mod account;
pub mod account_tag;
pub mod auth_id;
pub mod group;
pub mod group_user;
pub mod journal;
pub mod journal_group;
pub mod journal_tag;
pub mod journal_user;
pub mod record;
pub mod record_item;
pub mod record_tag;
pub mod user;

pub use account::Entity as Account;
pub use account_tag::Entity as AccountTag;
pub use auth_id::Entity as AuthId;
pub use group::Entity as Group;
pub use group_user::Entity as GroupUser;
pub use journal::Entity as Journal;
pub use journal_group::Entity as JournalGroup;
pub use journal_tag::Entity as JournalTag;
pub use journal_user::Entity as JournalUser;
pub use record::Entity as Record;
pub use record_item::Entity as RecordItem;
pub use record_tag::Entity as RecordTag;
use sea_orm::{sea_query::TableCreateStatement, ConnectionTrait};
pub use user::Entity as User;

pub async fn setup_schema(db: &sea_orm::DbConn) -> Result<(), sea_orm::DbErr> {
  let schema = sea_orm::Schema::new(sea_orm::DbBackend::Sqlite);

  let stmt: TableCreateStatement = schema.create_table_from_entity(User);
  let _ = db.execute(db.get_database_backend().build(&stmt)).await?;

  let stmt: TableCreateStatement = schema.create_table_from_entity(AuthId);
  let _ = db.execute(db.get_database_backend().build(&stmt)).await?;

  let stmt: TableCreateStatement = schema.create_table_from_entity(Group);
  let _ = db.execute(db.get_database_backend().build(&stmt)).await?;

  let stmt: TableCreateStatement = schema.create_table_from_entity(GroupUser);
  let _ = db.execute(db.get_database_backend().build(&stmt)).await?;

  let stmt: TableCreateStatement = schema.create_table_from_entity(Journal);
  let _ = db.execute(db.get_database_backend().build(&stmt)).await?;

  let stmt: TableCreateStatement = schema.create_table_from_entity(JournalTag);
  let _ = db.execute(db.get_database_backend().build(&stmt)).await?;

  let stmt: TableCreateStatement = schema.create_table_from_entity(JournalUser);
  let _ = db.execute(db.get_database_backend().build(&stmt)).await?;

  let stmt: TableCreateStatement = schema.create_table_from_entity(JournalGroup);
  let _ = db.execute(db.get_database_backend().build(&stmt)).await?;

  let stmt: TableCreateStatement = schema.create_table_from_entity(Account);
  let _ = db.execute(db.get_database_backend().build(&stmt)).await?;

  let stmt: TableCreateStatement = schema.create_table_from_entity(AccountTag);
  let _ = db.execute(db.get_database_backend().build(&stmt)).await?;

  let stmt: TableCreateStatement = schema.create_table_from_entity(Record);
  let _ = db.execute(db.get_database_backend().build(&stmt)).await?;

  let stmt: TableCreateStatement = schema.create_table_from_entity(RecordTag);
  let _ = db.execute(db.get_database_backend().build(&stmt)).await?;

  let stmt: TableCreateStatement = schema.create_table_from_entity(RecordItem);
  let _ = db.execute(db.get_database_backend().build(&stmt)).await?;

  Ok(())
}
