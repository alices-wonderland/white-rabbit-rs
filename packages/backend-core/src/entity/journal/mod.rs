mod command;
mod database;
mod query;

pub use command::*;
pub use database::*;
pub use query::*;

use crate::entity::{
  journal_tag, normalize_description, normalize_name, normalize_tags, normalize_unit, FIELD_ID,
  FIELD_NAME,
};
use itertools::Itertools;
use sea_orm::entity::prelude::*;
use sea_orm::sea_query::{BinOper, OnConflict};
use sea_orm::{ColumnTrait, DbConn, EntityTrait, IntoActiveModel, QuerySelect};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

pub const TYPE: &str = "Journal";

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Root {
  pub id: Uuid,
  pub name: String,
  pub description: String,
  pub unit: String,
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
    name: impl ToString,
    description: impl ToString,
    unit: impl ToString,
    tags: impl IntoIterator<Item = impl ToString>,
  ) -> crate::Result<Root> {
    let name = normalize_name(TYPE, name)?;
    let description = normalize_description(TYPE, description)?;
    let unit = normalize_unit(TYPE, unit)?;
    let tags = normalize_tags(TYPE, tags)?;

    Ok(Root { id: id.unwrap_or_else(Uuid::new_v4), name, description, unit, tags })
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
        tags: HashSet::default(),
      });
      ids.insert(model.id);
    }

    let tags = journal_tag::Entity::find()
      .filter(journal_tag::Column::JournalId.is_in(ids))
      .all(db)
      .await?
      .into_iter()
      .into_group_map_by(|tag| tag.journal_id)
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
    let roots: Vec<Root> = roots.into_iter().collect();
    if roots.is_empty() {
      return Ok(roots);
    }

    let mut model_ids = HashSet::new();
    let mut models: Vec<ActiveModel> = vec![];
    let mut tags: Vec<journal_tag::ActiveModel> = vec![];

    for ref root in roots {
      model_ids.insert(root.id);
      models.push(
        Model {
          id: root.id,
          name: root.name.to_string(),
          description: root.description.to_string(),
          unit: root.unit.to_string(),
        }
        .into_active_model(),
      );
      for tag in &root.tags {
        tags.push(
          journal_tag::Model { journal_id: root.id, tag: tag.to_string() }.into_active_model(),
        );
      }
    }

    journal_tag::Entity::delete_many()
      .filter(journal_tag::Column::JournalId.is_in(model_ids.clone()))
      .exec(db)
      .await?;

    // Update unique column name to temp value
    Entity::update_many()
      .col_expr(
        Column::Name,
        Expr::col(Column::Name).binary(BinOper::Custom("||"), Expr::current_timestamp()),
      )
      .filter(Column::Id.is_in(model_ids.clone()))
      .exec(db)
      .await?;

    let mut on_conflict = OnConflict::column(Column::Id);
    on_conflict.update_columns([Column::Name, Column::Description, Column::Unit]);
    Entity::insert_many(models).on_conflict(on_conflict).exec(db).await?;

    if !tags.is_empty() {
      journal_tag::Entity::insert_many(tags).exec(db).await?;
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

    let mut existing_names = HashSet::new();
    let mut commands_map = HashMap::new();

    for command in commands {
      existing_names.insert(command.name.clone());
      commands_map.insert(command.name.clone(), command);
    }

    let existings =
      Self::find_all(db, Some(Query { name: existing_names, ..Default::default() }), None).await?;

    if !existings.is_empty() {
      let existing_names = existings.iter().map(|model| model.name.clone()).sorted().join(", ");

      return Err(crate::Error::ExistingEntity {
        typ: TYPE.to_string(),
        values: vec![(FIELD_NAME.to_string(), existing_names)],
      });
    }

    let roots: Vec<_> = commands_map
      .into_values()
      .map(|command| Root::new(None, command.name, command.description, command.unit, command.tags))
      .try_collect()?;
    Self::save(db, roots).await
  }

  pub async fn update(db: &DbConn, commands: Vec<CommandUpdate>) -> crate::Result<Vec<Root>> {
    if commands.is_empty() {
      return Ok(vec![]);
    }

    let mut name_mappings = HashMap::new();
    let mut model_ids = HashSet::new();

    for command in &commands {
      if !command.name.is_empty() {
        name_mappings.insert(command.name.clone(), command.id);
      }
      model_ids.insert(command.id);
    }

    let existings_by_name = if name_mappings.is_empty() {
      vec![]
    } else {
      Self::find_all(
        db,
        Some(Query { name: name_mappings.keys().cloned().collect(), ..Default::default() }),
        None,
      )
      .await?
    };

    for model in existings_by_name {
      // If so, throw the error:
      //  1. the name is already is system
      //  2. AND, the updating model is not the same with the existing model already with the name
      //  3. AND, the existing model will not change the name
      if let Some(updating_id) = name_mappings.get(&model.name) {
        if updating_id != &model.id && !name_mappings.values().contains(&model.id) {
          return Err(crate::Error::ExistingEntity {
            typ: TYPE.to_string(),
            values: vec![(FIELD_NAME.to_string(), model.name.clone())],
          });
        }
      }
    }

    let mut models = Self::find_all(db, Some(Query { id: model_ids, ..Default::default() }), None)
      .await?
      .into_iter()
      .map(|model| (model.id, model))
      .collect::<HashMap<_, _>>();

    let mut updated = HashMap::new();
    for command in commands {
      let model = models.get(&command.id).ok_or_else(|| crate::Error::NotFound {
        typ: TYPE.to_string(),
        values: vec![(FIELD_ID.to_string(), command.id.to_string())],
      })?;

      if command.name.is_empty()
        && command.description.is_none()
        && command.unit.is_empty()
        && command.tags.is_none()
      {
        continue;
      }

      let model = Self::new(
        Some(model.id),
        if command.name.is_empty() { model.name.clone() } else { command.name },
        if let Some(description) = command.description {
          description
        } else {
          model.description.clone()
        },
        if command.unit.is_empty() { model.unit.clone() } else { command.unit },
        if let Some(tags) = command.tags { tags } else { model.tags.clone() },
      )?;

      models.insert(model.id, model.clone());
      updated.insert(model.id, model);
    }

    Self::save(db, updated.into_values()).await
  }
}

#[cfg(test)]
mod tests {
  use crate::entity::journal::{Column, Entity};
  use sea_orm::sea_query::{BinOper, Expr};
  use sea_orm::{ColumnTrait, DatabaseBackend, EntityTrait, QueryFilter, QueryTrait};

  #[test]
  fn test_update_name() -> anyhow::Result<()> {
    assert_eq!(
      Entity::update_many()
        .col_expr(
          Column::Name,
          Expr::col(Column::Name).binary(BinOper::Custom("||"), Expr::current_timestamp()),
        )
        .filter(Column::Id.is_in(vec!["id1", "id2"]))
        .build(DatabaseBackend::Sqlite)
        .to_string(),
      r#"UPDATE "journal" SET "name" = "name" || CURRENT_TIMESTAMP WHERE "journal"."id" IN ('id1', 'id2')"#
    );

    Ok(())
  }
}
