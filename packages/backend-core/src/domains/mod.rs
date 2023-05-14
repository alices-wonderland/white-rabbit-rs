mod user;

use serde::{Deserialize, Serialize};
use std::fmt::Debug;
pub use user::*;
use uuid::Uuid;

pub trait AggregateRoot<'a>: Debug + Send + Sync {
  fn id(&self) -> Uuid;
}

pub trait Presentation<'a>: Serialize + Deserialize<'a> {}
