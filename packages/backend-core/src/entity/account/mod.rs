mod command;
mod database;
mod query;

use crate::entity::{
  account, account_tag, journal, normalize_description, normalize_name, normalize_tags,
  normalize_unit, FIELD_ID, FIELD_JOURNAL, FIELD_NAME,
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
    let roots: Vec<Root> = roots.into_iter().collect();
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

  pub async fn create(db: &DbConn, commands: Vec<CommandCreate>) -> crate::Result<Vec<Root>> {
    if commands.is_empty() {
      return Ok(vec![]);
    }

    let journals = journal::Root::find_all(
      db,
      Some(journal::Query {
        id: commands.iter().map(|c| c.journal_id).collect::<HashSet<_>>(),
        ..Default::default()
      }),
      None,
    )
    .await?
    .into_iter()
    .map(|journal| (journal.id, journal))
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
        Some(account::Query {
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
          values: vec![(FIELD_NAME.to_string(), existing_names)],
        });
      }
    }

    let roots: Vec<_> = commands
      .into_iter()
      .map(|command| {
        Root::new(
          None,
          command.journal_id,
          command.name,
          command.description,
          command.unit,
          command.typ,
          command.tags,
        )
      })
      .try_collect()?;

    Self::save(db, roots).await
  }

  async fn do_update(
    db: &DbConn,
    journal: &journal::Root,
    accounts: &HashMap<Uuid, Root>,
    commands: &[CommandUpdate],
  ) -> crate::Result<Vec<Root>> {
    if commands.is_empty() {
      return Ok(vec![]);
    }

    let mut name_mappings = HashMap::new();
    let mut model_ids = HashSet::new();

    for command in commands {
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
        Some(Query {
          journal_id: HashSet::from_iter([journal.id]),
          name: name_mappings.keys().cloned().collect(),
          ..Default::default()
        }),
        None,
      )
      .await?
    };

    for model in existings_by_name {
      if let Some(updating_id) = name_mappings.get(&model.name) {
        if updating_id != &model.id && !name_mappings.values().contains(&model.id) {
          return Err(crate::Error::ExistingEntity {
            typ: TYPE.to_string(),
            values: vec![
              (FIELD_JOURNAL.to_string(), journal.id.to_string()),
              (FIELD_NAME.to_string(), model.name.clone()),
            ],
          });
        }
      }
    }

    let mut updated = Vec::new();
    for command in commands {
      let model = accounts.get(&command.id).ok_or_else(|| crate::Error::NotFound {
        typ: TYPE.to_string(),
        values: vec![(FIELD_ID.to_string(), command.id.to_string())],
      })?;

      if command.name.is_empty()
        && command.description.is_none()
        && command.unit.is_empty()
        && command.typ.is_none()
        && command.tags.is_none()
      {
        continue;
      }

      let model = Self::new(
        Some(model.id),
        model.journal_id,
        if command.name.is_empty() { model.name.clone() } else { command.name.clone() },
        if let Some(description) = &command.description {
          description.clone()
        } else {
          model.description.clone()
        },
        if command.unit.is_empty() { model.unit.clone() } else { command.unit.clone() },
        if let Some(typ) = &command.typ { *typ } else { model.typ },
        if let Some(tags) = &command.tags { tags.clone() } else { model.tags.clone() },
      )?;

      updated.push(model);
    }
    Ok(updated)
  }

  pub async fn update(db: &DbConn, commands: Vec<CommandUpdate>) -> crate::Result<Vec<Root>> {
    let model_ids = commands.iter().map(|command| command.id).collect::<HashSet<_>>();
    let models = Self::find_all(db, Some(Query { id: model_ids, ..Default::default() }), None)
      .await?
      .into_iter()
      .map(|model| (model.id, model))
      .collect::<HashMap<_, _>>();
    let journal_ids = models.values().map(|model| model.journal_id).collect::<HashSet<_>>();
    let journals = journal::Root::find_all(
      db,
      Some(journal::Query { id: journal_ids, ..Default::default() }),
      None,
    )
    .await?
    .into_iter()
    .map(|model| (model.id, model))
    .collect::<HashMap<_, _>>();

    let commands_by_journal: HashMap<Uuid, Vec<CommandUpdate>> = commands
      .into_iter()
      .filter_map(|command| {
        if let Some(model) = models.get(&command.id) {
          if let Some(journal) = journals.get(&model.journal_id) {
            return Some((journal.id, command));
          }
        }
        None
      })
      .fold(HashMap::new(), |mut acc, (journal_id, command)| {
        if let Some(commands) = acc.get_mut(&journal_id) {
          commands.push(command);
        } else {
          acc.insert(journal_id, vec![command]);
        }
        acc
      });

    let mut updated = Vec::new();
    for (journal_id, commands) in commands_by_journal {
      if let Some(journal) = journals.get(&journal_id) {
        for model in Self::do_update(db, journal, &models, &commands).await? {
          updated.push(model);
        }
      }
    }

    Self::save(db, updated).await
  }
}
