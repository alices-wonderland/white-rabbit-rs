mod command;
mod database;
mod query;

use crate::entity::{
  account_tag, normalize_description, normalize_name, normalize_tags, normalize_unit,
};
pub use command::*;
pub use database::*;
use itertools::Itertools;
pub use query::*;
use sea_orm::sea_query::OnConflict;
use sea_orm::{ColumnTrait, DbConn, EntityTrait, IntoActiveModel, QueryFilter, QuerySelect};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

pub const TYPE: &str = "Account";

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Root {
  pub id: Uuid,
  pub journal_id: Uuid,
  pub name: String,
  pub description: String,
  pub unit: String,
  pub typ: Type,
  pub tags: HashSet<String>,
}
impl super::Root for Root {
  fn id(&self) -> Uuid {
    self.id
  }
}

impl Root {
  pub fn new(
    id: Option<Uuid>,
    journal_id: Uuid,
    name: impl ToString,
    description: impl ToString,
    unit: impl ToString,
    typ: Type,
    tags: impl IntoIterator<Item = impl ToString>,
  ) -> crate::Result<Root> {
    let name = normalize_name(TYPE, name)?;
    let description = normalize_description(TYPE, description)?;
    let unit = normalize_unit(TYPE, unit)?;
    let tags = normalize_tags(TYPE, tags)?;

    Ok(Root { id: id.unwrap_or_else(Uuid::new_v4), journal_id, name, description, unit, typ, tags })
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
        name: model.name,
        description: model.description,
        unit: model.unit,
        typ: model.typ,
        journal_id: model.journal_id,
        tags: HashSet::default(),
      });
      ids.insert(model.id);
    }

    let tags = account_tag::Entity::find()
      .filter(account_tag::Column::AccountId.is_in(ids))
      .all(db)
      .await?
      .into_iter()
      .into_group_map_by(|tag| tag.account_id)
      .into_iter()
      .map(|(k, v)| (k, v.into_iter().map(|m| m.tag).collect::<HashSet<_>>()))
      .collect::<HashMap<_, _>>();

    Ok(
      roots
        .into_iter()
        .map(|root| Self { tags: tags.get(&root.id).cloned().unwrap_or_default(), ..root })
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
    let mut tags: Vec<account_tag::ActiveModel> = vec![];

    for ref root in roots {
      model_ids.insert(root.id);
      models.push(
        Model {
          id: root.id,
          name: root.name.to_string(),
          description: root.description.to_string(),
          unit: root.unit.to_string(),
          typ: root.typ,
          journal_id: root.journal_id,
        }
        .into_active_model(),
      );
      for tag in &root.tags {
        tags.push(
          account_tag::Model { account_id: root.id, tag: tag.to_string() }.into_active_model(),
        )
      }
    }

    account_tag::Entity::delete_many()
      .filter(account_tag::Column::AccountId.is_in(model_ids.clone()))
      .exec(db)
      .await?;

    let mut on_conflict = OnConflict::column(Column::Id);
    on_conflict.update_columns([
      Column::Name,
      Column::Description,
      Column::Unit,
      Column::Typ,
      Column::JournalId,
    ]);

    Entity::insert_many(models).on_conflict(on_conflict).exec(db).await?;
    account_tag::Entity::insert_many(tags).exec(db).await?;

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

  pub async fn create(_db: &DbConn, _command: Vec<CommandCreate>) -> crate::Result<Vec<Root>> {
    Ok(vec![])
  }
}
