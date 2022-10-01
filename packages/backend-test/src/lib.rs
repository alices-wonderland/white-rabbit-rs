mod m20220916_095218_seed_data;
mod migrator;
pub mod task;
pub mod user;

pub use migrator::Migrator;
pub use sea_orm_migration::MigratorTrait;

#[cfg(test)]
mod tests {
  use std::{env, sync::Arc};

  use crate::migrator::Migrator;
  use crate::task::{AuthUserInput, Input, Task};
  use backend_shared::models::User;
  use backend_shared::services::{AbstractReadService, AbstractWriteService, AuthUser};
  use sea_orm_migration::sea_orm::{Database, EntityTrait, QueryFilter, TransactionTrait};
  use sea_orm_migration::MigratorTrait;

  pub(crate) async fn run_test<S>(tasks: &[Task<S::Model, S::Query, S::Command, S::Presentation>]) -> anyhow::Result<()>
  where
    S: AbstractReadService + AbstractWriteService + Sync + Send,
  {
    dotenv::from_filename(".test.env")?;
    let _ = env_logger::try_init();

    let db = Database::connect(env::var("WHITE_RABBIT_DATABASE_URL")?).await?;
    Migrator::up(&db, None).await?;

    for task in tasks.iter() {
      log::info!("Start Task[{}]", task.name());

      let txn = Arc::new(db.begin().await?);

      let auth_user = Arc::new(match task.auth_user() {
        AuthUserInput::User(user) => AuthUser::User(User::find().filter(user.clone()).one(&*txn).await?.unwrap()),
        AuthUserInput::Id(id) => AuthUser::Id(id.clone()),
      });

      match task {
        Task::FindById(Input { input, checker, .. }) => {
          let input = &*input.clone();
          let input = input(txn.clone()).await?;
          let result = S::find_by_id(&*txn, &*auth_user, input).await;
          checker((txn, auth_user, input, result)).await?;
        }
        Task::FindPage(Input { input, checker, .. }) => {
          let input = &*input.clone();
          let input = input(txn.clone()).await?;
          let result = S::find_page(&*txn, &*auth_user, input.clone()).await;
          checker((txn, auth_user, input, result)).await?;
        }
        Task::Handle(Input { input, checker, .. }) => {
          let input = &*input.clone();
          let input = input(txn.clone()).await?;
          let result = S::handle(&*txn, &*auth_user, input.clone()).await;
          checker((txn, auth_user, input, result)).await?;
        }
        Task::HandleAll(Input { input, checker, .. }) => {
          let input = &*input.clone();
          let input = input(txn.clone()).await?;
          let result = S::handle_all(&*txn, &*auth_user, input.clone()).await;
          checker((txn, auth_user, input, result)).await?;
        }
      }
    }

    Migrator::down(&db, None).await?;

    Ok(())
  }
}
