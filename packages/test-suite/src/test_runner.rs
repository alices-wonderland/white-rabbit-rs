use backend_core::user::User;
use backend_core::AggregateRoot;
use migration::sea_orm::DatabaseConnection;

#[async_trait::async_trait]
pub trait TestRunner {
  type AggregateRoot: AggregateRoot;

  async fn run_test(
    db: &DatabaseConnection,
    operator: Option<&User>,
    command: <Self::AggregateRoot as AggregateRoot>::Command,
  ) -> backend_core::Result<Vec<Self::AggregateRoot>>;
}
