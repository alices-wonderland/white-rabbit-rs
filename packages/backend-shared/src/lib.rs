pub mod models;

use sea_orm::{Database, DatabaseConnection};
use std::env;

pub async fn run() -> Result<DatabaseConnection, anyhow::Error> {
  Ok(Database::connect(env::var("WHITE_RABBIT_DATABASE_URL")?).await?)
}

#[cfg(test)]
pub mod tests {
  use crate::{models, run};
  use sea_orm::prelude::Uuid;
  use sea_orm::sea_query::TableCreateStatement;
  use sea_orm::*;
  use std::env;

  async fn setup_schema(db: &DbConn) -> Result<(), DbErr> {
    let schema = Schema::new(DbBackend::Sqlite);

    let stmt: TableCreateStatement = schema.create_table_from_entity(models::user::Entity);
    let result = db.execute(db.get_database_backend().build(&stmt)).await?;
    log::info!("Finish Create Table: {:?}", result);

    let stmt: TableCreateStatement = schema.create_table_from_entity(models::auth_id::Entity);
    let result = db.execute(db.get_database_backend().build(&stmt)).await?;
    log::info!("Finish Create Table: {:?}", result);

    Ok(())
  }

  #[tokio::test]
  async fn my_test() -> Result<(), anyhow::Error> {
    env::set_var("WHITE_RABBIT_DATABASE_URL", "sqlite::memory:");
    env::set_var("RUST_LOG", "info");
    env_logger::init();
    let db = run().await?;
    setup_schema(&db).await?;

    let manager = models::user::ActiveModel {
      name: Set("Manager 1".to_owned()),
      role: Set(models::user::Role::Admin),
      ..Default::default()
    };
    let manager: models::user::ActiveModel = manager.save(&db).await?;

    let manager_auth_ids = vec![
      models::auth_id::ActiveModel {
        user_id: Set(manager.id.clone().unwrap()),
        provider: Set("provider 1".to_string()),
        value: Set(Uuid::new_v4().to_string()),
      },
      models::auth_id::ActiveModel {
        user_id: Set(manager.id.clone().unwrap()),
        provider: Set("provider 2".to_string()),
        value: Set(Uuid::new_v4().to_string()),
      },
      models::auth_id::ActiveModel {
        user_id: Set(manager.id.clone().unwrap()),
        provider: Set("provider 3".to_string()),
        value: Set(Uuid::new_v4().to_string()),
      },
    ];
    let manager_user_ids: InsertResult<_> = models::auth_id::Entity::insert_many(manager_auth_ids).exec(&db).await?;
    log::info!("manager_auth_ids: {:#?}", manager_user_ids);

    let user = models::user::ActiveModel {
      name: Set("User 1".to_owned()),
      role: Set(models::user::Role::User),
      ..Default::default()
    };
    let user: models::user::ActiveModel = user.save(&db).await?;

    let user_auth_ids = vec![
      models::auth_id::ActiveModel {
        user_id: Set(user.id.clone().unwrap()),
        provider: Set("provider 1".to_string()),
        value: Set(Uuid::new_v4().to_string()),
      },
      models::auth_id::ActiveModel {
        user_id: Set(user.id.clone().unwrap()),
        provider: Set("provider 2".to_string()),
        value: Set(Uuid::new_v4().to_string()),
      },
      models::auth_id::ActiveModel {
        user_id: Set(user.id.clone().unwrap()),
        provider: Set("provider 3".to_string()),
        value: Set(Uuid::new_v4().to_string()),
      },
    ];
    let user_user_ids: InsertResult<_> = models::auth_id::Entity::insert_many(user_auth_ids).exec(&db).await?;
    log::info!("user_auth_ids: {:#?}", user_user_ids);

    let users = models::user::Entity::find()
      .find_with_related(models::auth_id::Entity)
      .order_by_desc(models::user::Column::Name)
      .order_by_desc(models::auth_id::Column::Provider)
      .all(&db)
      .await?;
    log::info!("users: {:#?}", users);

    Ok(())
  }
}
