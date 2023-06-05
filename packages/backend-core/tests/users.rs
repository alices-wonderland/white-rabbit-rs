use backend_core::user::User;
use backend_core::{user, AggregateRoot, Result};
use sea_orm::DatabaseConnection;

struct TestRunner {}

#[async_trait::async_trait]
impl test_suite::TestRunner for TestRunner {
  type AggregateRoot = User;

  async fn run_test(
    db: &DatabaseConnection,
    operator: Option<&User>,
    command: user::Command,
  ) -> Result<Vec<Self::AggregateRoot>> {
    User::handle(db, operator, command).await
  }
}

test_suite::generate_user_tests!(TestRunner);
