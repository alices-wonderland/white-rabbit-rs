use backend_core::entity::{account, journal, FIELD_JOURNAL, FIELD_NAME};
use backend_core::Error;

use std::collections::HashSet;

#[tokio::test]
pub async fn test_create() -> anyhow::Result<()> {
  let db = backend_core::init(".test.env").await?;

  let journals = journal::Root::create(
    &db,
    vec![
      journal::CommandCreate {
        name: "Test Journal 1".to_string(),
        description: "Desc Journal 1".to_string(),
        unit: "CNY".to_string(),
        tags: HashSet::from_iter(["Tag 1".to_string(), "Tag 2".to_string()]),
      },
      journal::CommandCreate {
        name: "Test Journal 2".to_string(),
        description: "Desc Journal 2".to_string(),
        unit: "USD".to_string(),
        tags: HashSet::from_iter(["Tag 1".to_string(), "Tag 2".to_string()]),
      },
    ],
  )
  .await?;

  let commands = vec![
    account::CommandCreate {
      journal_id: journals[0].id,
      name: format!("{} - Account 1", journals[0].name),
      description: "Desc Account 1-1".to_string(),
      unit: "CNY".to_string(),
      typ: account::Type::Income,
      tags: HashSet::from_iter(["Tag 1".to_string(), "Tag 2".to_string()]),
    },
    account::CommandCreate {
      journal_id: journals[0].id,
      name: format!("{} - Account 2", journals[0].name),
      description: "Desc Journal 1-2".to_string(),
      unit: "USD".to_string(),
      typ: account::Type::Equity,
      tags: HashSet::from_iter(["Tag 2".to_string(), "Tag 4".to_string()]),
    },
    account::CommandCreate {
      journal_id: journals[1].id,
      name: format!("{} - Account 1", journals[0].name),
      description: "Desc Journal 2-1".to_string(),
      unit: "USD".to_string(),
      typ: account::Type::Equity,
      tags: HashSet::from_iter(["Tag 1".to_string(), "Tag 4".to_string()]),
    },
    account::CommandCreate {
      journal_id: journals[1].id,
      name: format!("{} - Account 2", journals[0].name),
      description: "Desc Journal 2-2".to_string(),
      unit: "CNY".to_string(),
      typ: account::Type::Liability,
      tags: HashSet::from_iter(["Tag 2".to_string(), "Tag 3".to_string()]),
    },
  ];
  let accounts = account::Root::create(&db, commands.clone()).await?;

  assert_eq!(commands.len(), accounts.len());

  Ok(())
}

#[tokio::test]
pub async fn test_swap_name() -> anyhow::Result<()> {
  let db = backend_core::init(".test.env").await?;

  let journals = journal::Root::create(
    &db,
    vec![journal::CommandCreate {
      name: "Test Journal 1".to_string(),
      description: "Desc Journal 1".to_string(),
      unit: "CNY".to_string(),
      tags: HashSet::from_iter(["Tag 1".to_string(), "Tag 2".to_string()]),
    }],
  )
  .await?;

  let accounts = account::Root::create(
    &db,
    vec![
      account::CommandCreate {
        journal_id: journals[0].id,
        name: format!("{} - Account 1", journals[0].name),
        description: "Desc Account 1-1".to_string(),
        unit: "CNY".to_string(),
        typ: account::Type::Income,
        tags: HashSet::from_iter(["Tag 1".to_string(), "Tag 2".to_string()]),
      },
      account::CommandCreate {
        journal_id: journals[0].id,
        name: format!("{} - Account 2", journals[0].name),
        description: "Desc Journal 1-2".to_string(),
        unit: "USD".to_string(),
        typ: account::Type::Equity,
        tags: HashSet::from_iter(["Tag 2".to_string(), "Tag 4".to_string()]),
      },
    ],
  )
  .await?;

  let updated = account::Root::update(
    &db,
    vec![
      account::CommandUpdate {
        id: accounts[0].id,
        name: accounts[1].name.to_string(),
        description: None,
        unit: "".to_string(),
        typ: None,
        tags: None,
      },
      account::CommandUpdate {
        id: accounts[1].id,
        name: accounts[0].name.to_string(),
        description: None,
        unit: "".to_string(),
        typ: None,
        tags: None,
      },
    ],
  )
  .await?;

  for account in &accounts {
    if let Some(updated) = updated.iter().find(|model| model.id == account.id) {
      assert_ne!(account.name, updated.name);
      assert_eq!(account.description, updated.description);
      assert_eq!(account.unit, updated.unit);
      assert_eq!(account.typ, updated.typ);
      assert_eq!(account.tags, updated.tags);
    }
  }

  Ok(())
}

#[tokio::test]
pub async fn test_swap_name2() -> anyhow::Result<()> {
  let db = backend_core::init(".test.env").await?;

  let journals = journal::Root::create(
    &db,
    vec![journal::CommandCreate {
      name: "Test Journal 1".to_string(),
      description: "Desc Journal 1".to_string(),
      unit: "CNY".to_string(),
      tags: HashSet::from_iter(["Tag 1".to_string(), "Tag 2".to_string()]),
    }],
  )
  .await?;

  let accounts = account::Root::create(
    &db,
    vec![
      account::CommandCreate {
        journal_id: journals[0].id,
        name: format!("{} - Account 1", journals[0].name),
        description: "Desc Account 1-1".to_string(),
        unit: "CNY".to_string(),
        typ: account::Type::Income,
        tags: HashSet::from_iter(["Tag 1".to_string(), "Tag 2".to_string()]),
      },
      account::CommandCreate {
        journal_id: journals[0].id,
        name: format!("{} - Account 2", journals[0].name),
        description: "Desc Journal 1-2".to_string(),
        unit: "USD".to_string(),
        typ: account::Type::Equity,
        tags: HashSet::from_iter(["Tag 2".to_string(), "Tag 4".to_string()]),
      },
    ],
  )
  .await?;

  if let Err(Error::ExistingEntity { values, .. }) = account::Root::update(
    &db,
    vec![account::CommandUpdate {
      id: accounts[0].id,
      name: accounts[1].name.to_string(),
      description: None,
      unit: "".to_string(),
      typ: None,
      tags: None,
    }],
  )
  .await
  {
    assert_eq!(
      vec![
        (FIELD_JOURNAL.to_string(), journals[0].id.to_string()),
        (FIELD_NAME.to_string(), accounts[1].name.to_string()),
      ],
      values
    );
  } else {
    unreachable!();
  }

  Ok(())
}
