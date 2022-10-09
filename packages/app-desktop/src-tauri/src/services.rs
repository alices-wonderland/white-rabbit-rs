use backend_shared::{
  models::{user, IntoPresentation, User},
  services::{AbstractReadService, AbstractWriteService, AuthUser, FindAllInput, FindPageInput, Page, FIELD_ID},
};
use futures::{stream, StreamExt, TryStreamExt};
use sea_orm::{ConnectionTrait, DatabaseConnection, EntityTrait, TransactionTrait};

async fn get_operator(conn: &impl ConnectionTrait, operator: Option<uuid::Uuid>) -> backend_shared::Result<AuthUser> {
  Ok(if let Some(operator) = operator {
    User::find_by_id(operator)
      .one(conn)
      .await?
      .ok_or_else(|| backend_shared::Error::NotFound {
        entity: user::TYPE.to_owned(),
        field: FIELD_ID.to_owned(),
        value: operator.to_string(),
      })?
      .into()
  } else {
    AuthUser::Id(("Provider".to_owned(), "Value".to_owned()))
  })
}

async fn find_by_id<S>(
  state: tauri::State<'_, DatabaseConnection>,
  operator: Option<uuid::Uuid>,
  input: uuid::Uuid,
) -> backend_shared::Result<Option<S::Presentation>>
where
  S: AbstractReadService + Send + Sync,
{
  let txn = state.inner().begin().await?;
  let operator: AuthUser = get_operator(&txn, operator).await?;
  let result = if let Some(result) = S::find_by_id(&txn, &operator, input).await? {
    Some(result.into_presentation(&txn).await?)
  } else {
    None
  };
  txn.commit().await?;
  Ok(result)
}

async fn find_page<S>(
  state: tauri::State<'_, DatabaseConnection>,
  operator: Option<uuid::Uuid>,
  input: FindPageInput<S::Query>,
) -> backend_shared::Result<Page<S::Presentation>>
where
  S: AbstractReadService + Send + Sync,
{
  let txn = state.inner().begin().await?;
  let operator: AuthUser = get_operator(&txn, operator).await?;
  let result = S::find_page(&txn, &operator, input).await?;
  txn.commit().await?;
  Ok(result)
}

async fn find_all<S>(
  state: tauri::State<'_, DatabaseConnection>,
  operator: Option<uuid::Uuid>,
  input: FindAllInput<S::Query>,
) -> backend_shared::Result<Vec<S::Presentation>>
where
  S: AbstractReadService + Send + Sync,
{
  let txn = state.inner().begin().await?;
  let operator: AuthUser = get_operator(&txn, operator).await?;

  let result = S::find_all(&txn, &operator, input).await?;
  let result = stream::iter(result)
    .then(|item| item.into_presentation(&txn))
    .try_collect()
    .await?;
  txn.commit().await?;
  Ok(result)
}

async fn handle<S>(
  state: tauri::State<'_, DatabaseConnection>,
  operator: uuid::Uuid,
  input: S::Command,
) -> backend_shared::Result<Option<S::Presentation>>
where
  S: AbstractWriteService + Send + Sync,
{
  let txn = state.inner().begin().await?;
  let operator: AuthUser = get_operator(&txn, Some(operator)).await?;
  let result = if let Some(result) = S::handle(&txn, &operator, input).await? {
    Some(result.into_presentation(&txn).await?)
  } else {
    None
  };
  txn.commit().await?;
  Ok(result)
}

async fn handle_all<S>(
  state: tauri::State<'_, DatabaseConnection>,
  operator: uuid::Uuid,
  input: Vec<S::Command>,
) -> backend_shared::Result<Vec<Option<S::Presentation>>>
where
  S: AbstractWriteService + Send + Sync,
{
  let txn = state.inner().begin().await?;
  let operator: AuthUser = get_operator(&txn, Some(operator)).await?;
  let result = S::handle_all(&txn, &operator, input).await?;
  txn.commit().await?;
  Ok(result)
}

macro_rules! create_commands {
  ($($typ: ident, $plural: ident, $service: ident);*) => {
    paste::paste! {$(
      #[tauri::command]
       async fn [<find_ $typ _by_id>](
        state: tauri::State<'_, DatabaseConnection>,
        operator: Option<uuid::Uuid>,
        input: uuid::Uuid,
      ) -> ::backend_shared::Result<Option<<::backend_shared::services::$service as AbstractReadService>::Presentation>> {
        find_by_id::<::backend_shared::services::$service>(state, operator, input).await
      }

      #[tauri::command]
       async fn [<find_ $plural>](
        state: tauri::State<'_, DatabaseConnection>,
        operator: Option<uuid::Uuid>,
        input: FindAllInput<<::backend_shared::services::$service as AbstractReadService>::Query>,
      ) -> ::backend_shared::Result<Vec<<::backend_shared::services::$service as AbstractReadService>::Presentation>> {
        find_all::<::backend_shared::services::$service>(state, operator, input).await
      }

      #[tauri::command]
       async fn [<find_ $typ _page>](
        state: tauri::State<'_, DatabaseConnection>,
        operator: Option<uuid::Uuid>,
        input: FindPageInput<<::backend_shared::services::$service as AbstractReadService>::Query>,
      ) -> ::backend_shared::Result<Page<<::backend_shared::services::$service as AbstractReadService>::Presentation>> {
        find_page::<::backend_shared::services::$service>(state, operator, input).await
      }

      #[tauri::command]
       async fn [<handle_ $typ>](
        state: tauri::State<'_, DatabaseConnection>,
        operator: uuid::Uuid,
        input: <::backend_shared::services::$service as AbstractWriteService>::Command,
      ) -> ::backend_shared::Result<Option<<::backend_shared::services::$service as AbstractReadService>::Presentation>> {
        handle::<::backend_shared::services::$service>(state, operator, input).await
      }

      #[tauri::command]
       async fn [<handle_all_ $plural>](
        state: tauri::State<'_, DatabaseConnection>,
        operator: uuid::Uuid,
        input: Vec<<::backend_shared::services::$service as AbstractWriteService>::Command>,
      ) -> ::backend_shared::Result<Vec<Option<<::backend_shared::services::$service as AbstractReadService>::Presentation>>> {
        handle_all::<::backend_shared::services::$service>(state, operator, input).await
      }
    )*}

    paste::paste! {
      pub const HANDLERS: fn(::tauri::Invoke) -> () = ::tauri::generate_handler![$([<find_ $typ _by_id>], [<find_ $plural>], [<find_ $typ _page>], [<handle_ $typ>], [<handle_all_ $plural>],)*];
    }
  };
}

create_commands!(
  user,
  users,
  UserService;
  group,
  groups,
  GroupService;
  journal,
  journals,
  JournalService;
  account,
  accounts,
  AccountService;
  record,
  records,
  RecordService
);
