use crate::entity::entry;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "commandType")]
pub enum Command {
  #[serde(rename = "entries:create")]
  Create(CommandCreate),
  #[serde(rename = "entries:update")]
  Update(CommandUpdate),
  #[serde(rename = "entries:delete")]
  Delete(CommandDelete),
  #[serde(rename = "entries:batch")]
  Batch(CommandBatch),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CommandCreate {
  pub journal_id: Uuid,
  pub name: String,
  #[serde(default)]
  pub description: String,
  #[serde(rename = "type")]
  pub typ: entry::Type,
  pub date: NaiveDate,
  #[serde(default)]
  pub tags: HashSet<String>,
  #[serde(default)]
  pub items: Vec<entry::Item>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CommandUpdate {
  pub id: Uuid,
  #[serde(default)]
  pub name: String,
  #[serde(default)]
  pub description: Option<String>,
  #[serde(default)]
  #[serde(rename = "type")]
  pub typ: Option<entry::Type>,
  #[serde(default)]
  pub date: Option<NaiveDate>,
  #[serde(default)]
  pub tags: Option<HashSet<String>>,
  #[serde(default)]
  pub items: Vec<entry::Item>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommandDelete {
  #[serde(default)]
  pub id: HashSet<Uuid>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommandBatch {
  #[serde(default)]
  pub create: Vec<CommandCreate>,
  #[serde(default)]
  pub update: Vec<CommandUpdate>,
  #[serde(default)]
  pub delete: HashSet<Uuid>,
}

#[cfg(test)]
mod tests {
  use crate::entity::entry;
  use chrono::NaiveDate;
  use rust_decimal_macros::dec;
  use serde_json::json;
  use std::collections::HashSet;
  use uuid::uuid;

  #[test]
  fn test_serde() {
    let commands = vec![
      entry::Command::Create(entry::CommandCreate {
        journal_id: uuid!("7aaec70c-adbc-47d1-8b74-a3e21f387d21"),
        name: "New Name".to_string(),
        description: "".to_string(),
        typ: entry::Type::Check,
        date: NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
        tags: HashSet::from_iter(["tag1".to_string()]),
        items: vec![entry::Item {
          account: uuid!("7aaec70c-adbc-47d1-8b74-a3e21f387d22"),
          amount: dec!(1.2),
          price: dec!(1.0),
        }],
      }),
      entry::Command::Update(entry::CommandUpdate {
        id: uuid!("7aaec70c-adbc-47d1-8b74-a3e21f387d21"),
        name: "New Name".to_string(),
        description: None,
        typ: Some(entry::Type::Record),
        date: Some(NaiveDate::from_ymd_opt(2023, 2, 1).unwrap()),
        tags: None,
        items: Vec::default(),
      }),
      entry::Command::Delete(entry::CommandDelete {
        id: HashSet::from_iter([uuid!("7aaec70c-adbc-47d1-8b74-a3e21f387d21")]),
      }),
      entry::Command::Batch(entry::CommandBatch {
        create: vec![
          entry::CommandCreate {
            journal_id: uuid!("7aaec70c-adbc-47d1-8b74-a3e21f387d21"),
            name: "New Name 1".to_string(),
            description: "".to_string(),
            typ: entry::Type::Check,
            date: NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            tags: HashSet::from_iter(["tag1".to_string()]),
            items: vec![entry::Item {
              account: uuid!("7aaec70c-adbc-47d1-8b74-a3e21f387d22"),
              amount: dec!(1.2),
              price: dec!(1.0),
            }],
          },
          entry::CommandCreate {
            journal_id: uuid!("7aaec70c-adbc-47d1-8b74-a3e21f387d22"),
            name: "New Name 2".to_string(),
            description: "Desc 2".to_string(),
            typ: entry::Type::Record,
            date: NaiveDate::from_ymd_opt(2023, 2, 1).unwrap(),
            tags: HashSet::from_iter(["tag1".to_string()]),
            items: vec![entry::Item {
              account: uuid!("7aaec70c-adbc-47d1-8b74-a3e21f387d22"),
              amount: dec!(1.1),
              price: dec!(2.2),
            }],
          },
        ],
        update: vec![
          entry::CommandUpdate {
            id: uuid!("7aaec70c-adbc-47d1-8b74-a3e21f387d24"),
            name: "New Name".to_string(),
            description: None,
            typ: Some(entry::Type::Record),
            date: Some(NaiveDate::from_ymd_opt(2023, 2, 1).unwrap()),
            tags: None,
            items: Vec::default(),
          },
          entry::CommandUpdate {
            id: uuid!("7aaec70c-adbc-47d1-8b74-a3e21f387d25"),
            name: "New Name 2".to_string(),
            description: None,
            typ: Some(entry::Type::Record),
            date: Some(NaiveDate::from_ymd_opt(2023, 2, 1).unwrap()),
            tags: None,
            items: Vec::default(),
          },
        ],
        delete: HashSet::from_iter([uuid!("7aaec70c-adbc-47d1-8b74-a3e21f387d21")]),
      }),
    ];

    assert_eq!(
      serde_json::to_value(commands).unwrap(),
      json!([
        {
          "commandType": "entries:create",
          "journalId": "7aaec70c-adbc-47d1-8b74-a3e21f387d21",
          "name": "New Name",
          "description": "",
          "type": "Check",
          "date": "2023-01-01",
          "tags": ["tag1"],
          "items": [
            { "account":"7aaec70c-adbc-47d1-8b74-a3e21f387d22", "amount": "1.2", "price": "1.0" }
          ]
        },
        {
          "commandType": "entries:update",
          "id": "7aaec70c-adbc-47d1-8b74-a3e21f387d21",
          "name": "New Name",
          "description": null,
          "type": "Record",
          "date": "2023-02-01",
          "tags": null,
          "items": []
        },
        {
          "commandType": "entries:delete",
          "id": ["7aaec70c-adbc-47d1-8b74-a3e21f387d21"],
        },
        {
          "commandType": "entries:batch",
          "create": [
            {
              "journalId": "7aaec70c-adbc-47d1-8b74-a3e21f387d21",
              "name": "New Name 1",
              "description": "",
              "type": "Check",
              "date": "2023-01-01",
              "tags": ["tag1"],
              "items": [
                { "account":"7aaec70c-adbc-47d1-8b74-a3e21f387d22", "amount": "1.2", "price": "1.0" }
              ]
            },
            {
              "journalId": "7aaec70c-adbc-47d1-8b74-a3e21f387d22",
              "name": "New Name 2",
              "description": "Desc 2",
              "type": "Record",
              "date": "2023-02-01",
              "tags": ["tag1"],
              "items": [
                { "account":"7aaec70c-adbc-47d1-8b74-a3e21f387d22", "amount": "1.1", "price": "2.2" }
              ]
            }
          ],
          "update": [
            {
              "id": "7aaec70c-adbc-47d1-8b74-a3e21f387d24",
              "name": "New Name",
              "description": null,
              "type": "Record",
              "date": "2023-02-01",
              "tags": null,
              "items": []
            },
            {
              "id": "7aaec70c-adbc-47d1-8b74-a3e21f387d25",
              "name": "New Name 2",
              "description": null,
              "type": "Record",
              "date": "2023-02-01",
              "tags": null,
              "items": []
            },
          ],
          "delete": [ "7aaec70c-adbc-47d1-8b74-a3e21f387d21" ]
        }
      ])
    );
  }
}
