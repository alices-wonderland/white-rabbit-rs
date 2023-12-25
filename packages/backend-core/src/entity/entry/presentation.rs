use crate::entity;
use crate::entity::account;
use crate::entity::entry::{Item, Query, Root, Type};
use chrono::NaiveDate;
use itertools::Itertools;
use rust_decimal::Decimal;
use sea_orm::ConnectionTrait;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum StateItem {
  Valid(Decimal),
  Invalid(Decimal, Decimal),
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Presentation {
  Record(PresentationRecord),
  Check(PresentationCheck),
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PresentationRecord {
  pub id: Uuid,
  pub journal_id: Uuid,
  pub name: String,
  pub description: String,
  pub date: NaiveDate,
  pub tags: HashSet<String>,
  pub items: Vec<Item>,
  pub state: StateItem,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PresentationCheck {
  pub id: Uuid,
  pub journal_id: Uuid,
  pub name: String,
  pub description: String,
  pub date: NaiveDate,
  pub tags: HashSet<String>,
  pub items: Vec<Item>,
  pub state: HashMap<Uuid, StateItem>,
}

#[async_trait::async_trait]
impl entity::Presentation for Presentation {
  type R = Root;

  async fn from_roots(db: &impl ConnectionTrait, roots: Vec<Self::R>) -> crate::Result<Vec<Self>> {
    let journal_ids: HashSet<_> = roots.iter().map(|root| root.journal_id).collect();
    let account_ids: HashSet<_> =
      roots.iter().flat_map(|root| root.items.iter()).map(|item| item.account).collect();

    let related_entries = Root::find_all(
      db,
      Some(Query { journal_id: journal_ids, typ: Some(Type::Record), ..Default::default() }),
      None,
      None,
    )
    .await?
    .into_iter()
    .into_group_map_by(|root| root.journal_id);
    let related_accounts: HashMap<_, _> = account::Root::find_all(
      db,
      Some(account::Query { id: account_ids, ..Default::default() }),
      None,
      None,
    )
    .await?
    .into_iter()
    .map(|account| (account.id, account))
    .collect();

    let mut results = vec![];
    for root in roots {
      if root.typ == Type::Record {
        let mut left = Decimal::ZERO;
        let mut right = Decimal::ZERO;
        for item in &root.items {
          if let Some(account) = related_accounts.get(&item.account) {
            match &account.typ {
              account::Type::Asset | account::Type::Expense => {
                left += item.amount * item.price.unwrap_or(Decimal::ONE);
              }
              _ => {
                right += item.amount * item.price.unwrap_or(Decimal::ONE);
              }
            }
          }

          let state =
            if left == right { StateItem::Valid(left) } else { StateItem::Invalid(left, right) };
          results.push(Presentation::Record(PresentationRecord {
            id: root.id,
            journal_id: root.journal_id,
            name: root.name.clone(),
            description: root.description.clone(),
            date: root.date,
            tags: root.tags.clone(),
            items: root.items.clone(),
            state,
          }))
        }
      } else {
        let mut actuals = HashMap::<Uuid, Decimal>::new();
        if let Some(related_records) = related_entries.get(&root.journal_id) {
          for record in related_records {
            if record.date <= root.date {
              for item in &record.items {
                let value = actuals.get(&item.account).copied().unwrap_or_default();
                actuals
                  .insert(item.account, value + item.amount * item.price.unwrap_or(Decimal::ONE));
              }
            }
          }
        }
        let state = root
          .items
          .iter()
          .map(|item| {
            let expected = item.amount * item.price.unwrap_or(Decimal::ONE);
            let actual = actuals.get(&item.account).copied().unwrap_or_default();
            (
              item.account,
              if expected == actual {
                StateItem::Valid(expected)
              } else {
                StateItem::Invalid(expected, actual)
              },
            )
          })
          .collect();
        results.push(Presentation::Check(PresentationCheck {
          id: root.id,
          journal_id: root.journal_id,
          name: root.name.clone(),
          description: root.description.clone(),
          date: root.date,
          tags: root.tags.clone(),
          items: root.items.clone(),
          state,
        }))
      }
    }

    Ok(results)
  }
}
