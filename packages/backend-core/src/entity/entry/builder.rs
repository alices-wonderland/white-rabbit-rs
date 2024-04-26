use crate::entity::entry::{Item, Root, Type, FIELD_AMOUNT, FIELD_PRICE, TYPE};
use crate::entity::{
  account, normalize_description, normalize_name, normalize_tags, FIELD_ID, FIELD_JOURNAL,
  FIELD_TYPE,
};
use crate::error::ErrorNotFound;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Debug, Default)]
pub struct Builder {
  id: Option<Uuid>,
  journal_id: Option<Uuid>,
  name: String,
  description: String,
  typ: Option<Type>,
  date: NaiveDate,
  tags: HashSet<String>,
  items: Vec<Item>,
}

impl From<Root> for Builder {
  fn from(value: Root) -> Self {
    Builder {
      id: Some(value.id),
      journal_id: Some(value.journal_id),
      name: value.name,
      description: value.description,
      typ: Some(value.typ),
      date: value.date,
      tags: value.tags,
      items: value.items,
    }
  }
}

impl Builder {
  pub fn build(self, accounts: &HashMap<Uuid, account::Root>) -> crate::Result<Root> {
    let name = normalize_name(TYPE, self.name)?;
    let description = normalize_description(TYPE, self.description)?;
    let tags = normalize_tags(TYPE, self.tags)?;
    let mut filtered_items = HashMap::new();
    let journal_id = self.journal_id.ok_or_else(|| crate::Error::RequiredField {
      typ: TYPE.to_string(),
      field: FIELD_JOURNAL.to_string(),
    })?;

    for Item { account, amount, price } in self.items {
      if let Some(account) = accounts.get(&account) {
        if account.journal_id != journal_id {
          return Err(crate::Error::NotFound(ErrorNotFound {
            entity: account::TYPE.to_string(),
            values: vec![
              (FIELD_JOURNAL.to_string(), journal_id.to_string()),
              (FIELD_ID.to_string(), account.id.to_string()),
            ],
          }));
        } else if amount.is_sign_negative() {
          return Err(crate::Error::OutOfRange {
            typ: TYPE.to_string(),
            field: FIELD_AMOUNT.to_string(),
            start: Some(0.to_string()),
            end: None,
          });
        } else if price <= Decimal::ZERO {
          {
            return Err(crate::Error::OutOfRange {
              typ: TYPE.to_string(),
              field: FIELD_PRICE.to_string(),
              start: Some(0.to_string()),
              end: None,
            });
          }
        }

        filtered_items.insert(account.id, (amount, price));
      } else {
        return Err(crate::Error::NotFound(ErrorNotFound {
          entity: account::TYPE.to_string(),
          values: vec![(FIELD_ID.to_string(), account.to_string())],
        }));
      }
    }

    Ok(Root {
      id: self.id.unwrap_or_else(Uuid::new_v4),
      journal_id,
      name,
      description,
      typ: self.typ.ok_or_else(|| crate::Error::RequiredField {
        typ: TYPE.to_string(),
        field: FIELD_TYPE.to_string(),
      })?,
      date: self.date,
      tags,
      items: filtered_items
        .into_iter()
        .map(|(account, (amount, price))| Item { account, amount, price })
        .collect(),
    })
  }

  pub fn id(self, id: Uuid) -> Builder {
    Builder { id: Some(id), ..self }
  }

  pub fn journal_id(self, journal_id: Uuid) -> Builder {
    Builder { journal_id: Some(journal_id), ..self }
  }

  pub fn name(self, name: impl ToString) -> Builder {
    Builder { name: name.to_string(), ..self }
  }

  pub fn description(self, description: impl ToString) -> Builder {
    Builder { description: description.to_string(), ..self }
  }

  pub fn typ(self, typ: Type) -> Builder {
    Builder { typ: Some(typ), ..self }
  }

  pub fn date(self, date: NaiveDate) -> Builder {
    Builder { date, ..self }
  }
  pub fn tags(self, tags: impl IntoIterator<Item = impl ToString>) -> Builder {
    Builder { tags: tags.into_iter().map(|s| s.to_string()).collect(), ..self }
  }

  pub fn items(self, items: Vec<Item>) -> Builder {
    Builder { items, ..self }
  }
}
