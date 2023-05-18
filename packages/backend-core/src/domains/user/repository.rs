use crate::domains::user::PrimaryKey;
use crate::{Repository, User, UserActiveModel, UserEntity};

pub struct UserRepository;

impl<'a> Repository<'a> for UserRepository {
  type AggregateRoot = User;

  type Model = User;
  type ActiveModel = UserActiveModel;
  type Entity = UserEntity;
  type Presentation = User;
  type PrimaryKey = PrimaryKey;
}
