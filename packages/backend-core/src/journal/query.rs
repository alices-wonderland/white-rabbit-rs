use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct Query {
  pub id: HashSet<Uuid>,
  pub name: (String, bool),
  pub description: String,
  pub admin: HashSet<Uuid>,
  pub member: HashSet<Uuid>,
}

impl crate::Query for Query {}
