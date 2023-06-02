use crate::AggregateRoot;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

pub fn into_map<I>(items: I) -> HashMap<Uuid, I::Item>
where
  I: IntoIterator,
  I::Item: AggregateRoot,
{
  items.into_iter().map(|item| (item.id(), item)).collect()
}

pub fn get_ids<A>(items: &[A]) -> HashSet<Uuid>
where
  A: AggregateRoot,
{
  items.iter().map(|item| item.id()).collect()
}
