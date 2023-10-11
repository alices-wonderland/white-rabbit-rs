mod command;
mod database;
mod query;

pub use command::*;
pub use database::*;
pub use query::*;

use crate::entity::{
  account, entry_item, entry_tag, journal, normalize_description, normalize_name, normalize_tags,
  FIELD_ID, FIELD_JOURNAL, FIELD_NAME,
};
use chrono::NaiveDate;
use itertools::Itertools;
use rust_decimal::Decimal;
use sea_orm::sea_query::{BinOper, Expr, OnConflict};
use sea_orm::{ColumnTrait, DbConn, EntityTrait, IntoActiveModel, QueryFilter, QuerySelect};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

pub const TYPE: &str = "Entry";
pub const FIELD_AMOUNT: &str = "amount";
pub const FIELD_PRICE: &str = "price";

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Root {
  pub id: Uuid,
  pub journal_id: Uuid,
  pub name: String,
  pub description: String,
  pub typ: Type,
  pub date: NaiveDate,
  pub tags: HashSet<String>,
  pub items: HashMap<Uuid, (Decimal, Option<Decimal>)>,
}
impl super::Root for Root {
  fn id(&self) -> Uuid {
    self.id
  }
}

impl Root {
  pub fn new(
    id: Option<Uuid>,
    journal: &journal::Root,
    name: impl ToString,
    description: impl ToString,
    typ: Type,
    date: NaiveDate,
    tags: impl IntoIterator<Item = impl ToString>,
    items: impl IntoIterator<Item = (Uuid, (Decimal, Option<Decimal>))>,
    accounts: &HashMap<Uuid, account::Root>,
  ) -> crate::Result<Root> {
    let name = normalize_name(TYPE, name)?;
    let description = normalize_description(TYPE, description)?;
    let tags = normalize_tags(TYPE, tags)?;
    let mut filtered_items = HashMap::new();

    for (account_id, (amount, price)) in items {
      if let Some(account) = accounts.get(&account_id) {
        if account.journal_id != journal.id {
          return Err(crate::Error::NotFound {
            typ: account::TYPE.to_string(),
            values: vec![
              (FIELD_JOURNAL.to_string(), journal.id.to_string()),
              (FIELD_ID.to_string(), account_id.to_string()),
            ],
          });
        } else if amount.is_sign_negative() {
          return Err(crate::Error::OutOfRange {
            typ: TYPE.to_string(),
            field: FIELD_AMOUNT.to_string(),
            start: Some(0.to_string()),
            end: None,
          });
        } else if let Some(price) = price {
          if price.is_sign_negative() {
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
        return Err(crate::Error::NotFound {
          typ: account::TYPE.to_string(),
          values: vec![(FIELD_ID.to_string(), account_id.to_string())],
        });
      }
    }

    Ok(Root {
      id: id.unwrap_or_else(Uuid::new_v4),
      journal_id: journal.id,
      name,
      description,
      typ,
      date,
      tags,
      items: filtered_items,
    })
  }

  pub async fn from_model(
    db: &DbConn,
    models: impl IntoIterator<Item = Model>,
  ) -> crate::Result<Vec<Root>> {
    let mut roots = Vec::new();
    let mut ids = HashSet::<Uuid>::new();

    for model in models {
      roots.push(Root {
        id: model.id,
        journal_id: model.journal_id,
        name: model.name,
        description: model.description,
        typ: model.typ,
        date: model.date,
        tags: HashSet::default(),
        items: HashMap::default(),
      });
      ids.insert(model.id);
    }

    let tags = entry_tag::Entity::find()
      .filter(entry_tag::Column::EntryId.is_in(ids.clone()))
      .all(db)
      .await?
      .into_iter()
      .into_group_map_by(|tag| tag.entry_id)
      .into_iter()
      .map(|(k, v)| (k, v.into_iter().map(|m| m.tag).collect::<HashSet<_>>()))
      .collect::<HashMap<_, _>>();

    let items = entry_item::Entity::find()
      .filter(entry_item::Column::EntryId.is_in(ids))
      .all(db)
      .await?
      .into_iter()
      .into_group_map_by(|item| item.entry_id)
      .into_iter()
      .map(|(k, v)| {
        let item_map = v
          .into_iter()
          .map(|item| (item.account_id, (item.amount, item.price)))
          .collect::<HashMap<_, _>>();
        (k, item_map)
      })
      .collect::<HashMap<_, _>>();

    Ok(
      roots
        .into_iter()
        .map(|root| Self {
          tags: tags.get(&root.id).cloned().unwrap_or_default(),
          items: items.get(&root.id).cloned().unwrap_or_default(),
          ..root
        })
        .collect(),
    )
  }

  pub async fn save(
    db: &DbConn,
    roots: impl IntoIterator<Item = Root>,
  ) -> crate::Result<Vec<Root>> {
    let roots: Vec<Root> = roots.into_iter().collect();
    if roots.is_empty() {
      return Ok(roots);
    }

    let mut model_ids = HashSet::new();
    let mut models: Vec<ActiveModel> = vec![];
    let mut tags: Vec<entry_tag::ActiveModel> = vec![];
    let mut items: Vec<entry_item::ActiveModel> = vec![];

    for ref root in roots {
      model_ids.insert(root.id);
      models.push(
        Model {
          id: root.id,
          journal_id: root.journal_id,
          name: root.name.to_string(),
          description: root.description.to_string(),
          typ: root.typ,
          date: root.date,
        }
        .into_active_model(),
      );
      for tag in &root.tags {
        tags.push(entry_tag::Model { entry_id: root.id, tag: tag.to_string() }.into_active_model());
      }
      for (account_id, (amount, price)) in &root.items {
        items.push(
          entry_item::Model {
            entry_id: root.id,
            account_id: *account_id,
            amount: *amount,
            price: *price,
          }
          .into_active_model(),
        )
      }
    }

    entry_tag::Entity::delete_many()
      .filter(entry_tag::Column::EntryId.is_in(model_ids.clone()))
      .exec(db)
      .await?;

    entry_item::Entity::delete_many()
      .filter(entry_item::Column::EntryId.is_in(model_ids.clone()))
      .exec(db)
      .await?;

    let mut on_conflict = OnConflict::column(Column::Id);
    on_conflict.update_columns([
      Column::JournalId,
      Column::Name,
      Column::Description,
      Column::Typ,
      Column::Date,
    ]);

    // Update unique column name to temp value
    Entity::update_many()
      .col_expr(
        Column::Name,
        Expr::col(Column::Name).binary(BinOper::Custom("||"), Expr::current_timestamp()),
      )
      .filter(Column::Id.is_in(model_ids.clone()))
      .exec(db)
      .await?;

    Entity::insert_many(models).on_conflict(on_conflict).exec(db).await?;

    if !tags.is_empty() {
      entry_tag::Entity::insert_many(tags).exec(db).await?;
    }

    if !items.is_empty() {
      entry_item::Entity::insert_many(items).exec(db).await?;
    }

    Self::find_all(db, Some(Query { id: model_ids, ..Default::default() }), None).await
  }

  pub async fn delete(db: &DbConn, ids: impl IntoIterator<Item = Uuid>) -> crate::Result<()> {
    Entity::delete_many().filter(Column::Id.is_in(ids)).exec(db).await?;
    Ok(())
  }

  pub async fn find_one(db: &DbConn, query: Option<Query>) -> crate::Result<Option<Root>> {
    Ok(Self::find_all(db, query, Some(1)).await?.into_iter().next())
  }

  pub async fn find_all(
    db: &DbConn,
    query: Option<Query>,
    limit: Option<u64>,
  ) -> crate::Result<Vec<Root>> {
    let select =
      if let Some(query) = query { Entity::find().filter(query) } else { Entity::find() };
    let models = select.limit(limit).all(db).await?;
    Self::from_model(db, models).await
  }

  pub async fn create(db: &DbConn, commands: Vec<CommandCreate>) -> crate::Result<Vec<Root>> {
    if commands.is_empty() {
      return Ok(vec![]);
    }

    let journals = journal::Root::find_all(
      db,
      Some(journal::Query {
        id: commands.iter().map(|c| c.journal_id).collect(),
        ..Default::default()
      }),
      None,
    )
    .await?
    .into_iter()
    .map(|journal| (journal.id, journal))
    .collect::<HashMap<_, _>>();

    let accounts = account::Root::find_all(
      db,
      Some(account::Query {
        id: commands.iter().flat_map(|c| c.items.keys()).copied().collect(),
        ..Default::default()
      }),
      None,
    )
    .await?
    .into_iter()
    .map(|account| (account.id, account))
    .collect::<HashMap<_, _>>();

    let names_by_journal = commands
      .iter()
      .map(|command| (command.journal_id, command.name.clone()))
      .into_group_map_by(|p| p.0)
      .into_iter()
      .map(|(k, vec)| (k, vec.into_iter().map(|(_, v)| v).unique().collect::<Vec<_>>()))
      .collect::<HashMap<_, _>>();

    for (journal_id, names) in names_by_journal {
      if !journals.contains_key(&journal_id) {
        return Err(crate::Error::NotFound {
          typ: journal::TYPE.to_string(),
          values: vec![(FIELD_ID.to_string(), journal_id.to_string())],
        });
      }

      let existings = Root::find_all(
        db,
        Some(Query {
          journal_id: HashSet::from_iter([journal_id]),
          name: HashSet::from_iter(names),
          ..Default::default()
        }),
        None,
      )
      .await?;
      if !existings.is_empty() {
        let existing_names = existings.iter().map(|model| model.name.clone()).sorted().join(", ");

        return Err(crate::Error::ExistingEntity {
          typ: TYPE.to_string(),
          values: vec![
            (FIELD_JOURNAL.to_string(), journal_id.to_string()),
            (FIELD_NAME.to_string(), existing_names),
          ],
        });
      }
    }

    let roots: Vec<_> = commands
      .into_iter()
      .filter_map(|command| {
        if let Some(journal) = journals.get(&command.journal_id) {
          Some(Root::new(
            None,
            journal,
            command.name,
            command.description,
            command.typ,
            command.date,
            command.tags,
            command.items,
            &accounts,
          ))
        } else {
          None
        }
      })
      .try_collect()?;

    Self::save(db, roots).await
  }
}
