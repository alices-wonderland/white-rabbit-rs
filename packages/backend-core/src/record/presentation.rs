use crate::record::{Record, RecordItem, State, Type};
use crate::user::User;
use crate::{AggregateRoot, Permission};
use chrono::NaiveDate;
use sea_orm::{ConnectionTrait, StreamTrait};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Presentation {
  pub id: Uuid,
  pub permission: Permission,
  pub journal: Uuid,
  pub name: String,
  pub description: String,
  #[serde(rename = "type")]
  pub typ: Type,
  pub date: NaiveDate,
  pub tags: HashSet<String>,
  pub items: HashSet<RecordItem>,
  pub state: State,
}

#[async_trait::async_trait]
impl crate::Presentation for Presentation {
  type AggregateRoot = Record;

  async fn from(
    db: &(impl ConnectionTrait + StreamTrait),
    operator: Option<&User>,
    roots: Vec<Self::AggregateRoot>,
  ) -> crate::Result<Vec<Self>> {
    let permissions = AggregateRoot::get_permission(db, operator, &roots).await?;
    let states = State::of(db, &roots).await?;

    Ok(
      roots
        .into_iter()
        .filter_map(|Record { id, journal, name, description, typ, date, tags, items }| {
          permissions.get(&id).map(|permission| Self {
            id,
            permission: *permission,
            journal,
            name,
            description,
            typ,
            date,
            tags,
            items,
            state: states.get(&id).cloned().unwrap_or(State::default(typ)),
          })
        })
        .collect(),
    )
  }
}
