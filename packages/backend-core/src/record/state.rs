use crate::account::Account;
use crate::journal::Journal;
use crate::record::{Query, Record, Type};
use crate::{account, utils, AggregateRoot, Error, FindAllArgs, Repository, Result};
use itertools::Itertools;
use rust_decimal::Decimal;
use sea_orm::{ConnectionTrait, StreamTrait};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum State {
  Record(Item),
  Check(HashMap<Uuid, Item>),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum Item {
  Valid(Decimal),
  Invalid(Decimal, Decimal),
}

impl Default for Item {
  fn default() -> Self {
    Self::Valid(Decimal::ZERO)
  }
}

impl State {
  pub fn default(typ: Type) -> State {
    match typ {
      Type::Record => State::Record(Item::default()),
      Type::Check => State::Check(HashMap::default()),
    }
  }

  fn of_record(
    journal: &Journal,
    accounts: &HashMap<Uuid, Account>,
    record: &Record,
  ) -> Result<Self> {
    let mut left = Decimal::ZERO;
    let mut right = Decimal::ZERO;
    for item in &record.items {
      if let Some(account) = accounts.get(&item.account) {
        let price = if journal.unit == account.unit {
          Decimal::ONE
        } else if let Some(price) = item.price {
          price
        } else {
          return Err(Error::RecordItemMustContainPrice { id: journal.id, account: account.id })?;
        };
        match account.typ {
          account::Type::Asset | account::Type::Expense => left += item.amount * price,
          _ => right += item.amount * price,
        };
      } else {
        return Err(Error::not_found::<Account>(vec![("id", item.account)]))?;
      }
    }
    Ok(State::Record(if left != right { Item::Invalid(left, right) } else { Item::Valid(left) }))
  }

  fn of_check(record: &Record, related: &[&Record]) -> Result<Self> {
    let sums_by_account = related
      .iter()
      .flat_map(|r| r.items.iter())
      .into_grouping_map_by(|item| item.account)
      .fold(Decimal::ZERO, |acc, _, val| acc + val.amount);
    Ok(State::Check(
      record
        .items
        .iter()
        .map(|item| {
          let actual = sums_by_account.get(&item.account).cloned().unwrap_or(Decimal::ZERO);
          let expected = item.amount;
          if actual == expected {
            (item.account, Item::Valid(expected))
          } else {
            (item.account, Item::Invalid(expected, actual))
          }
        })
        .collect(),
    ))
  }

  pub async fn of(
    db: &(impl ConnectionTrait + StreamTrait),
    records: &[Record],
  ) -> Result<HashMap<Uuid, Self>> {
    let journals = Repository::<Journal>::find_by_ids(
      db,
      records.iter().map(|record| record.journal).collect::<HashSet<_>>(),
    )
    .await?;
    let journals = utils::into_map(journals);

    let accounts = Repository::<Account>::find_by_ids(
      db,
      records
        .iter()
        .flat_map(|record| record.items.iter())
        .map(|item| item.account)
        .collect::<HashSet<_>>(),
    )
    .await?;
    let accounts = utils::into_map(accounts);

    let records_by_journal = Repository::<Record>::do_find_all(
      db,
      FindAllArgs {
        query: Query {
          journal: journals.keys().cloned().collect(),
          typ: Some(Type::Record),
          ..Default::default()
        },
        ..Default::default()
      },
      None,
    )
    .await?
    .into_iter()
    .into_group_map_by(|record| record.journal);

    let mut result = HashMap::default();

    for record in records {
      if let Some(journal) = journals.get(&record.journal) {
        if record.typ == Type::Record {
          result.insert(record.id(), Self::of_record(journal, &accounts, record)?);
        } else if record.typ == Type::Check {
          let related = records_by_journal
            .get(&journal.id)
            .into_iter()
            .flat_map(|records| records.iter())
            .filter(|related| related.date <= record.date)
            .collect::<Vec<_>>();

          result.insert(record.id(), Self::of_check(record, &related)?);
        }
      };
    }

    Ok(result)
  }
}
