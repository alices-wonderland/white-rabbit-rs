use std::collections::HashSet;
use uuid::Uuid;

pub struct UserQuery {
  pub id: HashSet<Uuid>,
}
