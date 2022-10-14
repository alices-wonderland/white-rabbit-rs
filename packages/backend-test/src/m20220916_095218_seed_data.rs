use std::collections::HashMap;

use backend_shared::models::{
  account, account_tag, auth_id, group, group_user, journal, journal_group, journal_tag, journal_user, record,
  record_item, record_tag, user, Account, AccountTag, AuthId, Group, GroupUser, Journal, JournalGroup, JournalTag,
  JournalUser, Record, RecordItem, RecordTag, User,
};

use fake::{
  faker::{
    address::en::{CityName, CountryCode, CountryName},
    boolean::en::Boolean,
    chrono::en::Date,
    company::en::{Bs, BsNoun, Buzzword, CompanyName, Industry},
    lorem::en::Paragraph,
    name::en::Name,
  },
  Fake,
};
use rand::{seq::SliceRandom, thread_rng, Rng};

use sea_orm_migration::{
  prelude::*,
  sea_orm::{EntityTrait, Iterable, Set, TransactionTrait},
};
use uuid::Uuid;

fn create_users(size: usize, role: user::Role) -> (Vec<user::ActiveModel>, Vec<auth_id::ActiveModel>) {
  let users: Vec<_> = (0..size)
    .into_iter()
    .map(|_| user::ActiveModel {
      id: Set(Uuid::new_v4()),
      name: Set(Name().fake()),
      role: Set(role.clone()),
    })
    .collect();

  let auth_ids: Vec<_> = users
    .iter()
    .map(|u| auth_id::ActiveModel {
      user_id: u.id.clone(),
      provider: Set(CompanyName().fake()),
      value: Set(Uuid::new_v4().to_string()),
    })
    .collect();

  (users, auth_ids)
}

fn create_groups(size: usize, users: &mut [user::Model]) -> (Vec<group::ActiveModel>, Vec<group_user::ActiveModel>) {
  let groups: Vec<_> = (0..size)
    .into_iter()
    .map(|idx| group::ActiveModel {
      id: Set(Uuid::new_v4()),
      name: Set(format!("{}-{}", CountryName().fake::<String>(), idx)),
      description: Set(Paragraph(1..3).fake()),
    })
    .collect();

  let group_users: Vec<_> = groups
    .iter()
    .flat_map(|group| {
      users.shuffle(&mut thread_rng());
      let admins: Vec<_> = users[0..3]
        .iter()
        .map(|u| group_user::ActiveModel {
          group_id: group.id.clone(),
          user_id: Set(u.id),
          is_admin: Set(true),
        })
        .collect();
      let members: Vec<_> = users[3..8]
        .iter()
        .map(|u| group_user::ActiveModel {
          group_id: group.id.clone(),
          user_id: Set(u.id),
          is_admin: Set(false),
        })
        .collect();
      vec![admins, members].into_iter().flatten().collect::<Vec<_>>()
    })
    .collect();

  (groups, group_users)
}

fn create_journals(
  size: usize,
  users: &mut [user::Model],
  groups: &mut [group::Model],
) -> (
  Vec<journal::ActiveModel>,
  Vec<journal_tag::ActiveModel>,
  Vec<journal_user::ActiveModel>,
  Vec<journal_group::ActiveModel>,
) {
  let mut rng = thread_rng();

  let journals: Vec<_> = (0..size)
    .into_iter()
    .map(|idx| journal::ActiveModel {
      id: Set(Uuid::new_v4()),
      name: Set(format!("{}-{}", CompanyName().fake::<String>(), idx)),
      description: Set(Paragraph(1..3).fake()),
      unit: Set(CountryCode().fake()),
      is_archived: Set(Boolean(20).fake()),
    })
    .collect();

  let tags: Vec<_> = journals
    .iter()
    .flat_map(|journal| {
      (0..5).into_iter().map(|idx| journal_tag::ActiveModel {
        journal_id: journal.id.clone(),
        tag: Set(format!("{}-{}", BsNoun().fake::<String>(), idx)),
      })
    })
    .collect();

  let (users, groups): (
    Vec<Vec<journal_user::ActiveModel>>,
    Vec<Vec<journal_group::ActiveModel>>,
  ) = journals
    .iter()
    .map(|journal| {
      users.shuffle(&mut rng);
      groups.shuffle(&mut rng);

      let users = users[0..5]
        .iter()
        .enumerate()
        .map(|(idx, user)| journal_user::ActiveModel {
          journal_id: journal.id.clone(),
          user_id: Set(user.id),
          is_admin: Set(idx < 2),
        })
        .collect();

      let groups = groups[0..3]
        .iter()
        .enumerate()
        .map(|(idx, group)| journal_group::ActiveModel {
          journal_id: journal.id.clone(),
          group_id: Set(group.id),
          is_admin: Set(idx < 1),
        })
        .collect();

      (users, groups)
    })
    .unzip();
  let users: Vec<_> = users.into_iter().flatten().collect();
  let groups: Vec<_> = groups.into_iter().flatten().collect();

  (journals, tags, users, groups)
}

