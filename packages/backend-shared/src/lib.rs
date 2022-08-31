pub mod models;

use sea_orm::{Database, DatabaseConnection};
use std::env;

pub async fn run() -> Result<DatabaseConnection, anyhow::Error> {
  Ok(Database::connect(env::var("WHITE_RABBIT_DATABASE_URL")?).await?)
}

#[cfg(test)]
pub mod tests {
  use crate::{models, run};
  use chrono::Utc;
  use migration::{Migrator, MigratorTrait};
  use rust_decimal_macros::dec;
  use sea_orm::prelude::Uuid;
  use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, InsertResult, ModelTrait, QueryFilter, QueryOrder, Set};
  use std::env;

  #[tokio::test]
  async fn my_test() -> Result<(), anyhow::Error> {
    env::set_var("WHITE_RABBIT_DATABASE_URL", "sqlite::memory:");
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    let db = run().await?;
    Migrator::up(&db, None).await?;

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
    let manager_user_ids = models::AuthId::insert_many(manager_auth_ids).exec(&db).await?;
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
    let user_user_ids = models::AuthId::insert_many(user_auth_ids).exec(&db).await?;
    log::info!("user_auth_ids: {:#?}", user_user_ids);

    let user2 = models::user::ActiveModel {
      name: Set("User 2".to_owned()),
      role: Set(models::user::Role::Admin),
      ..Default::default()
    };
    let user2 = user2.save(&db).await?;

    let user2_auth_ids = vec![
      models::auth_id::ActiveModel {
        user_id: Set(user2.id.clone().unwrap()),
        provider: Set("provider 1".to_string()),
        value: Set(Uuid::new_v4().to_string()),
      },
      models::auth_id::ActiveModel {
        user_id: Set(user2.id.clone().unwrap()),
        provider: Set("provider 2".to_string()),
        value: Set(Uuid::new_v4().to_string()),
      },
      models::auth_id::ActiveModel {
        user_id: Set(user2.id.clone().unwrap()),
        provider: Set("provider 3".to_string()),
        value: Set(Uuid::new_v4().to_string()),
      },
    ];
    let user2_user_ids: InsertResult<_> = models::AuthId::insert_many(user2_auth_ids).exec(&db).await?;
    log::info!("user2_auth_ids: {:#?}", user2_user_ids);

    let users = models::User::find()
      .find_with_related(models::AuthId)
      .order_by_desc(models::user::Column::Name)
      .order_by_desc(models::auth_id::Column::Provider)
      .all(&db)
      .await?;
    log::info!("users: {:#?}", users);
    assert_eq!(users.len(), 3);
    for (_, auth_ids) in &users {
      assert_eq!(auth_ids.len(), 3);
    }

    let result = models::User::delete_by_id(manager.id.clone().unwrap())
      .exec(&db)
      .await?;
    assert_eq!(result.rows_affected, 1);

    let user_auth_ids: Vec<models::auth_id::Model> = models::AuthId::find()
      .filter(models::auth_id::Column::UserId.eq(manager.id.clone().unwrap()))
      .all(&db)
      .await?;
    assert!(user_auth_ids.is_empty());

    let group = models::group::ActiveModel {
      name: Set("Group 1".to_owned()),
      description: Set("Description".to_owned()),
      ..Default::default()
    };

    let group = group.save(&db).await?;
    let users = models::User::find().all(&db).await?;
    let members: Vec<models::group_user::ActiveModel> = users
      .iter()
      .map(|u| models::group_user::ActiveModel {
        group_id: Set(group.id.clone().unwrap()),
        user_id: Set(u.id),
        is_admin: Set(u.role != models::user::Role::User),
      })
      .collect();
    assert_eq!(members.len(), 2);

    let _: InsertResult<_> = models::GroupUser::insert_many(members.clone()).exec(&db).await?;

    let _ = models::GroupUser::find().all(&db).await?;

    let group = models::Group::find_by_id(group.id.clone().unwrap())
      .one(&db)
      .await?
      .unwrap();
    let group_admins = group.find_linked(models::group::GroupAdmin).all(&db).await?;
    log::info!("group_admins: {:#?}", group_admins);
    assert_eq!(group_admins.len(), 1);
    assert_eq!(group_admins[0].id, members[1].user_id.clone().unwrap());

    let group_members = group.find_linked(models::group::GroupMember).all(&db).await?;
    log::info!("group_members: {:#?}", group_members);
    assert_eq!(group_members.len(), 1);
    assert_eq!(group_members[0].id, members[0].user_id.clone().unwrap());

    let journal = models::journal::ActiveModel {
      name: Set("Journal 1".to_owned()),
      description: Set("Journal 1 description".to_owned()),
      unit: Set("UNIT".to_owned()),
      ..Default::default()
    };
    let journal = journal.save(&db).await?;

    let journal_tags: Vec<_> = vec!["tag 1", "tag 2"]
      .iter()
      .map(|tag| models::journal_tag::ActiveModel {
        journal_id: Set(journal.id.clone().unwrap()),
        tag: Set(tag.to_string()),
      })
      .collect();
    let _ = models::JournalTag::insert_many(journal_tags).exec(&db).await?;

    let journal = models::Journal::find_by_id(journal.id.clone().unwrap())
      .one(&db)
      .await?
      .unwrap();
    assert!(!journal.is_archived);
    let journal_tags = journal.find_related(models::JournalTag).all(&db).await?;
    assert_eq!(journal_tags.len(), 2);

    let members_users: Vec<_> = users
      .iter()
      .map(|u| models::journal_user::ActiveModel {
        journal_id: Set(journal.id),
        user_id: Set(u.id),
        is_admin: Set(u.role != models::user::Role::User),
      })
      .collect();
    assert_eq!(members.len(), 2);
    let _ = models::JournalUser::insert_many(members_users).exec(&db).await?;

    let members_group = models::journal_group::ActiveModel {
      journal_id: Set(journal.id),
      group_id: Set(group.id),
      is_admin: Set(true),
    };
    let _ = models::JournalGroup::insert(members_group).exec(&db).await?;

    let members_users_admins = journal.find_linked(models::journal::JournalUserAdmin).all(&db).await?;
    assert_eq!(members_users_admins.len(), 1);
    let members_users_members = journal.find_linked(models::journal::JournalUserMember).all(&db).await?;
    assert_eq!(members_users_members.len(), 1);
    let members_groups_admins = journal.find_linked(models::journal::JournalGroupAdmin).all(&db).await?;
    assert_eq!(members_groups_admins.len(), 1);
    let members_groups_members = journal
      .find_linked(models::journal::JournalGroupMember)
      .all(&db)
      .await?;
    assert_eq!(members_groups_members.len(), 0);

    let account = models::account::ActiveModel {
      journal_id: Set(journal.id),
      name: Set("Journal 1".to_owned()),
      description: Set("Journal Description".to_owned()),
      typ: Set(models::account::Type::Asset),
      strategy: Set(models::account::Strategy::Average),
      unit: Set("TEST".to_owned()),
      ..Default::default()
    };
    let account = account.insert(&db).await?;

    let accounts = journal.find_related(models::Account).all(&db).await?;
    assert_eq!(accounts.len(), 1);

    let account_tags: Vec<_> = vec!["account tag 1", "account tag 2"]
      .iter()
      .map(|tag| models::account_tag::ActiveModel {
        account_id: Set(account.id),
        tag: Set(tag.to_string()),
      })
      .collect();
    let _ = models::AccountTag::insert_many(account_tags).exec(&db).await?;
    let account_tags = account.find_related(models::AccountTag).all(&db).await?;
    assert_eq!(account_tags.len(), 2);

    let record = models::record::ActiveModel {
      journal_id: Set(journal.id),
      name: Set("Journal Record 1".to_owned()),
      description: Set("Journal Record Description".to_owned()),
      typ: Set(models::record::Type::Record),
      date: Set(Utc::now()),
      ..Default::default()
    };
    let record = record.insert(&db).await?;

    let record_tags: Vec<_> = vec!["record tag 1", "record tag 2"]
      .iter()
      .map(|tag| models::record_tag::ActiveModel {
        record_id: Set(record.id),
        tag: Set(tag.to_string()),
      })
      .collect();
    let _ = models::RecordTag::insert_many(record_tags).exec(&db).await?;
    let record_tags = record.find_related(models::RecordTag).all(&db).await?;
    assert_eq!(record_tags.len(), 2);

    let record_item = models::record_item::ActiveModel {
      record_id: Set(record.id),
      account_id: Set(account.id),
      amount: Set(Some(dec!(1.2))),
      price: Set(Some(dec!(3.4))),
    };
    let record_item = record_item.insert(&db).await?;
    let record_items = record.find_related(models::RecordItem).all(&db).await?;
    assert_eq!(record_items.len(), 1);
    assert_eq!(record_item, record_items[0]);

    Migrator::down(&db, None).await?;
    Ok(())
  }
}
