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
use sea_orm::{ColumnTrait, EntityTrait, IntoActiveModel, Order, QueryOrder, QuerySelect};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

pub const TYPE: &str = "Journal";

#[derive(Debug, Default)]
pub struct Builder {
  id: Option<Uuid>,
  name: String,
  description: String,
  unit: String,
  tags: HashSet<String>,
}

impl From<Root> for Builder {
  fn from(value: Root) -> Self {
    Builder {
      id: Some(value.id),
      name: value.name,
      description: value.description,
      unit: value.unit,
      tags: value.tags,
    }
  }
}

impl Builder {
  pub fn build(self) -> crate::Result<Root> {
    let name = normalize_name(TYPE, self.name)?;
    let description = normalize_description(TYPE, self.description)?;
    let unit = normalize_unit(TYPE, self.unit)?;
    let tags = normalize_tags(TYPE, self.tags)?;
    Ok(Root { id: self.id.unwrap_or_else(Uuid::new_v4), name, description, unit, tags })
  }

  pub fn id(self, id: Uuid) -> Builder {
    Builder { id: Some(id), ..self }
  }

  pub fn name(self, name: impl ToString) -> Builder {
    Builder { name: name.to_string(), ..self }
  }

  pub fn description(self, description: impl ToString) -> Builder {
    Builder { description: description.to_string(), ..self }
  }

  pub fn unit(self, unit: impl ToString) -> Builder {
    Builder { unit: unit.to_string(), ..self }
  }

  pub fn tags(self, tags: impl IntoIterator<Item = impl ToString>) -> Builder {
    Builder { tags: tags.into_iter().map(|s| s.to_string()).collect(), ..self }
  }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sort {
  #[serde(rename = "name")]
  Name,
  #[serde(rename = "unit")]
  Unit,
  #[serde(rename = "-name")]
  MinusName,
  #[serde(rename = "-unit")]
  MinusUnit,
}

impl From<Sort> for (Column, Order) {
  fn from(value: Sort) -> Self {
    match value {
      Sort::Name => (Column::Name, Order::Asc),
      Sort::Unit => (Column::Unit, Order::Asc),
      Sort::MinusName => (Column::Name, Order::Desc),
      Sort::MinusUnit => (Column::Unit, Order::Desc),
    }
  }
}

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
  pub async fn from_model(
    db: &impl ConnectionTrait,
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
    db: &impl ConnectionTrait,
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
        Expr::col((Entity, Column::Name)).binary(BinOper::Custom("||"), Expr::current_timestamp()),
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

    Self::find_all(db, Some(Query { id: model_ids, ..Default::default() }), None, None).await
  }

  pub async fn delete(
    db: &impl ConnectionTrait,
    ids: impl IntoIterator<Item = Uuid>,
  ) -> crate::Result<()> {
    Entity::delete_many().filter(Column::Id.is_in(ids)).exec(db).await?;
    Ok(())
  }

  pub async fn find_one(
    db: &impl ConnectionTrait,
    query: Option<Query>,
  ) -> crate::Result<Option<Root>> {
    Ok(Self::find_all(db, query, Some(1), None).await?.into_iter().next())
  }

  pub async fn find_all(
    db: &impl ConnectionTrait,
    query: Option<Query>,
    limit: Option<u64>,
    sort: Option<Sort>,
  ) -> crate::Result<Vec<Root>> {
    let select =
      if let Some(query) = query { Entity::find().filter(query) } else { Entity::find() };
    let select = if let Some(sort) = sort {
      let (field, order) = Into::<(Column, Order)>::into(sort);
      select.order_by(field, order)
    } else {
      select
    };
    let models = select.limit(limit).all(db).await?;
    Self::from_model(db, models).await
  }

  pub async fn handle(db: &impl ConnectionTrait, command: Command) -> crate::Result<Vec<Root>> {
    match command {
      Command::Create(command) => Self::create(db, vec![command]).await,
      Command::Update(command) => Self::update(db, vec![command]).await,
      Command::Delete(CommandDelete { id }) => {
        Self::delete(db, id).await?;
        Ok(Vec::default())
      }
      Command::Batch(CommandBatch { create, update, delete }) => {
        let mut ids = HashSet::<Uuid>::new();

        Self::delete(db, delete).await?;

        for root in Self::update(db, update).await? {
          ids.insert(root.id);
        }

        for root in Self::create(db, create).await? {
          ids.insert(root.id);
        }

        Self::find_all(db, Some(Query { id: ids, ..Default::default() }), None, None).await
      }
    }
  }

  pub async fn create(
    db: &impl ConnectionTrait,
    commands: Vec<CommandCreate>,
  ) -> crate::Result<Vec<Root>> {
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
      Self::find_all(db, Some(Query { name: existing_names, ..Default::default() }), None, None)
        .await?;

    if !existings.is_empty() {
      let existing_names = existings.iter().map(|model| model.name.clone()).sorted().join(", ");

      return Err(crate::Error::ExistingEntity {
        typ: TYPE.to_string(),
        values: vec![(FIELD_NAME.to_string(), existing_names)],
      });
    }

    let roots: Vec<_> = commands_map
      .into_values()
      .map(|command| {
        Builder::default()
          .name(command.name)
          .unit(command.unit)
          .description(command.description)
          .tags(command.tags)
          .build()
      })
      .try_collect()?;
    Self::save(db, roots).await
  }

  pub async fn update(
    db: &impl ConnectionTrait,
    commands: Vec<CommandUpdate>,
  ) -> crate::Result<Vec<Root>> {
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

    let mut models =
      Self::find_all(db, Some(Query { id: model_ids, ..Default::default() }), None, None)
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

      let mut builder = Builder::from(model.clone());
      if !command.name.is_empty() {
        builder = builder.name(command.name.clone());
      }

      if let Some(description) = command.description {
        builder = builder.description(description.clone());
      }

      if !command.unit.is_empty() {
        builder = builder.unit(command.unit.clone());
      }

      if let Some(tags) = command.tags {
        builder = builder.tags(tags.clone());
      }

      let model = builder.build()?;

      models.insert(model.id, model.clone());
      updated.insert(model.id, model);
    }

    Self::save(db, updated.into_values()).await
  }
}

#[cfg(test)]
mod tests {
  use crate::entity::journal::{Column, Entity, Sort};
  use sea_orm::sea_query::{BinOper, Expr};
  use sea_orm::{
    ColumnTrait, DatabaseBackend, EntityTrait, Order, QueryFilter, QueryOrder, QueryTrait,
  };

  #[test]
  fn test_update_name() -> anyhow::Result<()> {
    assert_eq!(
      Entity::update_many()
        .col_expr(
          Column::Name,
          Expr::col((Entity, Column::Name))
            .binary(BinOper::Custom("||"), Expr::current_timestamp()),
        )
        .filter(Column::Id.is_in(vec!["id1", "id2"]))
        .build(DatabaseBackend::Sqlite)
        .to_string(),
      r#"UPDATE "journal" SET "name" = "journal"."name" || CURRENT_TIMESTAMP WHERE "journal"."id" IN ('id1', 'id2')"#
    );

    Ok(())
  }

  #[test]
  fn test_select_query() -> anyhow::Result<()> {
    let (field, order): (Column, Order) = (Sort::Name).into();

    assert_eq!(
      Entity::find().order_by(field, order).build(DatabaseBackend::Sqlite).to_string(),
      r#"SELECT "journal"."id", "journal"."name", "journal"."description", "journal"."unit" FROM "journal" ORDER BY "journal"."name" ASC"#
    );

    Ok(())
  }
}
