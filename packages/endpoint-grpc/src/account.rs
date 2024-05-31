use crate::interceptors::check_auth;
use crate::map_err;
use backend_core::entity::{account, ReadRoot, FIELD_ID};
use pb::account_service_server::{AccountService, AccountServiceServer};
use pb::{Account, AccountQuery, AccountType, AccountsResponse};
use sea_orm::DatabaseConnection;
use std::collections::HashSet;
use std::sync::Arc;
use tonic::codec::CompressionEncoding;
use tonic::codegen::InterceptedService;
use tonic::transport::server::Routes;
use tonic::{Code, Request, Response, Status};
use tonic_reflection::server::Builder;
use uuid::Uuid;

mod pb {
  tonic::include_proto!("whiterabbit.account");

  pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
    tonic::include_file_descriptor_set!("account_descriptor");
}

impl From<Vec<account::Root>> for AccountsResponse {
  fn from(results: Vec<account::Root>) -> Self {
    Self { values: results.into_iter().map(|model| model.into()).collect() }
  }
}

impl From<account::Root> for Account {
  fn from(model: account::Root) -> Self {
    Self {
      id: model.id.to_string(),
      journal_id: model.journal_id.to_string(),
      created_date: None,
      name: model.name,
      description: model.description,
      unit: model.unit,
      r#type: AccountType::from(model.typ).into(),
      tags: Vec::from_iter(model.tags),
    }
  }
}

impl From<AccountType> for account::Type {
  fn from(value: AccountType) -> Self {
    match value {
      AccountType::Income => account::Type::Income,
      AccountType::Expense => account::Type::Expense,
      AccountType::Asset => account::Type::Asset,
      AccountType::Liability => account::Type::Liability,
      AccountType::Equity => account::Type::Equity,
    }
  }
}

impl From<account::Type> for AccountType {
  fn from(value: account::Type) -> Self {
    match value {
      account::Type::Income => AccountType::Income,
      account::Type::Expense => AccountType::Expense,
      account::Type::Asset => AccountType::Asset,
      account::Type::Liability => AccountType::Liability,
      account::Type::Equity => AccountType::Equity,
    }
  }
}

impl TryFrom<AccountQuery> for account::Query {
  type Error = Status;

  fn try_from(value: AccountQuery) -> Result<Self, Self::Error> {
    Ok(Self {
      id: value
        .id
        .iter()
        .map(|v| v.parse())
        .collect::<Result<HashSet<_>, _>>()
        .map_err(|_| Status::new(Code::Internal, "Invalid UUID"))?,
      journal_id: value
        .journal_id
        .iter()
        .map(|v| v.parse())
        .collect::<Result<HashSet<_>, _>>()
        .map_err(|_| Status::new(Code::Internal, "Invalid UUID"))?,

      name: HashSet::from_iter(value.name.clone()),
      unit: value.unit.clone(),
      typ: AccountType::try_from(value.r#type).ok().map(|v| v.into()),
      tags: value.tags.iter().cloned().collect(),
      full_text: value.full_text.clone(),
    })
  }
}

#[derive(Debug)]
pub struct AccountServiceImpl {
  pub db: Arc<DatabaseConnection>,
}

#[tonic::async_trait]
impl AccountService for AccountServiceImpl {
  async fn find_all(
    &self,
    request: Request<AccountQuery>,
  ) -> Result<Response<AccountsResponse>, Status> {
    let query = request.get_ref();
    let results =
      account::Root::find_all(self.db.as_ref(), Some(query.clone().try_into()?), None, None)
        .await
        .map_err(map_err)?;

    Ok(Response::new(results.into()))
  }

  async fn find_by_id(&self, request: Request<String>) -> Result<Response<Account>, Status> {
    let id: Uuid =
      request.get_ref().parse().map_err(|_| Status::new(Code::Internal, "Invalid UUID"))?;
    if let Some(model) = account::Root::find_one(
      self.db.as_ref(),
      Some(account::Query { id: HashSet::from_iter([id]), ..Default::default() }),
    )
    .await
    .map_err(map_err)?
    .map(|model| model.into())
    {
      Ok(Response::new(model))
    } else {
      Err(map_err(backend_core::Error::NotFound(backend_core::error::ErrorNotFound {
        entity: account::TYPE.to_string(),
        values: vec![(FIELD_ID.to_string(), id.to_string())],
      })))
    }
  }
}

pub(crate) fn init(
  reflection_builder: Builder,
  routes: Routes,
  db: Arc<DatabaseConnection>,
) -> (Builder, Routes) {
  let service = InterceptedService::new(
    AccountServiceServer::new(AccountServiceImpl { db })
      .send_compressed(CompressionEncoding::Gzip)
      .accept_compressed(CompressionEncoding::Gzip),
    check_auth,
  );

  (
    reflection_builder.register_encoded_file_descriptor_set(pb::FILE_DESCRIPTOR_SET),
    routes.add_service(service),
  )
}
