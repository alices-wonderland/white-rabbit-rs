use crate::entity;
use crate::entity::entry::{Query, Root, Type};
use itertools::Itertools;
use sea_orm::ConnectionTrait;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Serialize, Deserialize)]
pub struct Presentation {}

#[async_trait::async_trait]
impl entity::Presentation for Presentation {
  type R = Root;

  async fn from_roots(db: &impl ConnectionTrait, roots: Vec<Self::R>) -> crate::Result<Vec<Self>> {
    let journal_ids: HashSet<_> = roots.iter().map(|root| root.journal_id).collect();
    let _related_entries = Root::find_all(
      db,
      Some(Query { journal_id: journal_ids, typ: Some(Type::Record), ..Default::default() }),
      None,
      None,
    )
    .await?
    .into_iter()
    .into_group_map_by(|root| root.journal_id);

    Ok(vec![])
  }
}
