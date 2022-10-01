use std::collections::HashMap;

use sea_orm::ConnectionTrait;

use crate::models::{user, IntoPresentation};

use super::{AbstractReadService, AuthUser};

pub trait AbstractCommand: Send + Sync + Clone {
  fn target_id(&self) -> Option<uuid::Uuid>;
  fn with_target_id(self, id: uuid::Uuid) -> Self;
}

#[async_trait::async_trait]
pub trait AbstractWriteService: AbstractReadService {
  type Command: AbstractCommand;

  async fn check_writeable(
    _conn: &impl ConnectionTrait,
    _user: &user::Model,
    _model: &Self::Model,
  ) -> anyhow::Result<()> {
    Ok(())
  }

  async fn handle(
    conn: &impl ConnectionTrait,
    operator: &AuthUser,
    command: Self::Command,
  ) -> anyhow::Result<Option<Self::Model>>;

  async fn handle_all(
    conn: &impl ConnectionTrait,
    operator: &AuthUser,
    commands: Vec<Self::Command>,
  ) -> anyhow::Result<Vec<Option<Self::Presentation>>> {
    let mut id_map = HashMap::<uuid::Uuid, uuid::Uuid>::new();
    let mut results = Vec::new();

    for mut command in commands {
      let target_id: Option<uuid::Uuid> = command.target_id();

      if let Some(ref target_id) = target_id {
        if let Some(id) = id_map.get(target_id) {
          command = command.with_target_id(*id);
        }
      }

      let result = Self::handle(conn, operator, command).await?;
      if let Some(result) = result {
        if let Some(target_id) = target_id {
          id_map.insert(target_id, Self::primary_value(&result));
        }
        let result = result.into_presentation(conn).await?;
        results.push(Some(result));
      } else {
        results.push(None);
      }
    }

    Ok(results)
  }
}
