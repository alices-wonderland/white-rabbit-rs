use backend_core::entity::{account, journal, FIELD_NAME};
use backend_core::Error;
use std::collections::HashSet;

#[tokio::test]
pub async fn test_swap_name() -> anyhow::Result<()> {
  let db = test_suite::init().await?;
  let journals = journal::Root::find_all(&db, None, Some(2)).await?;
  let accounts = account::Root::find_all(
    &db,
    Some(account::Query { journal_id: HashSet::from_iter([journals[0].id]), ..Default::default() }),
    None,
  )
  .await?;

  let updated = journal::Root::update(
    &db,
    vec![
      journal::CommandUpdate {
        id: journals[0].id,
        name: journals[1].name.clone(),
        description: None,
        unit: "".to_string(),
        tags: None,
      },
      journal::CommandUpdate {
        id: journals[1].id,
        name: journals[0].name.clone(),
        description: None,
        unit: "".to_string(),
        tags: None,
      },
    ],
  )
  .await?;
  let updated_accounts = account::Root::find_all(
    &db,
    Some(account::Query { journal_id: HashSet::from_iter([journals[0].id]), ..Default::default() }),
    None,
  )
  .await?;

  assert_eq!(2, updated.len());
  assert_eq!(accounts.len(), updated_accounts.len());

  for journal in &journals {
    if let Some(updated) = updated.iter().find(|model| model.id == journal.id) {
      assert_ne!(journal.name, updated.name);
      assert_eq!(journal.description, updated.description);
      assert_eq!(journal.unit, updated.unit);
      assert_eq!(journal.tags, updated.tags);
    }
  }

  Ok(())
}

#[tokio::test]
pub async fn test_swap_name2() -> anyhow::Result<()> {
  let db = test_suite::init().await?;
  let journals = journal::Root::find_all(&db, None, Some(3)).await?;

  if let Err(Error::ExistingEntity { values, .. }) = journal::Root::update(
    &db,
    vec![
      journal::CommandUpdate {
        id: journals[0].id,
        name: journals[1].name.clone(),
        description: None,
        unit: "".to_string(),
        tags: None,
      },
      journal::CommandUpdate {
        id: journals[1].id,
        name: journals[2].name.clone(),
        description: None,
        unit: "".to_string(),
        tags: None,
      },
    ],
  )
  .await
  {
    assert_eq!((FIELD_NAME.to_string(), journals[2].name.to_string()), values[0]);
  } else {
    unreachable!();
  }

  Ok(())
}

#[tokio::test]
pub async fn test_multi_update() -> anyhow::Result<()> {
  let db = test_suite::init().await?;
  let journal = journal::Root::find_one(&db, None).await?.unwrap();

  let updated = journal::Root::update(
    &db,
    vec![
      journal::CommandUpdate {
        id: journal.id,
        name: "New Name".to_string(),
        description: None,
        unit: "".to_string(),
        tags: None,
      },
      journal::CommandUpdate {
        id: journal.id,
        name: "".to_string(),
        description: Some("New Description".to_string()),
        unit: "".to_string(),
        tags: None,
      },
    ],
  )
  .await?;

  assert_eq!(1, updated.len());

  assert_eq!(journal.id, updated[0].id);
  assert_ne!(journal.name, updated[0].name);
  assert_ne!(journal.description, updated[0].description);
  assert_eq!(journal.unit, updated[0].unit);
  assert_eq!(journal.tags, updated[0].tags);

  Ok(())
}
