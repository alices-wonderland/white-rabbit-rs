mod command;
mod database;
mod query;

use crate::entity::{entry_item, entry_tag};
use chrono::NaiveDate;
pub use database::*;
use itertools::Itertools;
pub use query::*;
use rust_decimal::Decimal;
use sea_orm::sea_query::OnConflict;
use sea_orm::{ColumnTrait, DbConn, EntityTrait, IntoActiveModel, QueryFilter, QuerySelect};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

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
    journal_id: Uuid,
    name: impl ToString,
    description: impl ToString,
    typ: Type,
    date: NaiveDate,
    tags: impl IntoIterator<Item = impl ToString>,
    items: impl IntoIterator<Item = (Uuid, (Decimal, Option<Decimal>))>,
  ) -> crate::Result<Root> {
    Ok(Root {
      id: Uuid::new_v4(),
      journal_id,
      name: name.to_string().trim().to_string(),
      description: description.to_string().trim().to_string(),
      typ,
      date,
      tags: tags
        .into_iter()
        .map(|tag| tag.to_string().trim().to_string())
        .filter(|tag: &String| !tag.is_empty())
        .collect(),
      items: items.into_iter().collect(),
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
    let roots = roots.into_iter().collect::<Vec<_>>();
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

    Entity::insert_many(models).on_conflict(on_conflict).exec(db).await?;
    entry_tag::Entity::insert_many(tags).exec(db).await?;
    entry_item::Entity::insert_many(items).exec(db).await?;

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
}
