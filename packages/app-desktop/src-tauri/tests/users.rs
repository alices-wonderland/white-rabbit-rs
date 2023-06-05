use backend_core::user::User;
use backend_core::{AggregateRoot, Result};
use test_suite::RunnerArgs;

async fn runner(RunnerArgs { db, operator, command }: RunnerArgs<User>) -> Result<Vec<User>> {
  User::handle(&db, operator.as_ref(), command).await
}

test_suite::generate_user_tests!(runner);
