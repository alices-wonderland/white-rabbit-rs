use backend_core::entity::{journal, FIELD_NAME};
use backend_core::Error;
use std::collections::HashSet;

#[tokio::test]
pub async fn test_swap_name() -> anyhow::Result<()> {
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

  let updated_models = journal::Root::update(
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

  for updated in &updated_models {
    for journal in &journals {
      if updated.id == journal.id {
        assert_ne!(journal.name, updated.name);
      } else {
        assert_eq!(journal.name, updated.name);
      }
    }
  }

  Ok(())
}

#[tokio::test]
pub async fn test_swap_name2() -> anyhow::Result<()> {
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
      journal::CommandCreate {
        name: "Test Journal 3".to_string(),
        description: "Desc Journal 3".to_string(),
        unit: "CNY".to_string(),
        tags: HashSet::from_iter(["Tag 1".to_string(), "Tag 2".to_string()]),
      },
    ],
  )
  .await?;

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