fn create_accounts(
  size_per_journals: usize,
  journals: &[journal::Model],
) -> (Vec<account::ActiveModel>, Vec<account_tag::ActiveModel>) {
  let accounts: Vec<_> = journals
    .iter()
    .flat_map(|journal| {
      (0..size_per_journals).into_iter().map(|idx| account::ActiveModel {
        id: Set(Uuid::new_v4()),
        journal_id: Set(journal.id),
        name: Set(format!("{}-{}-{}", journal.name, Industry().fake::<String>(), idx)),
        description: Set(Paragraph(1..3).fake()),
        typ: Set(account::Type::iter().nth(idx % 5).unwrap()),
        strategy: Set(account::Strategy::iter().nth(idx % 2).unwrap()),
        unit: Set(CountryCode().fake()),
        is_archived: Set(Boolean(10).fake()),
      })
    })
    .collect();

  let account_tags: Vec<_> = accounts
    .iter()
    .flat_map(|account| {
      (0..thread_rng().gen_range(10..15))
        .into_iter()
        .map(|idx| account_tag::ActiveModel {
          account_id: account.id.clone(),
          tag: Set(format!("{}-{}", Bs().fake::<String>(), idx)),
        })
    })
    .collect();

  (accounts, account_tags)
}

fn create_records(
  size_per_journal: usize,
  accounts: &[account::Model],
) -> (
  Vec<record::ActiveModel>,
  Vec<record_item::ActiveModel>,
  Vec<record_tag::ActiveModel>,
) {
  let mut rng = thread_rng();
  let accounts: HashMap<uuid::Uuid, Vec<&account::Model>> = accounts.iter().fold(HashMap::new(), |mut map, account| {
    map.entry(account.journal_id).or_insert_with(Vec::new).push(account);
    map
  });

  let mut records = Vec::new();
  let mut record_items = Vec::new();
  let mut record_tags = Vec::new();

  for (journal_id, mut accounts) in accounts {
    for _ in 0..size_per_journal {
      accounts.shuffle(&mut rng);

      let typ = if Boolean(20).fake() {
        record::Type::Check
      } else {
        record::Type::Record
      };
      let date = Date().fake();
      let record = record::ActiveModel {
        id: Set(Uuid::new_v4()),
        journal_id: Set(journal_id),
        name: Set(format!("{} - {}", date, CityName().fake::<String>())),
        description: Set(Paragraph(1..3).fake()),
        typ: Set(typ.clone()),
        date: Set(date),
      };
      let record_id = record.id.clone();
      records.push(record);

      let mut items: Vec<_> = accounts[0..accounts.len().min(5)]
        .iter()
        .map(|account| record_item::ActiveModel {
          record_id: record_id.clone(),
          account_id: Set(account.id),
          amount: Set(rng.gen_range(10..100).into()),
          price: Set(Some(rng.gen_range(10..100).into())),
        })
        .collect();
      record_items.append(&mut items);

      let mut tags: Vec<_> = (0..5)
        .into_iter()
        .map(|tag_idx| record_tag::ActiveModel {
          record_id: record_id.clone(),
          tag: Set(format!("{}-{}", Buzzword().fake::<String>(), tag_idx)),
        })
        .collect();
      record_tags.append(&mut tags);
    }
  }

  (records, record_items, record_tags)
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  // 1 1 2 3 5 8 13 21 34
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    dotenv::from_filename(".test.env").ok();
    let _ = env_logger::try_init();
    let db = manager.get_connection();
    let txn = db.begin().await?;

    let (users, auth_ids): (Vec<_>, Vec<_>) = vec![
      create_users(13, user::Role::User),
      create_users(8, user::Role::Admin),
      create_users(5, user::Role::Owner),
    ]
    .into_iter()
    .unzip();
    let users: Vec<_> = users.into_iter().flatten().collect();
    let auth_ids: Vec<_> = auth_ids.into_iter().flatten().collect();
    let _ = User::insert_many(users).exec(&txn).await?;
    let _ = AuthId::insert_many(auth_ids).exec(&txn).await?;
    let mut users: Vec<user::Model> = User::find().all(&txn).await?;

    let (groups, group_users) = create_groups(13, &mut users);
    let _ = Group::insert_many(groups).exec(&txn).await?;
    let _ = GroupUser::insert_many(group_users).exec(&txn).await?;
    let mut groups: Vec<group::Model> = Group::find().all(&txn).await?;

    let (journals, journal_tags, journal_users, journal_groups) = create_journals(13, &mut users, &mut groups);
    let _ = Journal::insert_many(journals).exec(&txn).await?;
    let _ = JournalTag::insert_many(journal_tags).exec(&txn).await?;
    let _ = JournalUser::insert_many(journal_users).exec(&txn).await?;
    let _ = JournalGroup::insert_many(journal_groups).exec(&txn).await?;
    let journals: Vec<journal::Model> = Journal::find().all(&txn).await?;

    let (accounts, account_tags) = create_accounts(8, &journals);
    let _ = Account::insert_many(accounts).exec(&txn).await?;
    let _ = AccountTag::insert_many(account_tags).exec(&txn).await?;
    let accounts: Vec<account::Model> = Account::find().all(&txn).await?;

    let (records, record_items, record_tags) = create_records(8, &accounts);
    let _ = Record::insert_many(records).exec(&txn).await?;
    let _ = RecordTag::insert_many(record_tags).exec(&txn).await?;
    let _ = RecordItem::insert_many(record_items).exec(&txn).await?;

    txn.commit().await?;
    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    let db = manager.get_connection();
    let txn = db.begin().await?;

    let _ = Record::delete_many().exec(&txn).await?;
    let _ = Account::delete_many().exec(&txn).await?;
    let _ = Journal::delete_many().exec(&txn).await?;
    let _ = Group::delete_many().exec(&txn).await?;
    let _ = User::delete_many().exec(&txn).await?;

    txn.commit().await?;
    Ok(())
  }
}
