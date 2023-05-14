use crate::domains::user::PrimaryKey;
use crate::{Repository, User, UserEntity};

pub struct UserRepository;

impl<'a> Repository<'a> for UserRepository {
  type AggregateRoot = User;

  type Model = User;
  type Entity = UserEntity;
  type Presentation = User;
  type PrimaryKey = PrimaryKey;
}
