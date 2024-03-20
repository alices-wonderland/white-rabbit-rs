mod query;

pub use query::*;

use crate::entity::{account, entry, ReadRoot};
use itertools::Itertools;
use rust_decimal::Decimal;
use sea_orm::ConnectionTrait;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::ops::{Add, Mul};
use uuid::Uuid;

const REPORT_SPLITERATOR: &str = ":::";

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Root {
  pub journal_id: Uuid,
  pub prefix: String,
  pub unit: String,
  pub values: HashMap<Uuid, Decimal>,
}

impl ReadRoot for Root {
  type Query = Query;
  type Sort = ();

  fn id(&self) -> String {
    [self.journal_id.to_string(), self.prefix.clone(), self.unit.clone()].join(REPORT_SPLITERATOR)
  }

  async fn find_all(
    db: &impl ConnectionTrait,
    query: Option<Query>,
    _limit: Option<u64>,
    _sort: Option<()>,
  ) -> crate::Result<Vec<Root>> {
    let mut journal_ids: HashSet<_> =
      query.iter().flat_map(|query| query.journal_id.iter().copied()).collect();
    if let Some(query) = &query {
      for id in &query.id {
        if let Some((journal_id, _)) = id.split_once(REPORT_SPLITERATOR) {
          if let Ok(journal_id) = Uuid::parse_str(journal_id) {
            journal_ids.insert(journal_id);
          }
        }
      }
    }

    let entries = entry::Root::find_all(
      db,
      Some(entry::Query {
        journal_id: journal_ids.clone(),
        start: query.as_ref().and_then(|query| query.start),
        end: query.as_ref().and_then(|query| query.end),
        typ: Some(entry::Type::Record),
        ..Default::default()
      }),
      None,
      None,
    )
    .await?;

    let accounts = account::Root::find_all(
      db,
      Some(account::Query { journal_id: journal_ids.clone(), ..Default::default() }),
      None,
      None,
    )
    .await?;

    let aggregated = Self::do_aggregate_by_account(&entries, &accounts);
    let aggregated = Self::do_aggregate(aggregated);

    Ok(match query {
      Some(query) if !query.id.is_empty() => aggregated
        .into_iter()
        .map(|root| (root.id(), root))
        .filter(|(id, _)| query.id.contains(id))
        .map(|(_, root)| root)
        .collect::<Vec<_>>(),
      _ => aggregated,
    })
  }
}

#[derive(Hash, Debug, Clone, PartialEq, Eq)]
struct Index {
  pub journal_id: Uuid,
  pub prefix: String,
  pub unit: String,
  pub account_id: Uuid,
}

impl Root {
  fn do_aggregate(aggregated: HashMap<Index, Decimal>) -> Vec<Root> {
    let mut results = Vec::default();
    for ((journal_id, prefix, unit), items) in
      aggregated.into_iter().into_group_map_by(|(Index { journal_id, prefix, unit, .. }, _)| {
        (*journal_id, prefix.to_string(), unit.to_string())
      })
    {
      results.push(Root {
        journal_id,
        prefix,
        unit,
        values: items
          .into_iter()
          .map(|(Index { account_id, .. }, value)| (account_id, value))
          .collect::<HashMap<_, _>>(),
      })
    }

    results
  }

  fn do_aggregate_by_account(
    entries: &[entry::Root],
    accounts: &[account::Root],
  ) -> HashMap<Index, Decimal> {
    let accounts = accounts.iter().map(|account| (account.id, account)).collect::<HashMap<_, _>>();

    let mut results = HashMap::<Index, Decimal>::default();

    for entry in entries {
      for entry_item in &entry.items {
        if let Some(account) = accounts.get(&entry_item.account) {
          let mut prefixes = HashSet::<String>::default();
          prefixes.insert(account.name.to_string());
          for (idx, _) in account.name.match_indices(account::NAME_SPLITERATOR) {
            prefixes.insert(account.name[0..idx].to_string());
          }

          for prefix in prefixes {
            let map_index = Index {
              journal_id: entry.journal_id,
              prefix,
              unit: account.unit.to_string(),
              account_id: account.id,
            };
            let item = results.get(&map_index).copied().unwrap_or_default();
            results.insert(map_index, item.add(entry_item.price.mul(entry_item.amount)));
          }
        }
      }
    }

    results
  }
}

#[cfg(test)]
mod tests {
  use crate::entity::hierarchy_report::Root;
  use crate::entity::{account, entry};
  use chrono::NaiveDate;
  use rust_decimal_macros::dec;
  use sea_orm::Iterable;
  use std::collections::HashSet;
  use uuid::Uuid;

  #[test]
  fn test_do_aggregate() -> anyhow::Result<()> {
    let journal_id = Uuid::new_v4();
    let mut accounts = Vec::default();
    for typ in account::Type::iter() {
      let id = Uuid::new_v4();
      accounts.push(account::Root {
        id,
        journal_id,
        name: format!("Account::{}", typ),
        description: "".to_string(),
        unit: "CNY".to_string(),
        typ,
        tags: HashSet::default(),
      });

      let id = Uuid::new_v4();
      accounts.push(account::Root {
        id,
        journal_id,
        name: format!("Account::{}::Child", typ),
        description: "".to_string(),
        unit: "USD".to_string(),
        typ,
        tags: HashSet::default(),
      });
    }

    let entries = vec![
      entry::Root {
        id: Uuid::new_v4(),
        journal_id,
        name: "Entry: 1".to_string(),
        description: "".to_string(),
        typ: entry::Type::Record,
        date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        tags: HashSet::default(),
        items: vec![
          entry::Item { account: accounts[0].id, amount: dec!(1.0), price: dec!(2.0) },
          entry::Item { account: accounts[1].id, amount: dec!(3.0), price: dec!(4.0) },
        ],
      },
      entry::Root {
        id: Uuid::new_v4(),
        journal_id,
        name: "Entry: 2".to_string(),
        description: "".to_string(),
        typ: entry::Type::Record,
        date: NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
        tags: HashSet::default(),
        items: vec![
          entry::Item { account: accounts[0].id, amount: dec!(2.0), price: dec!(2.0) },
          entry::Item { account: accounts[2].id, amount: dec!(1.0), price: dec!(3.0) },
        ],
      },
    ];

    let aggregated = Root::do_aggregate_by_account(&entries, &accounts);
    println!("Result: {:#?}", aggregated);

    println!("Aggregated: {:#?}", Root::do_aggregate(aggregated));

    Ok(())
  }
}
