pub mod account;
pub mod group;
pub mod journal;
mod m20220916_095218_seed_data;
mod migrator;
pub mod record;
pub mod task;
pub mod user;

pub use migrator::Migrator;
pub use sea_orm_migration::MigratorTrait;

#[cfg(test)]
mod tests {
  use std::{env, sync::Arc};

  use crate::migrator::Migrator;
  use crate::task::{Input, ServiceTask, Task};

  use backend_shared::services::{AbstractReadService, AbstractWriteService};
  use sea_orm_migration::sea_orm::{Database, TransactionTrait};
  use sea_orm_migration::MigratorTrait;

  pub(crate) async fn run_test<S>(tasks: &[ServiceTask<S>]) -> backend_shared::Result<()>
  where
    S: AbstractReadService + AbstractWriteService + Sync + Send,
  {
    dotenv::from_filename(".test.env").unwrap();
    let _ = env_logger::try_init();

    let db = Database::connect(env::var("WHITE_RABBIT_DATABASE_URL").unwrap()).await?;
    Migrator::up(&db, None).await?;

    for task in tasks.iter() {
      log::info!("Start Task[{}]", task.name());

      let txn = Arc::new(db.begin().await?);

      let auth_user = Arc::new(task.auth_user()(txn.clone()).await?);

      match task {
        Task::FindById(Input { input, checker, .. }) => {
          let input = &*input.clone();
          let input = input((txn.clone(), auth_user.clone())).await?;
          let result = S::find_by_id(&*txn, &auth_user, input).await;
          checker((txn, auth_user, input, result)).await?;
        }
        Task::FindPage(Input { input, checker, .. }) => {
          let input = &*input.clone();
          let input = input((txn.clone(), auth_user.clone())).await?;
          let result = S::find_page(&*txn, &auth_user, input.clone()).await;
          checker((txn, auth_user, input, result)).await?;
        }
        Task::Handle(Input { input, checker, .. }) => {
          let input = &*input.clone();
          let input = input((txn.clone(), auth_user.clone())).await?;
          let result = S::handle(&*txn, &auth_user, input.clone()).await;
          checker((txn, auth_user, input, result)).await?;
        }
        Task::HandleAll(Input { input, checker, .. }) => {
          let input = &*input.clone();
          let input = input((txn.clone(), auth_user.clone())).await?;
          let result = S::handle_all(&*txn, &auth_user, input.clone()).await;
          checker((txn, auth_user, input, result)).await?;
        }
      }
    }

    Migrator::down(&db, None).await?;

    Ok(())
  }
}
