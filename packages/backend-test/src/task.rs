use std::{future::Future, pin::Pin, sync::Arc};

use sea_orm_migration::{sea_orm::DatabaseTransaction, sea_query::IntoCondition};

use backend_shared::services::{AbstractReadService, AbstractWriteService, AuthUser, FindPageInput, Page};

type AsyncFn<I, O> = Arc<Box<dyn Send + Sync + Fn(I) -> Pin<Box<dyn Future<Output = anyhow::Result<O>>>>>>;
type AuthUserFn = AsyncFn<Arc<DatabaseTransaction>, AuthUser>;
type InputFn<I> = AsyncFn<(Arc<DatabaseTransaction>, Arc<AuthUser>), I>;
type CheckerFn<I, O> = AsyncFn<(Arc<DatabaseTransaction>, Arc<AuthUser>, I, anyhow::Result<O>), ()>;

pub type ServiceTask<S> = Task<
  <S as AbstractReadService>::Model,
  <S as AbstractReadService>::Query,
  <S as AbstractWriteService>::Command,
  <S as AbstractReadService>::Presentation,
>;

#[derive(Clone)]
pub struct Input<I, O>
where
  Self: Sync + Send,
  O: Send + Sync,
{
  pub name: String,
  pub auth_user: AuthUserFn,
  pub input: InputFn<I>,
  pub checker: CheckerFn<I, O>,
}

#[derive(Clone)]
pub enum Task<M, Q, C, P>
where
  M: Send + Sync,
  Q: Sized + IntoCondition + Clone,
  P: Send + Sync,
{
  FindById(Input<uuid::Uuid, Option<M>>),
  FindPage(Input<FindPageInput<Q>, Page<P>>),
  Handle(Input<C, Option<M>>),
  HandleAll(Input<Vec<C>, Vec<Option<P>>>),
}

impl<M, Q, C, P> Task<M, Q, C, P>
where
  M: Send + Sync,
  Q: Sized + IntoCondition + Clone,
  P: Send + Sync,
{
  pub fn name(&self) -> &str {
    match self {
      Task::FindById(Input { name, .. }) => name,
      Task::FindPage(Input { name, .. }) => name,
      Task::Handle(Input { name, .. }) => name,
      Task::HandleAll(Input { name, .. }) => name,
    }
  }

  pub fn auth_user(&self) -> &AuthUserFn {
    match self {
      Task::FindById(Input { auth_user, .. }) => auth_user,
      Task::FindPage(Input { auth_user, .. }) => auth_user,
      Task::Handle(Input { auth_user, .. }) => auth_user,
      Task::HandleAll(Input { auth_user, .. }) => auth_user,
    }
  }
}
